use std::fmt::Debug;

use eyre::Result;

#[async_trait::async_trait]
pub trait Command: Debug + Send + Sync + Container {
    fn pre_run(&self) -> Result<()> {
        Ok(())
    }

    async fn run(&self) -> Result<()> {
        Ok(())
    }

    fn post_run(&self) -> Result<()> {
        Ok(())
    }
}

// Allows recursion through subcommands. Adding `derive::Command` to the primary
// struct and its subcommand enum will automatically implement the correct
// matching dispatch.
pub trait Container {
    fn next(&self) -> Option<&dyn Command> {
        None
    }
}
