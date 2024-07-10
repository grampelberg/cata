//! Consume input into a struct automatically.
//!
//! Takes a user provided path, reads the file and deserializes it into the
//! provided struct. Does file extension detection to understand the file's
//! format. Currently supports JSON and YAML.
//!
//! # Examples
//!
//! See [examples/file] for a more detailed example.
//!
//! ```
//! use cata::{Container, File};
//!
//! #[derive(clap::Parser, Container)]
//! struct Cmd {
//!   input: File<MyType>,
//! }
//!
//! #[derive(Clone, Debug, serde::Deserialize, File)]
//! struct MyType {
//!   field: String,
//! }
//!
//! #[async_trait::async_trait]
//! impl cata::Command for Cmd {
//!   async fn run(&self) -> eyre::Result<()> {
//!     println!("input: {:#?}", self.input);
//!     Ok(())
//!   }
//! }
//! ```
//!
//! [examples/file]: ../examples/file/src/main.rs
use clap::{builder::TypedValueParser, error::ErrorKind};
use eyre::{eyre, Result};
use serde::de::DeserializeOwned;

/// Implementation of `TypedValueParser` for deserializing a file into a struct.
///
/// This is not meant to be used directly, see the `File` derive macro for how
/// to use it. The `ValueParserFactory` trait is automatically generated for
/// structs using that macro and the implementation uses this implementation.
#[derive(Debug, Clone)]
pub struct File<T> {
    _p: std::marker::PhantomData<T>,
}

impl<T> Default for File<T> {
    fn default() -> Self {
        Self {
            _p: std::marker::PhantomData,
        }
    }
}

impl<T> TypedValueParser for File<T>
where
    T: DeserializeOwned + Sync + Send + Clone + 'static,
{
    type Value = T;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let path = std::path::PathBuf::from(value);
        let raw = std::fs::read_to_string(&path).map_err(|e| {
            cmd.clone().error(
                ErrorKind::InvalidValue,
                format!(
                    "Could not read file {} for {}: {}",
                    value.to_str().unwrap(),
                    arg.unwrap(),
                    e
                ),
            )
        })?;

        let content: Result<T> = match mime_guess::from_path(path.clone())
            .first_or_text_plain()
            .subtype()
            .as_str()
        {
            "x-yaml" => serde_path_to_error::deserialize(serde_yaml::Deserializer::from_str(&raw))
                .map_err(|e| eyre!(e)),
            "json" => {
                serde_path_to_error::deserialize(&mut serde_json::Deserializer::from_str(&raw))
                    .map_err(|e| eyre!(e))
            }
            unsupported => Err(eyre!("Unsupported file type: {}", unsupported)),
        };

        content.map_err(|e| {
            cmd.clone().error(
                ErrorKind::InvalidValue,
                format!(
                    "Failed to deserialize {} for {}: {}",
                    value.to_str().unwrap(),
                    arg.unwrap(),
                    e
                ),
            )
        })
    }
}
