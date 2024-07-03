use clap::{ArgAction, Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use eyre::Result;
use tracing_error::ErrorLayer;
use tracing_log::AsTrace;
use tracing_subscriber::{filter::EnvFilter, prelude::*};

use crate::{
    cli::{content, extractor, graph, index, namespace},
    client,
    command::Command,
    derive::Command,
    output, telemetry,
    telemetry::posthog::Posthog,
};

static PH_KEY: Option<&str> = option_env!("POSTHOG_API_KEY");

/// Interact with the indexify service
#[derive(Debug, Parser, Command)]
#[command(name = "indexify")]
pub struct Root {
    #[command(subcommand)]
    pub cmd: RootCmd,

    // TODO: can I pin this and get rid of the clone() calls?
    /// URL of the indexify service
    #[arg(
        long,
        global = true,
        default_value = client::DEFAULT_SERVICE_URL,
        env = "INDEXIFY_API_SERVER",
        requires = "namespace"
    )]
    pub api_server: client::Client,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = output::Format::Pretty, global = true)]
    pub output: output::Format,

    /// Verbosity level, pass extra v's to increase verbosity
    #[command(flatten)]
    verbosity: Verbosity,

    /// Namespace containing the resources
    #[arg(
        short,
        long,
        global = true,
        default_value = "default",
        env = "INDEXIFY_NAMESPACE"
    )]
    pub namespace: String,

    /// Enable or disable telemetry
    #[arg(
        long,
        global = true,
        env = "INDEXIFY_TELEMETRY",
        default_missing_value("true"),
        default_value("true"),
        num_args(0..=1),
        require_equals(true),
        action = ArgAction::Set,
    )]
    pub telemetry: bool,
}

#[derive(Debug, Subcommand, Command)]
pub enum RootCmd {
    Content(content::Content),
    Extractor(extractor::Extractor),
    Graph(graph::Graph),
    Index(index::Index),
    Namespace(namespace::Namespace),
}

impl Command for Root {
    fn pre_run(&self) -> Result<()> {
        let ph = Posthog::new(PH_KEY.unwrap_or("unimplemented"));

        let filter = EnvFilter::builder()
            .with_default_directive(self.verbosity.log_level_filter().as_trace().into())
            .from_env_lossy();

        // TODO: figure out how to make with_span_events(FmtSpan::CLOSE) be configurable
        let fmt = tracing_subscriber::fmt::layer()
            .pretty()
            .with_writer(std::io::stderr)
            .with_filter(filter);

        let registry = tracing_subscriber::registry()
            .with(fmt)
            .with(ErrorLayer::default());

        if self.telemetry {
            registry
                .with(telemetry::Telemetry::new(ph).with_activity().with_errors())
                .init();
        } else {
            registry.init();
        }

        Ok(())
    }
}
