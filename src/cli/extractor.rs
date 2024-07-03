use clap::{Parser, Subcommand};
use eyre::Result;
use tracing::instrument;

use crate::{api, client, command::Command, derive::Command, output};

/// Examine and manipulate extractors
#[derive(Debug, Parser, Command)]
pub struct Extractor {
    #[command(subcommand)]
    pub cmd: ExtractorCmd,
}

#[derive(Debug, Subcommand, Command)]
pub enum ExtractorCmd {
    List(List),
}

impl Command for Extractor {}

/// List all the registered extractors
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
        let extractors: Vec<api::ExtractorDescription> = self.api_server.list().await?;

        self.output.list(&extractors)
    }
}
