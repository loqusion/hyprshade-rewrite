use std::{
    collections::HashMap,
    error::Error,
    fmt::{self, Display},
};

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
                return Err(clap_error::invalid_utf8(cmd));
            }
        };

        let ret = match value.split(ASSIGN).collect::<Vec<_>>()[..] {
            ["", _] => {
                return Err(clap_error::value_validation(
                    cmd,
                    arg.unwrap().to_string(),
                    value.to_string(),
                    &["empty KEY".into()],
                ));
            }
            [_, ""] => {
                return Err(clap_error::value_validation(
                    cmd,
                    arg.unwrap().to_string(),
                    value.to_string(),
                    &["empty VALUE".into()],
                ));
            }
            [lhs, rhs] => {
                let rhs = rhs.to_string();
                let lhs: Vec<String> =
                    lhs.split(LHS_SEP)
                        .enumerate()
                        .map(|(i, s)| {
                            if s.is_empty() {
                                let sep_count = lhs.matches(LHS_SEP).count();
                                let suggestions: &[String] = match i {
                                    0 => &["KEY must not begin with '.'".into()],
                                    _ if i == sep_count => &["KEY must not end with '.'".into()],
                                    _ => &["each word in KEY must be separated by exactly one '.'"
                                        .into()],
                                };
                                Err(clap_error::value_validation(
                                    cmd,
                                    arg.unwrap().to_string(),
                                    value.to_string(),
                                    suggestions,
                                ))
                            } else {
                                Ok(s.to_string())
                            }
                        })
                        .collect::<Result<_, _>>()?;
                VarArg { lhs: Lhs(lhs), rhs }
            }
            [_, _, ..] => {
                return Err(clap_error::value_validation(
                    cmd,
                    arg.unwrap().to_string(),
                    value.to_string(),
                    &["too many equals signs".into()],
                ));
            }
            [""] => {
                return Err(clap_error::value_validation(
                    cmd,
                    arg.unwrap().to_string(),
                    value.to_string(),
                    &["empty".into()],
                ));
            }
            [_] => {
                return Err(clap_error::value_validation(
                    cmd,
                    arg.unwrap().to_string(),
                    value.to_string(),
                    &["no equals sign".into()],
                ));
            }
            [] => {
                return Err(clap_error::value_validation(
                    cmd,
                    arg.unwrap().to_string(),
                    value.to_string(),
                    &["no content".into()],
                ));
            }
        };

        Ok(ret)
    }
}

pub trait MergeVarArg: CommandFactory {
    fn merge_into_data(vars: Vec<VarArg>, arg_name: &str) -> Result<TemplateDataMap, clap::Error> {
        check_no_conflicts::<Self>(&vars, arg_name)?;

        let map = vars
            .into_iter()
            .try_fold(TemplateDataMap::new(), |mut map, arg| {
                let leaf_data = match TemplateData::from_cli_arg(&arg.rhs) {
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
                            format!("--{arg_name}"),
                            arg.to_string(),
                            &messages,
                        ));
                    }
                };
                let mut lhs_iter = arg.lhs.0.into_iter().rev();
                let leaf_key = lhs_iter.next().expect("lhs should be non-empty");
                let data = lhs_iter.fold(
                    TemplateDataMap::from([(leaf_key, leaf_data)]),
                    |map, key| TemplateDataMap::from([(key, TemplateData::from(map))]),
                );

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
                            format!("--{arg_name} {arg}"),
                            format!("--{arg_name} {prior}"),
                            &[format!(
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

    type Error = clap::Error;

    pub(super) fn invalid_utf8(cmd: &clap::Command) -> Error {
        Error::new(ErrorKind::InvalidUtf8).with_cmd(cmd)
    }

    pub(super) fn argument_conflict(
        cmd: &clap::Command,
        arg: String,
        other: String,
        suggested: &[String],
    ) -> Error {
        let mut err = Error::new(ErrorKind::ArgumentConflict).with_cmd(cmd);

        err.insert(ContextKind::InvalidArg, ContextValue::String(arg));
        err.insert(ContextKind::PriorArg, ContextValue::String(other));
        if !suggested.is_empty() {
            let suggested = suggested.iter().map(|s| s.into()).collect();
            err.insert(ContextKind::Suggested, ContextValue::StyledStrs(suggested));
        }

        err
    }

    pub(super) fn value_validation(
        cmd: &clap::Command,
        arg: String,
        val: String,
        suggested: &[String],
    ) -> Error {
        let mut err = Error::new(ErrorKind::ValueValidation).with_cmd(cmd);

        err.insert(ContextKind::InvalidArg, ContextValue::String(arg));
        err.insert(ContextKind::InvalidValue, ContextValue::String(val));
        if !suggested.is_empty() {
            let suggested = suggested.iter().map(|s| s.into()).collect();
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
