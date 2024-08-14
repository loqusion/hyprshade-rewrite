use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Display},
};

use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{builder::TypedValueParser, CommandFactory};
use color_eyre::owo_colors::OwoColorize;

use crate::template::{MergeDeep, TemplateData, TemplateDataMap};

const LHS_SEP: &str = ".";
const ASSIGN: &str = "=";

#[derive(Debug, Clone)]
pub struct VarArg {
    lhs: Lhs,
    rhs: String,
}

#[derive(Debug, Clone)]
struct Lhs(Vec<String>);

impl Display for VarArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", &self.lhs, ASSIGN, &self.rhs)
    }
}

impl Display for Lhs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0.join(LHS_SEP))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct VarArgParser;

impl TypedValueParser for VarArgParser {
    type Value = VarArg;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let value = match value.to_str() {
            Some(value) => value,
            None => {
                return Err(clap::Error::new(ErrorKind::InvalidUtf8).with_cmd(cmd));
            }
        };

        fn invalid_arg(arg: Option<&clap::Arg>, value: &str, suggested: &[&str]) -> clap::Error {
            let mut err = clap::Error::new(ErrorKind::ValueValidation);
            err.insert(
                ContextKind::InvalidArg,
                ContextValue::String(arg.unwrap().to_string()),
            );
            err.insert(
                ContextKind::InvalidValue,
                ContextValue::String(value.to_string()),
            );
            if !suggested.is_empty() {
                err.insert(
                    ContextKind::Suggested,
                    ContextValue::StyledStrs(
                        suggested.iter().map(|s| s.to_string().into()).collect(),
                    ),
                );
            }
            err
        }

        let ret = match value.split(ASSIGN).collect::<Vec<_>>()[..] {
            ["", _] => {
                return Err(invalid_arg(arg, value, &["empty KEY"]).with_cmd(cmd));
            }
            [_, ""] => {
                return Err(invalid_arg(arg, value, &["empty VALUE"]).with_cmd(cmd));
            }
            [lhs, rhs] => {
                let rhs = rhs.to_string();
                let lhs: Vec<String> = lhs
                    .split(LHS_SEP)
                    .enumerate()
                    .map(|(i, s)| {
                        if s.is_empty() {
                            let sep_count = lhs.matches(LHS_SEP).count();
                            let suggestions: &[&str] = match i {
                                0 => &["KEY must not begin with '.'"],
                                _ if i == sep_count => &["KEY must not end with '.'"],
                                _ => &["each word in KEY must be separated by exactly one '.'"],
                            };
                            Err(invalid_arg(arg, value, suggestions).with_cmd(cmd))
                        } else {
                            Ok(s.to_string())
                        }
                    })
                    .collect::<Result<_, _>>()?;
                VarArg { lhs: Lhs(lhs), rhs }
            }
            [_, _, ..] => {
                return Err(invalid_arg(arg, value, &["too many equals signs"]).with_cmd(cmd));
            }
            [""] => {
                return Err(invalid_arg(arg, value, &["empty"]).with_cmd(cmd));
            }
            [_] => {
                return Err(invalid_arg(arg, value, &["no equals sign"]).with_cmd(cmd));
            }
            [] => {
                return Err(invalid_arg(arg, value, &["no content"]).with_cmd(cmd));
            }
        };

        Ok(ret)
    }
}

pub trait MergeVarArg: CommandFactory {
    fn merge_var_into_data(
        vars: Vec<VarArg>,
        arg_name: &str,
    ) -> Result<TemplateDataMap, clap::Error> {
        check_no_conflicts::<Self>(&vars, arg_name)?;

        let map = vars
            .into_iter()
            .try_fold(TemplateDataMap::new(), |mut map, arg| {
                let data = match TemplateData::from_cli_arg(&arg.rhs) {
                    Ok(data) => data,
                    Err(err) => {
                        let mut messages = Vec::from([err.to_string()]);
                        let mut err: &dyn Error = &err;
                        while let Some(source) = err.source() {
                            messages.push(source.to_string());
                            err = source;
                        }

                        return Err(clap_error::value_validation(
                            &Self::command(),
                            arg_name,
                            &arg.to_string(),
                            &messages.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                        ));
                    }
                };
                let VarArg { mut lhs, .. } = arg;
                let last = lhs.0.pop().expect("lhs should be non-empty");
                let data = lhs
                    .0
                    .into_iter()
                    .rev()
                    .fold(TemplateDataMap::from([(last, data)]), |map, key| {
                        TemplateDataMap::from([(key, TemplateData::from(map))])
                    });

                map.merge_deep_force(data);

                Ok(map)
            })?;
        Ok(map)
    }
}

