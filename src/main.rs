#![deny(unused_crate_dependencies)]

mod api;
mod cli;
mod client;
mod command;
mod derive;
mod file;
mod output;
mod telemetry;

use clap::Parser;
use eyre::{Report, Result};
use futures::future::{BoxFuture, FutureExt};

use crate::{cli::root::Root, command::Command};

#[tokio::main]
async fn main() -> Result<(), Report> {
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .display_location_section(false)
        .install()?;

    execute(&Root::parse()).await
}

fn execute(cmd: &dyn Command) -> BoxFuture<Result<()>> {
    async move {
        cmd.pre_run()?;

        cmd.run().await?;

        if let Some(next) = cmd.next() {
            execute(next).await?;
        }

        cmd.post_run()
    }
    .boxed()
}
