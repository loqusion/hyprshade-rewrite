use std::{collections::HashMap, error::Error, fmt, iter};

use clap::{builder::TypedValueParser, CommandFactory};
use color_eyre::owo_colors::OwoColorize;

use crate::template::{MergeDeep, TemplateData, TemplateDataMap};

#[derive(Debug, Clone)]
pub struct VarArg {
    name: ArgName,
    lhs: Lhs,
    rhs: String,
}

#[derive(Debug, Clone)]
enum ArgName {
    Unknown,
    Short(char),
    Long(String),
}

#[derive(Debug, Clone)]
struct Lhs(Vec<String>);

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
        // This assertion is necessary for unambiguously referencing the option name in error messages
        debug_assert!(
            arg.map(|a| {
                a.get_long().iter().len()
                    + a.get_short().iter().len()
                    + a.get_all_aliases().map_or(0, |al| al.len())
                    + a.get_all_short_aliases().map_or(0, |al| al.len())
            })
            .unwrap_or(0)
                <= 1,
            "There must be only one way to refer to {arg_name} in subcommand {cmd}; remove all but one of the following: long, short, alias, short_alias",
            arg_name = arg.map_or_else(
                || format!("argument using `{}`", std::any::type_name::<Self>()),
                |a| format!("`{}`", a.get_id())
            ),
            cmd = format_args!("'{}'", cmd.get_name())
        );

        let value = match value.to_str() {
            Some(value) => value,
            None => {
                return Err(clap_error::invalid_utf8(cmd));
            }
        };

        let value_validation = |suggested: &[String]| {
            clap_error::value_validation(
                cmd,
                arg.unwrap().to_string(),
                value.to_string(),
                suggested,
            )
        };

        let ret = match value.split(VarArg::ASSIGN).collect::<Vec<_>>()[..] {
            [] => {
                return Err(value_validation(&["no content".into()]));
            }
            [""] => {
                return Err(value_validation(&["empty".into()]));
            }
            [_] => {
                return Err(value_validation(&["no equals sign".into()]));
            }
            ["", _] => {
                return Err(value_validation(&["empty KEY".into()]));
            }
            [_, ""] => {
                return Err(value_validation(&["empty VALUE".into()]));
            }
            [_, _, _, ..] => {
                return Err(value_validation(&["too many equals signs".into()]));
            }
            [lhs, rhs] => {
                let rhs = rhs.to_string();
                let lhs: Vec<String> =
                    lhs.split(VarArg::LHS_SEP)
                        .enumerate()
                        .map(|(i, s)| {
                            if s.is_empty() {
                                let sep_count = lhs.matches(VarArg::LHS_SEP).count();
                                let suggestions: &[String] = match i {
                                    0 => &["KEY must not begin with '.'".into()],
                                    _ if i == sep_count => &["KEY must not end with '.'".into()],
                                    _ => &["each word in KEY must be separated by exactly one '.'"
                                        .into()],
                                };
                                Err(value_validation(suggestions))
                            } else {
                                Ok(s.to_string())
                            }
                        })
                        .collect::<Result<_, _>>()?;
                let name = arg
                    .and_then(ArgName::from_clap_arg)
                    .unwrap_or(ArgName::Unknown);
                VarArg {
                    name,
                    lhs: Lhs(lhs),
                    rhs,
                }
            }
        };

        Ok(ret)
    }
}

pub trait MergeVarArg: CommandFactory {
    fn merge_into_data(vars: Vec<VarArg>) -> Result<TemplateDataMap, clap::Error> {
        check_no_conflicts::<Self>(&vars)?;

        let map = vars
            .into_iter()
            .try_fold(TemplateDataMap::new(), |mut map, arg| {
                let leaf_data = match TemplateData::from_cli_arg(&arg.rhs) {
                    Ok(data) => data,
                    Err(err) => {
                        let messages =
                            iter::successors(Some(&err as &dyn Error), |err| (*err).source())
                                .map(ToString::to_string)
                                .collect::<Vec<_>>();

                        return Err(clap_error::value_validation(
                            &Self::command(),
                            arg.display_name(),
                            arg.display_value(),
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

fn check_no_conflicts<C: CommandFactory>(vars: &[VarArg]) -> Result<(), clap::Error> {
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
                            arg.display(),
                            prior.display(),
                            &[format!(
                                "'{}' would override '{}'",
                                arg.display().yellow(),
                                prior.display().yellow(),
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

impl VarArg {
    const LHS_SEP: &'static str = ".";
    const ASSIGN: &'static str = "=";

    fn display(&self) -> String {
        format!(
            "{opt} {value}",
            opt = self.display_name(),
            value = self.display_value()
        )
    }

    fn display_name(&self) -> String {
        self.name.to_string()
    }

    fn display_value(&self) -> String {
        format!(
            "{lhs}{eq}{rhs}",
            lhs = &self.lhs,
            eq = VarArg::ASSIGN,
            rhs = &self.rhs
        )
    }
}

impl fmt::Display for ArgName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArgName::Unknown => write!(f, "--<unknown>"),
            ArgName::Short(c) => write!(f, "-{c}"),
            ArgName::Long(s) => write!(f, "--{s}"),
        }
    }
}

impl fmt::Display for Lhs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0.join(VarArg::LHS_SEP))
    }
}

impl ArgName {
    fn from_clap_arg(arg: &clap::Arg) -> Option<ArgName> {
        arg.get_long()
            .map(ArgName::from)
            .or_else(|| arg.get_short().map(ArgName::from))
    }
}

impl From<char> for ArgName {
    fn from(value: char) -> Self {
        ArgName::Short(value)
    }
}

impl From<String> for ArgName {
    fn from(value: String) -> Self {
        ArgName::Long(value)
    }
}

impl From<&str> for ArgName {
    fn from(value: &str) -> Self {
        ArgName::from(value.to_owned())
    }
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

        let err = check_no_conflicts::<CommandFactoryImpl>(&vars);
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
