use clap::{
    builder::{TypedValueParser, ValueParserFactory},
    error::ErrorKind,
};
use eyre::{eyre, Result, WrapErr};
use serde::de::DeserializeOwned;

#[derive(Debug, Clone)]
pub enum File<T> {
    None,
    Some(T),
}

impl<T> Default for File<T>
where
    T: Sync,
{
    fn default() -> Self {
        Self::None
    }
}

impl<T> ValueParserFactory for File<T>
where
    T: Sync,
{
    type Parser = File<T>;

    fn value_parser() -> Self {
        Self::default()
    }
}

impl<T> TypedValueParser for File<T>
where
    T: DeserializeOwned + Sync + Send + Clone + 'static,
{
    type Value = File<T>;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        _: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let path = std::path::PathBuf::from(value);
        let raw = std::fs::read_to_string(&path)?;

        // TODO: convert to first_or_octet_stream()
        let content: Result<T> = match mime_guess::from_path(path.clone()).first() {
            Some(mime) => match mime.subtype().as_str() {
                "x-yaml" => {
                    serde_path_to_error::deserialize(serde_yaml::Deserializer::from_str(&raw))
                        .wrap_err("Invalid YAML")
                }
                "json" => {
                    serde_path_to_error::deserialize(&mut serde_json::Deserializer::from_str(&raw))
                        .wrap_err("Invalid JSON")
                }
                unsupported => Err(eyre!("Unsupported file type: {}", unsupported)),
            },
            None => Err(eyre!(
                "MIME type not detected for {}",
                path.to_string_lossy()
            )),
        };

        content
            .map(File::Some)
            .map_err(|e| cmd.clone().error(ErrorKind::InvalidValue, format!("{}", e)))
    }
}
