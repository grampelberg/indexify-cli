use clap::{Parser, Subcommand};
use eyre::{eyre, Result};
use tracing::instrument;

use crate::{
    api::{DataNamespace, ExtractionGraph},
    client,
    command::Command,
    derive::Command,
    file::File,
    output,
};

/// Interact with extraction graphs
#[derive(Debug, Parser, Command)]
pub struct Graph {
    #[command(subcommand)]
    pub cmd: GraphCmd,
}

#[derive(Debug, Subcommand, Command)]
pub enum GraphCmd {
    Create(Create),
    Get(Get),
    List(List),
}

impl Command for Graph {}

/// Create a new graph
#[derive(Debug, Parser, Command)]
pub struct Create {
    #[clap(from_global)]
    pub api_server: client::Client,

    #[clap(from_global)]
    pub output: output::Format,

    #[clap(from_global)]
    pub namespace: String,

    /// Path to the graph file
    pub input: File<ExtractionGraph>,
}

#[async_trait::async_trait]
impl Command for Create {
    #[instrument]
    async fn run(&self) -> Result<()> {
        let mut content = match self.input.clone() {
            File::None => return Err(eyre!("No input file provided")),
            File::Some(content) => content,
        };

        if content.namespace.is_empty() {
            content.namespace.clone_from(&self.namespace);
        }

        let result = self
            .api_server
            .clone()
            .with_namespace(&self.namespace)
            .create(&content)
            .await?;

        self.output.item(&result)
    }
}

/// Get a graph by name
#[derive(Debug, Parser, Command)]
pub struct Get {
    #[clap(from_global)]
    pub api_server: client::Client,

    #[clap(from_global)]
    pub output: output::Format,

    #[clap(from_global)]
    pub namespace: String,

    /// Name of the graph
    pub name: String,
}

#[async_trait::async_trait]
impl Command for Get {
    #[instrument]
    async fn run(&self) -> Result<()> {
        let namespaces: Vec<DataNamespace> = self.api_server.list().await?;

        let ns = match namespaces.iter().find(|ns| ns.name == self.namespace) {
            Some(ns) => &ns.extraction_graphs,
            None => return Err(eyre::eyre!("Namespace not found: {}", self.namespace)),
        };

        match ns.iter().find(|g| g.name == self.name) {
            Some(g) => self.output.item(g),
            None => Err(eyre::eyre!("Graph not found: {}", self.name)),
        }
    }
}

/// List all the graphs in a namespace
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
        let namespaces: Vec<DataNamespace> = self.api_server.list().await?;

        match namespaces.iter().find(|ns| ns.name == self.namespace) {
            Some(ns) => self.output.list(&ns.extraction_graphs),
            None => Err(eyre::eyre!("Namespace not found: {}", self.namespace)),
        }
    }
}
