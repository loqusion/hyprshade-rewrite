use std::{collections::HashMap, str::FromStr};

use serde::{ser, Deserialize, Serialize};

pub trait MergeDeep<A> {
    fn merge_deep<T: IntoIterator<Item = A>>(&mut self, iter: T, force: bool);
    #[allow(dead_code)]
    fn merge_deep_keep<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        self.merge_deep(iter, false)
    }
    #[allow(dead_code)]
    fn merge_deep_force<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        self.merge_deep(iter, true)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct TemplateDataMap(HashMap<String, TemplateData>);

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TemplateData {
    #[serde(serialize_with = "TemplateData::serialize_enum")]
    Enum(String),
    Float(f64),
    Map(HashMap<String, TemplateData>),
}

#[derive(Debug, thiserror::Error)]
#[error("failed to parse cli argument")]
pub struct TemplateDataCliParseError;

impl TemplateDataMap {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MergeDeep<(String, TemplateData)> for HashMap<String, TemplateData> {
    fn merge_deep<T: IntoIterator<Item = (String, TemplateData)>>(&mut self, iter: T, force: bool) {
        use std::collections::hash_map::Entry::*;

        let iter = iter.into_iter();
        let reserve = if self.is_empty() {
            iter.size_hint().0
        } else {
            iter.size_hint().0.saturating_add(1) / 2
        };
        self.reserve(reserve);

        iter.for_each(move |(k, v)| match self.entry(k) {
            Vacant(entry) => {
                entry.insert(v);
            }
            Occupied(entry) => {
                let value = entry.into_mut();
                match (value, v) {
                    (TemplateData::Map(inner_value), TemplateData::Map(inner_v)) => {
                        inner_value.merge_deep(inner_v, force);
                    }
                    (value, v) => {
                        if force {
                            *value = v;
                        }
                    }
                }
            }
        });
    }
}

impl MergeDeep<(String, TemplateData)> for TemplateDataMap {
    fn merge_deep<T: IntoIterator<Item = (String, TemplateData)>>(&mut self, iter: T, force: bool) {
        self.0.merge_deep(iter, force);
    }
}

impl FromIterator<(String, TemplateData)> for TemplateDataMap {
    fn from_iter<T: IntoIterator<Item = (String, TemplateData)>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl IntoIterator for TemplateDataMap {
    type Item = (String, TemplateData);
    type IntoIter = <HashMap<String, TemplateData> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> From<T> for TemplateDataMap
where
    T: Into<HashMap<String, TemplateData>>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl TemplateData {
    pub fn from_cli_arg(value: &str) -> Result<Self, TemplateDataCliParseError> {
        value.parse::<TemplateDataFromCliArg>().map(|v| v.value)
    }

    fn serialize_enum<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: AsRef<str>,
        S: ser::Serializer,
    {
        serializer.serialize_str(&value.as_ref().to_ascii_uppercase().replace(['-', '_'], ""))
    }
}

impl FromIterator<(String, TemplateData)> for TemplateData {
    fn from_iter<T: IntoIterator<Item = (String, TemplateData)>>(iter: T) -> Self {
        Self::Map(iter.into_iter().collect())
    }
}

impl<const N: usize> From<[(String, TemplateData); N]> for TemplateData {
    fn from(value: [(String, TemplateData); N]) -> Self {
        Self::Map(value.into())
    }
}

impl From<TemplateDataMap> for TemplateData {
    fn from(value: TemplateDataMap) -> Self {
        Self::Map(value.0)
    }
}

struct TemplateDataFromCliArg {
    pub value: TemplateData,
}

impl FromStr for TemplateDataFromCliArg {
    type Err = TemplateDataCliParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<f64>()
            .map(TemplateData::Float)
            .or_else(|_| Ok(TemplateData::Enum(s.to_owned())))
            .map(|value| Self { value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_from_toml() {
        #[derive(Debug, Deserialize)]
        struct Document {
            data: TemplateDataMap,
        }

        let Document { data } = toml::from_str(
            r#"
                [data]
                enum = "red-green"
                strength = 0.5
                balance = { red = 100.1, green = 253.0, blue = 93.4 }
            "#,
        )
        .unwrap();

        assert_eq!(
            data,
            TemplateDataMap::from([
                ("enum".into(), TemplateData::Enum("red-green".into())),
                ("strength".into(), TemplateData::Float(0.5)),
                (
                    "balance".into(),
                    TemplateData::from([
                        ("red".into(), TemplateData::Float(100.1)),
                        ("green".into(), TemplateData::Float(253.0)),
                        ("blue".into(), TemplateData::Float(93.4)),
                    ])
                ),
            ])
        );
    }

    #[test]
    fn compile() {
        let template = mustache::compile_str(
            "({{balance.red}}, {{balance.green}}, {{balance.blue}}) with strength={{strength}} and variant={{variant}}",
        )
        .unwrap();
        let data = TemplateDataMap::from([
            (
                "balance".into(),
                TemplateData::from([
                    ("red".into(), TemplateData::Float(100.1)),
                    ("green".into(), TemplateData::Float(253.0)),
                    ("blue".into(), TemplateData::Float(93.4)),
                ]),
            ),
            ("strength".into(), TemplateData::Float(0.5)),
            ("variant".into(), TemplateData::Enum("red-green".into())),
        ]);

        let s = template.render_to_string(&data).unwrap();
        assert_eq!(
            s,
            "(100.1, 253, 93.4) with strength=0.5 and variant=REDGREEN"
        );
    }

    #[test]
    fn merge_deep() {
        let mut data = TemplateDataMap::from([(
            String::from("balance"),
            TemplateData::from([
                (String::from("red"), TemplateData::Float(1.0)),
                (String::from("blue"), TemplateData::Float(5.0)),
            ]),
        )]);
        let other_data = TemplateDataMap::from([
            (
                String::from("balance"),
                TemplateData::from([
                    (String::from("green"), TemplateData::Float(2.0)),
                    (String::from("blue"), TemplateData::Float(3.0)),
                ]),
            ),
            (String::from("strength"), TemplateData::Float(0.15)),
        ]);

        data.merge_deep(other_data, true);

        assert_eq!(
            TemplateDataMap::from([
                (
                    String::from("balance"),
                    TemplateData::from([
                        (String::from("red"), TemplateData::Float(1.0)),
                        (String::from("green"), TemplateData::Float(2.0)),
                        (String::from("blue"), TemplateData::Float(3.0)),
                    ]),
                ),
                (String::from("strength"), TemplateData::Float(0.15)),
            ]),
            data
        );
    }
}
