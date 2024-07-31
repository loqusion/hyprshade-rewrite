use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use crate::template::{MergeDeep, TemplateData, TemplateDataCliParseError, TemplateDataMap};
use clap::builder::TypedValueParser;
use clap::error::{ContextKind, ContextValue, ErrorKind};

const LHS_SEP: &str = ".";
const ASSIGN: &str = "=";

#[derive(Debug, Clone)]
pub struct VarArg {
    lhs: Vec<String>,
    rhs: String,
}

impl VarArg {
    pub fn merge_into_data(
        vars: Vec<VarArg>,
        arg_name: &str,
    ) -> Result<TemplateDataMap, VarArgMergeError> {
        VarArg::check_no_conflicts(&vars, arg_name)?;

        let map = vars
            .into_iter()
            .try_fold(TemplateDataMap::new(), |mut map, arg| {
                let data = match TemplateData::from_cli_arg(&arg.rhs) {
                    Ok(data) => data,
                    Err(source) => {
                        return Err(VarArgMergeError::Parse {
                            arg,
                            arg_name: arg_name.to_owned(),
                            source,
                        })
                    }
                };
                let VarArg { mut lhs, .. } = arg;
                let last = lhs.pop().expect("lhs is non-empty");
                let data = lhs
                    .into_iter()
                    .rev()
                    .fold(TemplateDataMap::from([(last, data)]), |map, key| {
                        TemplateDataMap::from([(key, TemplateData::from(map))])
                    });

                map.merge_deep(data);

                Ok(map)
            })?;
        Ok(map)
    }

    fn check_no_conflicts(vars: &[VarArg], arg_name: &str) -> Result<(), VarArgMergeError> {
        vars.iter().try_fold(
            HashMap::new(),
            |mut map: HashMap<&[_], (&VarArg, bool)>, arg| -> Result<_, VarArgMergeError> {
                for i in 1..=arg.lhs.len() {
                    let is_leaf = i == arg.lhs.len();
                    let prefix = &arg.lhs[..i];

                    match map.get(prefix) {
                        Some(&(prior, is_leaf_prior)) if is_leaf_prior || is_leaf => {
                            return Err(VarArgMergeError::Conflict {
                                prior: prior.clone(),
                                conflicting: arg.clone(),
                                arg_name: arg_name.to_owned(),
                            });
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
}

impl Display for VarArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", &self.lhs.join(LHS_SEP), ASSIGN, &self.rhs)
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
            [lhs, rhs] => {
                let rhs = if rhs.is_empty() {
                    return Err(invalid_arg(arg, value, &["empty value"]).with_cmd(cmd));
                } else {
                    rhs.to_string()
                };
                let lhs: Vec<String> = lhs
                    .split(LHS_SEP)
                    .map(|s| {
                        if s.is_empty() {
                            Err(invalid_arg(arg, value, &["empty key"]).with_cmd(cmd))
                        } else {
                            Ok(s.to_string())
                        }
                    })
                    .collect::<Result<_, _>>()?;
                VarArg { lhs, rhs }
            }
            [_, _, ..] => {
                return Err(invalid_arg(arg, value, &["too many equals signs"]).with_cmd(cmd));
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

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum VarArgMergeError {
    #[error("--{arg_name}: '{conflicting}' conflicts with '{prior}'")]
    Conflict {
        prior: VarArg,
        conflicting: VarArg,
        arg_name: String,
    },
    #[error("in '--{arg_name} {arg}'")]
    Parse {
        arg: VarArg,
        arg_name: String,
        source: TemplateDataCliParseError,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(var_strs: &[&'_ str], is_valid: bool) {
        let cmd = clap::Command::new("test")
            .arg(
                clap::Arg::new("var")
                    .long("var")
                    .value_name("KEY=VALUE")
                    .action(clap::ArgAction::Append)
                    .value_parser(VarArgParser),
            )
            .no_binary_name(true);

        let args = {
            let mut args = Vec::with_capacity(2 * var_strs.len());
            for var in var_strs {
                args.push("--var");
                args.push(*var);
            }
            args
        };

        let vars: Vec<VarArg> = cmd
            .get_matches_from(args)
            .get_occurrences("var")
            .unwrap()
            .flat_map(|values| {
                values
                    .map(|var: &VarArg| var.to_owned())
                    .collect::<Vec<_>>()
            })
            .collect();

        let err = VarArg::check_no_conflicts(&vars, "var");
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
