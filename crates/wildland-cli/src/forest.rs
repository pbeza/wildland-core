use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct ForestCliOpts {
    #[clap(subcommand)]
    pub subcommand: ForestSubcommands,
}

#[derive(Debug, Subcommand)]
pub enum ForestSubcommands {
    Create,
    List,
}

impl ForestCliOpts {
    pub fn handle_command(&self) -> Result<()> {
        match &self.subcommand {
            ForestSubcommands::Create => todo!(),
            ForestSubcommands::List => todo!(),
        }
    }
}
