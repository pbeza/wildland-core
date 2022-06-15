use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct BridgeCliOpts {
    #[clap(subcommand)]
    pub subcommand: BridgeSubcommands,
}

#[derive(Debug, Subcommand)]
pub enum BridgeSubcommands {
    Create,
    List,
}

impl BridgeCliOpts {
    pub fn handle_command(&self) -> Result<()> {
        match &self.subcommand {
            BridgeSubcommands::Create => {}
            BridgeSubcommands::List {} => {}
        }

        Ok(())
    }
}
