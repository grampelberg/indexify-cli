use std::fmt::{self, Display};

use serde::{de::Deserializer, Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct Option<T>(std::option::Option<T>);

impl<T> Display for Option<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Some(value) => write!(f, "{}", value),
            None => write!(f, ""),
        }
    }
}

impl<'de, T> Deserialize<'de> for Option<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<T> = Option::<T>::from(std::option::Option::deserialize(deserializer)?);

        Ok(opt)
    }
}

impl<T> From<std::option::Option<T>> for Option<T> {
    fn from(option: std::option::Option<T>) -> Self {
        match option {
            Some(value) => Self(Some(value)),
            None => Self(None),
        }
    }
}

// TODO: This feels like it should be a newtype for Vec<T>
pub fn display<T>(value: &[T]) -> String
where
    T: Display,
{
    let display = &mut value
        .iter()
        .map(|v| format!("{}", v))
        .collect::<Vec<String>>();

    display.sort();

    display.join("\n")
}
