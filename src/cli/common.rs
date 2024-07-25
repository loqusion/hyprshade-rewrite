use std::{collections::HashMap, convert::Infallible, str::FromStr};

pub const SHADER_HELP: &str = "Which shader to turn on";
pub const SHADER_HELP_LONG: &str = "\
    Which shader to turn on\n\
\n\
    May be a name (e.g. `blue-light-filter`)\n\
    or a path (e.g. `~/.config/hypr/shaders/blue-light-filter.glsl`)\
";

#[derive(Debug, Clone)]
pub struct ArgVar(String);

impl FromStr for ArgVar {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl TryFrom<ArgVar> for HashMap<String, mustache::Data> {
    type Error = ArgVarError;

    fn try_from(value: ArgVar) -> Result<Self, Self::Error> {
        let ArgVar(s) = value;
        match s.split('=').collect::<Vec<_>>()[..] {
            ["", _] => Err(ArgVarError::EmptyKey(s)),
            [_, ""] => Err(ArgVarError::EmptyValue(s)),
            [dotted_key, value] => {
                let mut key_split = dotted_key.split('.').rev();
                let mut map: HashMap<String, mustache::Data> = HashMap::from([(
                    key_split.next().unwrap().to_string(),
                    mustache::Data::String(value.to_string()),
                )]);

                for key in key_split {
                    map = HashMap::from([(key.to_string(), mustache::Data::Map(map))]);
                }
                Ok(map)
            }
            [_, _, ..] => Err(ArgVarError::TooManyEquals(s)),
            [_] => Err(ArgVarError::NoEquals(s)),
            [] => Err(ArgVarError::NoContent),
        }
    }
}

impl FromIterator<ArgVar> for Result<mustache::Data, Vec<ArgVarError>> {
    fn from_iter<T: IntoIterator<Item = ArgVar>>(iter: T) -> Self {
        let mut map: HashMap<String, mustache::Data> = HashMap::new();

        let (things, errors): (Vec<_>, Vec<_>) = iter
            .into_iter()
            .map(|arg_var| arg_var.try_into())
            .partition(Result::is_ok);
        let things: Vec<_> = things.into_iter().map(Result::unwrap).collect();
        let mut errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();

        let merge_errors = things
            .into_iter()
            .filter_map(|data| ArgVar::deep_merge(&mut map, data).err())
            .collect::<Vec<_>>();
        errors.extend(merge_errors);

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(mustache::Data::Map(map))
    }
}

impl ArgVar {
    fn deep_merge(
        map: &mut HashMap<String, mustache::Data>,
        data: HashMap<String, mustache::Data>,
    ) -> Result<(), ArgVarError> {
        ArgVar::deep_merge_impl(map, data, &Vec::new())
    }

    fn deep_merge_impl(
        map: &mut HashMap<String, mustache::Data>,
        data: HashMap<String, mustache::Data>,
        ctx: &[&str],
    ) -> Result<(), ArgVarError> {
        use std::collections::hash_map::Entry;

        for (key, value) in data {
            match map.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert(value);
                }
                Entry::Occupied(mut entry) => {
                    let key = entry.key().clone();
                    let entry_value = entry.get_mut();
                    match (entry_value, value) {
                        (mustache::Data::Map(inner_map), mustache::Data::Map(inner_value)) => {
                            let mut ctx = Vec::from(ctx);
                            ctx.push(&key);
                            ArgVar::deep_merge_impl(inner_map, inner_value, &ctx)?;
                        }
                        (mustache::Data::Map(inner_map), value) => {
                            dbg!(&ctx);
                            dbg!(&inner_map);
                            dbg!(&value);
                            return Err(ArgVarError::Merge);
                        }
                        (entry_value, mustache::Data::Map(inner_value)) => {
                            dbg!(&ctx);
                            dbg!(&entry_value);
                            dbg!(&inner_value);
                            return Err(ArgVarError::Merge);
                        }
                        (entry_value, value) => {
                            dbg!(&ctx);
                            dbg!(&entry_value);
                            dbg!(&value);
                            return Err(ArgVarError::Merge);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ArgVarError {
    #[error("{0:?}: empty key (variable must be in the form `key=value`)")]
    EmptyKey(String),
    #[error("{0:?}: empty value (variable must be in the form `key=value`)")]
    EmptyValue(String),
    #[error("{0:?}: too many `=` (variable must be in the form `key=value`)")]
    TooManyEquals(String),
    #[error("{0:?}: missing `=` (variable must be in the form `key=value`)")]
    NoEquals(String),
    #[error("missing (variable must be in the form `key=value`)")]
    NoContent,
    #[error("encountered errors when merging")]
    Merge,
}

#[allow(clippy::manual_try_fold)]
pub fn arg_vars_to_data<I>(iter: I, opt_name: &str) -> eyre::Result<mustache::Data>
where
    I: IntoIterator<Item = ArgVar>,
{
    use color_eyre::Section;
    use eyre::eyre;

    iter.into_iter().collect::<Result<_, _>>().or_else(|errs| {
        errs.into_iter().fold(
            Err(eyre!("encountered errors parsing --{opt_name}")),
            |report, e| report.error(e),
        )
    })
}
