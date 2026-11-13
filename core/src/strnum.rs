use serde::{Deserialize, Deserializer, Serializer};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Deserialize)]
#[serde(untagged)]
enum Repr {
    Text(String),
    Number(u64),
}

pub fn serialize<T: ToString, S: Serializer>(value: &T, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&value.to_string())
}

pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let text = match Repr::deserialize(deserializer)? {
        Repr::Text(text) => text,
        Repr::Number(number) => number.to_string(),
    };
    text.parse().map_err(serde::de::Error::custom)
}

pub mod opt {
    use super::Repr;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error> {
        match value {
            Some(value) => serializer.serialize_str(&value.to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<u64>, D::Error> {
        match Option::<Repr>::deserialize(deserializer)? {
            None => Ok(None),
            Some(Repr::Number(number)) => Ok(Some(number)),
            Some(Repr::Text(text)) => text.parse().map(Some).map_err(serde::de::Error::custom),
        }
    }
}
