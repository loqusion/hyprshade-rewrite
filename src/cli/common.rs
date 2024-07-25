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

impl TryFrom<ArgVar> for (String, mustache::Data) {
    type Error = ArgVarError;

    fn try_from(value: ArgVar) -> Result<Self, Self::Error> {
        let ArgVar(s) = value;
        match s.split('=').collect::<Vec<_>>()[..] {
            [_, ""] => Err(ArgVarError::EmptyValue(s)),
            [key, value] => Ok((key.to_string(), mustache::Data::String(value.to_string()))),
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
        let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();

        if !errors.is_empty() {
            return Err(errors);
        }

        for (key, value) in things {
            map.insert(key, value);
        }

        Ok(mustache::Data::Map(map))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ArgVarError {
    #[error("{0:?}: empty value (variable must be in the form `key=value`)")]
    EmptyValue(String),
    #[error("{0:?}: too many `=` (variable must be in the form `key=value`)")]
    TooManyEquals(String),
    #[error("{0:?}: missing `=` (variable must be in the form `key=value`)")]
    NoEquals(String),
    #[error("missing (variable must be in the form `key=value`)")]
    NoContent,
}

#[allow(clippy::manual_try_fold)]
pub fn arg_vars_to_data<I>(iter: I, opt_name: &str) -> eyre::Result<mustache::Data>
where
    I: IntoIterator<Item = ArgVar>,
{
    use color_eyre::Section;
    use eyre::eyre;

    iter.into_iter().collect::<Result<_, _>>().or_else(|errs| {
        errs.into_iter()
            .fold(Err(eyre!("error parsing --{opt_name}")), |report, e| {
                report.error(e)
            })
    })
}
