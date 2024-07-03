use std::{fmt::Write, path::PathBuf};

use clap::{Parser, Subcommand};
use eyre::Result;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use tokio::{fs::File, io::AsyncRead};
use tracing::{info, instrument};

use crate::{api, client, command::Command, derive::Command, output};

/// Work with content, such as downloading it or examining its metadata
#[derive(Debug, Parser, Command)]
pub struct Content {
    #[command(subcommand)]
    pub cmd: ContentCmd,
}

#[derive(Debug, Subcommand, Command)]
pub enum ContentCmd {
    Delete(Delete),
    Download(Download),
    Get(Get),
    List(List),
    Upload(Upload),
}

impl Command for Content {}

/// Delete a piece of content
#[derive(Debug, Parser, Command)]
pub struct Delete {
    #[clap(from_global)]
    pub api_server: client::Client,

    #[clap(from_global)]
    pub output: output::Format,

    #[clap(from_global)]
    pub namespace: String,

    /// ID of the content
    pub id: String,
}

#[async_trait::async_trait]
impl Command for Delete {
    #[instrument]
    async fn run(&self) -> Result<()> {
        self.api_server
            .clone()
            .with_namespace(&self.namespace)
            .delete(client::types::ContentIds::new(vec![self.id.clone()]))
            .await?;
        Ok(())
    }
}

/// Download a piece of content locally
#[derive(Debug, Parser, Command)]
pub struct Download {
    #[clap(from_global)]
    pub api_server: client::Client,

    #[clap(from_global)]
    pub output: output::Format,

    #[clap(from_global)]
    pub namespace: String,

    /// ID of the content
    pub id: String,

    /// Path to save the content to
    #[clap(short, long)]
    pub file: Option<PathBuf>,
}

impl Download {
    async fn to_file<W>(&self, path: &PathBuf, mut reader: W) -> Result<()>
    where
        W: AsyncRead + std::marker::Unpin,
    {
        let mut writer = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .await?;

        tokio::io::copy(&mut reader, &mut writer).await?;

        Ok(())
    }

    async fn to_stdout<W>(&self, mut reader: W) -> Result<()>
    where
        W: AsyncRead + std::marker::Unpin,
    {
        let mut writer = tokio::io::stdout();

        tokio::io::copy(&mut reader, &mut writer).await?;

        Ok(())
    }

    fn progress_bar(&self, size: u64) -> Result<ProgressBar> {
        let pb = ProgressBar::new(size);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] \
                     {bytes}/{total_bytes} ({eta})",
                )?
                .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
                    write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
                })
                .progress_chars("#>-"),
        );

        Ok(pb)
    }

    fn spinner(&self) -> Result<ProgressBar> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] {bytes}")?,
        );

        Ok(pb)
    }
}

#[async_trait::async_trait]
impl Command for Download {
    #[allow(clippy::blocks_in_conditions)]
    // #[instrument(err, "content::download", target = "telemetry")]
    #[instrument(err, fields(activity = "content::download"))]
    async fn run(&self) -> Result<()> {
        let (size, reader) = self
            .api_server
            .clone()
            .with_namespace(&self.namespace)
            .get_stream::<client::types::Download>(Some(&self.id))
            .await?;

        let progress = match size {
            Some(size) => self.progress_bar(size)?,
            None => self.spinner()?,
        };

        match &self.file {
            Some(path) => self.to_file(path, progress.wrap_async_read(reader)).await,
            None => self.to_stdout(reader).await,
        }
    }
}

/// Get the details of a piece of content
#[derive(Debug, Parser, Command)]
pub struct Get {
    #[clap(from_global)]
    pub api_server: client::Client,

    #[clap(from_global)]
    pub output: output::Format,

    #[clap(from_global)]
    pub namespace: String,

    /// ID of the content
    pub id: String,
}

#[async_trait::async_trait]
impl Command for Get {
    #[instrument]
    async fn run(&self) -> Result<()> {
        let content: api::ContentMetadata = self
            .api_server
            .clone()
            .with_namespace(&self.namespace)
            .get(&self.id)
            .await?;

        self.output.item(&content)
    }
}

/// List all the content in a namespace
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
        let content: Vec<api::ContentMetadata> = self
            .api_server
            .clone()
            .with_namespace(&self.namespace)
            .list()
            .await?;

        self.output.list(&content)
    }
}

/// Upload a piece of content
#[derive(Debug, Parser, Command)]
pub struct Upload {
    #[clap(from_global)]
    pub api_server: client::Client,

    #[clap(from_global)]
    pub output: output::Format,

    #[clap(from_global)]
    pub namespace: String,

    /// Path to get the content from
    pub path: clio::InputPath,

    /// Name of the graph the content is associated with
    #[clap(short, long, required = true, num_args(1..))]
    pub graph: Vec<String>,
}

#[async_trait::async_trait]
impl Command for Upload {
    #[instrument]
    async fn run(&self) -> Result<()> {
        info!("names: {:?}", self.graph);
        let content = client::types::ContentUpload::new(self.path.path().path())
            .with_graph_names(&self.graph);

        self.api_server
            .clone()
            .with_namespace(&self.namespace)
            .upload(content)
            .await?;

        Ok(())
    }
}