fn check_no_conflicts<C: CommandFactory>(
    vars: &[VarArg],
    arg_name: &str,
) -> Result<(), clap::Error> {
    vars.iter().try_fold(
        HashMap::new(),
        |mut map: HashMap<&[_], (&VarArg, bool)>, arg| {
            for i in 1..=arg.lhs.0.len() {
                let is_leaf = i == arg.lhs.0.len();
                let prefix = &arg.lhs.0[..i];

                match map.get(prefix) {
                    Some(&(prior, is_leaf_prior)) if is_leaf_prior || is_leaf => {
                        return Err(clap_error::argument_conflict(
                            &C::command(),
                            arg_name,
                            arg,
                            prior,
                            &[&format!(
                                "'{}' would override '{}'",
                                format_args!("--{arg_name} {arg}").yellow(),
                                format_args!("--{arg_name} {prior}").yellow(),
                            )],
                        ));
                    }
                    _ => (),
                }

                map.insert(prefix, (arg, is_leaf));
            }
            Ok(map)
        },
    )?;
    Ok(())
}

mod clap_error {
    use clap::error::{ContextKind, ContextValue, ErrorKind};

    use super::VarArg;

    type Error = clap::Error;

    pub(super) fn argument_conflict(
        cmd: &clap::Command,
        arg_name: &str,
        arg: &VarArg,
        other: &VarArg,
        suggested: &[&str],
    ) -> Error {
        let mut err = Error::new(ErrorKind::ArgumentConflict).with_cmd(cmd);

        err.insert(
            ContextKind::InvalidArg,
            ContextValue::String(format!("--{arg_name} {arg}")),
        );
        err.insert(
            ContextKind::PriorArg,
            ContextValue::String(format!("--{arg_name} {other}")),
        );
        if !suggested.is_empty() {
            let suggested = suggested.iter().map(|s| s.to_string().into()).collect();
            err.insert(ContextKind::Suggested, ContextValue::StyledStrs(suggested));
        }

        err
    }

    pub(super) fn value_validation(
        cmd: &clap::Command,
        arg_name: &str,
        val: &str,
        suggested: &[&str],
    ) -> Error {
        let mut err = Error::new(ErrorKind::ValueValidation).with_cmd(cmd);

        err.insert(
            ContextKind::InvalidArg,
            ContextValue::String(format!("--{arg_name}")),
        );
        err.insert(
            ContextKind::InvalidValue,
            ContextValue::String(val.to_string()),
        );
        if !suggested.is_empty() {
            let suggested = suggested.iter().map(|s| s.to_string().into()).collect();
            err.insert(ContextKind::Suggested, ContextValue::StyledStrs(suggested));
        }

        err
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(var_strs: &[&'_ str], is_valid: bool) {
        struct CommandFactoryImpl;

        impl CommandFactory for CommandFactoryImpl {
            fn command() -> clap::Command {
                clap::Command::new("test")
                    .arg(
                        clap::Arg::new("var")
                            .long("var")
                            .value_name("KEY=VALUE")
                            .action(clap::ArgAction::Append)
                            .value_parser(VarArgParser),
                    )
                    .no_binary_name(true)
            }

            fn command_for_update() -> clap::Command {
                Self::command()
            }
        }

        let args = {
            let mut args = Vec::with_capacity(2 * var_strs.len());
            for var in var_strs {
                args.push("--var");
                args.push(*var);
            }
            args
        };

        let vars: Vec<VarArg> = CommandFactoryImpl::command()
            .get_matches_from(args)
            .get_occurrences("var")
            .unwrap()
            .flat_map(|values| {
                values
                    .map(|var: &VarArg| var.to_owned())
                    .collect::<Vec<_>>()
            })
            .collect();

        let err = check_no_conflicts::<CommandFactoryImpl>(&vars, "var");
        if is_valid {
            assert!(err.is_ok(), "Error: {}", err.unwrap_err());
        } else {
            assert!(err.is_err(), "Expected conflict: {:#?}", var_strs);
        }
    }

    #[test]
    fn var_arg_valid() {
        check(&["strength=1", "type=red-green"], true);
        check(
            &["balance.red=1", "balance.green=2", "balance.blue=3"],
            true,
        );
        check(
            &["balance.foo=1", "balance.bar.baz=2", "balance.bar.qux=3"],
            true,
        );
    }

    #[test]
    fn var_arg_invalid() {
        check(&["balance=1", "balance=2"], false);
        check(&["balance=1", "balance.red=2"], false);
        check(&["balance=1", "balance.foo.bar=2"], false);
        check(&["balance=1", "balance.foo.bar.baz=2"], false);

        check(&["balance.red=1", "balance=2"], false);
        check(&["balance.red=1", "balance.red=2"], false);
        check(&["balance.red=1", "balance.red.foo=2"], false);
        check(&["balance.red=1", "balance.red.foo.baz=2"], false);

        check(&["balance.foo.red=1", "balance=2"], false);
        check(&["balance.foo.red=1", "balance.foo=2"], false);
        check(&["balance.foo.red=1", "balance.foo.red=2"], false);
        check(&["balance.foo.red=1", "balance.foo.red.baz=2"], false);
        check(&["balance.foo.red=1", "balance.foo.red.baz.qux=2"], false);
    }
}
