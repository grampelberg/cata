use clap::{
    builder::{TypedValueParser, ValueParserFactory},
    error::ErrorKind,
};
use eyre::{eyre, Result, WrapErr};
use serde::de::DeserializeOwned;

/// Consume input into a struct automatically.
///
/// Takes a user provided path, reads the file and deserializes it into the
/// provided struct. Does file extension detection to understand the file's
/// format. Currently supports JSON and YAML.
///
/// # Examples
///
/// ```
/// use cata::file::File;
///
/// #[derive(clap::Parser, cata::Container)]
/// struct Cmd {
///   input: File<MyType>,
/// }
///
/// #[derive(Clone, Debug, serde::Deserialize)]
/// struct MyType {
///   field: String,
/// }
///
/// #[async_trait::async_trait]
/// impl cata::Command for Cmd {
///   async fn run(&self) -> eyre::Result<()> {
///     println!("input: {:#?}", self.input);
///     Ok(())
///   }
/// }
/// ```
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

        let content: Result<T> = match mime_guess::from_path(path.clone())
            .first_or_text_plain()
            .subtype()
            .as_str()
        {
            "x-yaml" => serde_path_to_error::deserialize(serde_yaml::Deserializer::from_str(&raw))
                .wrap_err("Invalid YAML"),
            "json" => {
                serde_path_to_error::deserialize(&mut serde_json::Deserializer::from_str(&raw))
                    .wrap_err("Invalid JSON")
            }
            unsupported => Err(eyre!("Unsupported file type: {}", unsupported)),
        };

        content
            .map(File::Some)
            .map_err(|e| cmd.clone().error(ErrorKind::InvalidValue, format!("{}", e)))
    }
}
