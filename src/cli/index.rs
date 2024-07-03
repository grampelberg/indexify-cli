use clap::{Parser, Subcommand};
use eyre::Result;
use tracing::instrument;

use crate::{api, client, command::Command, derive::Command, output};

/// Examine and manipulate indexes
#[derive(Debug, Parser, Command)]
pub struct Index {
    #[command(subcommand)]
    pub cmd: IndexCmd,
}

#[derive(Debug, Subcommand, Command)]
pub enum IndexCmd {
    List(List),
}

impl Command for Index {}

/// List all the indexes in a namespace
#[derive(Debug, Parser, Command)]
pub struct List {
    #[clap(from_global)]
    pub api_server: client::Client,

    #[clap(from_global)]
    pub output: output::Format,

    #[clap(from_global)]
    pub namespace: String,
}

#[async_trait::async_trait]
impl Command for List {
    #[instrument]
    async fn run(&self) -> Result<()> {
        let indexes: Vec<api::Index> = self
            .api_server
            .clone()
            .with_namespace(&self.namespace)
            .list()
            .await?;

        self.output.list(&indexes)
    }
}
