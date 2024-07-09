use cata::{execute, Command, Container};
use clap::{Parser, Subcommand};
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    execute(&Root::parse()).await
}

#[derive(Parser, Container)]
struct Root {
    #[command(subcommand)]
    cmd: RootCmd,
}

#[derive(Subcommand, Container)]
enum RootCmd {
    Child(Child),
}

impl Command for Root {}

#[derive(Parser, Container)]
struct Child {}

#[async_trait::async_trait]
impl Command for Child {
    async fn run(&self) -> Result<()> {
        println!("Hello");

        Ok(())
    }
}
