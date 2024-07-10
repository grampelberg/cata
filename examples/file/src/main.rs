use cata::{execute, Command, Container, File};
use clap::Parser;
use eyre::Result;
use serde::Deserialize;

#[tokio::main]
async fn main() -> Result<()> {
    execute(&Root::parse()).await
}

#[derive(Clone, Debug, Deserialize, File)]
struct Thing {
    single: String,
}

#[derive(Parser, Container)]
struct Root {
    input: Thing,
}

#[async_trait::async_trait]
impl Command for Root {
    async fn run(&self) -> Result<()> {
        println!("input: {:#?}", self.input);

        Ok(())
    }
}
