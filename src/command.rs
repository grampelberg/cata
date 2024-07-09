use eyre::Result;

/// The base structure for commands.
///
/// A command is a single unit of work, the trait exposes hooks that allow for
/// multiple commands to cooperate in a single lifecycle. There should be an
/// implementation of this for every instance of a [`Parser`] in clap.
///
/// The default implementations are no-ops and allow for commands to implement
/// only what they need. This primarily results in parent commands implementing
/// pre/post run and child commands implementing run.
///
/// Commands are called recursively, starting at the root command and traversing
/// through all the subcommands that were successfully parsed. The `pre-run` and
/// `run` hooks are called first on the parent before recursing into the child.
/// Subsequently, `post-run` is called first on the child as it recurses up to
/// the parent.
///
/// [`Parser`]: clap::Parser
#[async_trait::async_trait]
pub trait Command: Send + Sync + Container {
    /// Performs any setup required before the command is run.
    fn pre_run(&self) -> Result<()> {
        Ok(())
    }

    /// Execution of the command.
    async fn run(&self) -> Result<()> {
        Ok(())
    }

    /// Performs any cleanup required after the command is run.
    fn post_run(&self) -> Result<()> {
        Ok(())
    }
}

/// Allows commands to optionally contain subcommands.
///
/// `cata::execute` expects commands to have implemented this trait to discover
/// if it needs to recurse into a subcommand. While it is possible to implement
/// this yourself, it is recommended that `#[derive(Command)]` is used to
/// automatically generate the code required to switch between subcommand enums.
pub trait Container {
    /// Optionally returns the next command to be run.
    fn next(&self) -> Option<&dyn Command> {
        None
    }
}
