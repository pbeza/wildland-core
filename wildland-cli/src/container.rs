use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct ContainerCliOpts {
    #[clap(subcommand)]
    pub subcommand: ContainerSubcommands,
}

#[derive(Debug, Subcommand)]
pub enum ContainerSubcommands {
    Create,
    List,
}

impl ContainerCliOpts {
    pub fn handle_command(&self) -> Result<()> {
        match &self.subcommand {
            ContainerSubcommands::Create => {}
            ContainerSubcommands::List {} => {}
        }

        Ok(())
    }
}
