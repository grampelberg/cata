//! cata(lyst) for building complex CLI tools.
//!
//! This crate provides a collection of utilities that make it easier to build
//! CLI tools.
//!
//! - [`command`]: recursively traverse a tree of clap commands and subcommands
//!   calling lifecycle hooks at each level.
//! - [`mod@file`]: derive `clap::value_parser` for deserializing values from
//!   files. Detects the file format from the extension and currently supports
//!   JSON in addition to YAML.
//! - [`output`]: structured output for commands. Users can choose the output
//!   format they would like, currently supporting JSON, YAML and pretty.
//! - [`telemetry`]: a simple way to track activity and errors for your CLI.
pub mod command;
pub mod file;
pub mod output;
pub mod telemetry;

pub use cata_derive::{Container, File};
use eyre::Result;
use futures::future::{BoxFuture, FutureExt};

pub use crate::{command::Command, output::Format};

/// Executes a command and all of its subcommands.
///
/// Recursively calls `pre_run`, `run`, and `post_run` on the command and all of
/// its subcommands.
pub fn execute(cmd: &dyn Command) -> BoxFuture<Result<()>> {
    async move {
        cmd.pre_run()?;

        cmd.run().await?;

        if let Some(next) = cmd.next() {
            execute(next).await?;
        }

        cmd.post_run()
    }
    .boxed()
}
