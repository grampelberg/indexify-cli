use std::fmt::Debug;

use clap::{Parser, Subcommand};
use eyre::{eyre, Result};
use tracing::{info, instrument};

use crate::{
    api::{CreateNamespace, DataNamespace},
    client,
    command::Command,
    derive::Command,
    file::File,
    output,
};

/// Interact with namespaces
#[derive(Debug, Parser, Command)]
pub struct Namespace {
    #[command(subcommand)]
    pub cmd: NamespaceCmd,
}

impl Command for Namespace {}

#[derive(Debug, Subcommand, Command)]
pub enum NamespaceCmd {
    Create(Create),
    Get(Get),
    List(List),
}

/// Create a new namespace
#[derive(Debug, Parser, Command)]
pub struct Create {
    #[clap(from_global)]
    pub api_server: client::Client,

    #[clap(from_global)]
    pub output: output::Format,

    /// Name of the namespace
    #[clap(required_unless_present = "file")]
    pub name: Option<String>,

    /// Path to a file containing the namespace definition
    #[clap(long)]
    pub file: Option<File<CreateNamespace>>,
}

#[async_trait::async_trait]
impl Command for Create {
    #[instrument]
    async fn run(&self) -> Result<()> {
        let namespace = match self.name {
            Some(ref name) => CreateNamespace {
                name: name.clone(),
                ..Default::default()
            },
            None => match self.file.clone() {
                Some(File::Some(mut content)) => {
                    content.name = self.name.clone().unwrap();

                    content
                }
                _ => return Err(eyre!("No namespace provided")),
            },
        };

        info!("Creating namespace: {:?}", namespace);

        self.api_server.create(&namespace).await?;

        Ok(())
    }
}

/// Get a specific namespace
#[derive(Debug, Parser, Command)]
pub struct Get {
    #[clap(from_global)]
    pub api_server: client::Client,

    #[clap(from_global)]
    pub output: output::Format,

    /// Name of the namespace
    pub name: String,
}

#[async_trait::async_trait]
impl Command for Get {
    #[instrument]
    async fn run(&self) -> Result<()> {
        let namespace = self.api_server.get::<DataNamespace>(&self.name).await?;

        self.output.item(&namespace)
    }
}

/// List all namespaces
#[derive(Debug, Parser, Command)]
pub struct List {
    #[clap(from_global)]
    pub api_server: client::Client,

    #[clap(from_global)]
    pub output: output::Format,
}

#[async_trait::async_trait]
impl Command for List {
    async fn run(&self) -> Result<()> {
        self.output
            .list(&self.api_server.list::<Vec<DataNamespace>>().await?)
    }
}
