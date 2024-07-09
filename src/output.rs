/// Structured output for commands.
///
/// When implemented, users of a CLI can choose what type of structured output
/// they would like from the CLI. JSON, YAML and pretty are currently supported.
/// This can be added as part of a root command and then any subcommands are
/// able to output correctly.
///
/// Any type being output is required to implement [`serde::Serialize`] in
/// addition to [`tabled::Tabled`]. `Tabled` requires that every field
/// implements `Display`. The [`cata::output::tabled`] module provides some
/// helpers.
///
/// # Examples
///
/// ```
/// use cata::{Command, output::Format};
///
/// #[derive(serde::Serialize, tabled::Tabled)]
/// struct MyType {
///    field: String,
/// }
///
/// #[derive(clap::Parser, cata::Container)]
/// struct Cmd {
///   #[arg(short, long, value_enum)]
///   output: Format,
/// }
///
/// #[async_trait::async_trait]
/// impl Command for Cmd {
///   async fn run(&self) -> eyre::Result<()> {
///     self.output.item(&MyType { field: "value".into() })
///   }
/// }
/// ```
///
/// For a more complete example, see [examples/output].
///
/// [`serde::Serialize`]: serde::Serialize
/// [`tabled::Tabled`]: tabled::Tabled
/// [`cata::output::tabled`]: cata::output::tabled
/// [examples/output]: ../examples/output
pub mod tabled;

use ::tabled::{Table, Tabled};
use clap::ValueEnum;
use eyre::Result;
use serde::Serialize;

/// Argument for specifying the output format of structured data.
///
/// See the module documentation for usage.
#[derive(ValueEnum, Debug, Default, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Format {
    #[default]
    Pretty,
    Json,
    Yaml,
}

impl Format {
    /// Print a list of items to the console.
    pub fn list(&self, data: &[impl Serialize + Tabled]) -> Result<()> {
        match self {
            Format::Pretty => println!("{}", Table::new(data)),
            Format::Json => println!("{}", serde_json::to_string_pretty(&data)?),
            Format::Yaml => println!("{}", serde_yaml::to_string(&data)?),
        }

        Ok(())
    }

    /// Print a single item to the console.
    ///
    /// This allows format implementations to produce different outputs
    /// depending based on the number of items.
    pub fn item(&self, data: &(impl Serialize + Tabled)) -> Result<()> {
        match self {
            Format::Pretty => self.list(&[data])?,
            Format::Json => println!("{}", serde_json::to_string_pretty(data)?),
            Format::Yaml => println!("{}", serde_yaml::to_string(data)?),
        }

        Ok(())
    }
}
