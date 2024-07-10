#![warn(unused_crate_dependencies)]

pub mod command;
pub mod file;
pub mod output;

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
