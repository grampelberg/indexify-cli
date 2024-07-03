pub mod tabled;

use ::tabled::{Table, Tabled};
use clap::ValueEnum;
use eyre::Result;
use serde::Serialize;

#[derive(ValueEnum, Debug, Default, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Format {
    #[default]
    Pretty,
    Json,
}

impl Format {
    pub fn list(&self, data: &[impl Serialize + Tabled]) -> Result<()> {
        match self {
            Format::Pretty => println!("{}", Table::new(data)),
            Format::Json => println!("{}", serde_json::to_string_pretty(&data).unwrap()),
        }

        Ok(())
    }

    // TODO: this should probably be a separate pretty implementation from
    // list(Vec<T>).
    pub fn item(&self, data: &(impl Serialize + Tabled)) -> Result<()> {
        match self {
            Format::Pretty => self.list(&[data])?,
            Format::Json => println!("{}", serde_json::to_string_pretty(data).unwrap()),
        }

        Ok(())
    }
}
