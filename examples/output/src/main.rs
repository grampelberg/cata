use cata::{
    execute,
    output::{tabled::display, Format},
    Command, Container,
};
use clap::Parser;
use eyre::Result;
use serde::Serialize;
use tabled::Tabled;

#[tokio::main]
async fn main() -> Result<()> {
    execute(&Root::parse()).await
}

#[derive(Serialize, Tabled)]
struct Thing {
    single: String,
    #[tabled(display_with = "display")]
    multiple: Vec<String>,
}

#[derive(Parser, Container)]
struct Root {
    /// Output format
    #[arg(short, long, value_enum, default_value_t = Format::Pretty)]
    pub output: Format,
}

#[async_trait::async_trait]
impl Command for Root {
    async fn run(&self) -> Result<()> {
        let things = &[
            Thing {
                single: "single".into(),
                multiple: vec!["one".into(), "two".into()],
            },
            Thing {
                single: "another".into(),
                multiple: vec!["three".into(), "four".into()],
            },
        ];

        self.output.list(things)?;
        self.output.item(&things[0])?;

        Ok(())
    }
}
