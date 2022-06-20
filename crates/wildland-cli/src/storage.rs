use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct StorageCliOpts {
    #[clap(subcommand)]
    pub subcommand: StorageSubcommands,
}

#[derive(Debug, Subcommand)]
pub enum StorageSubcommands {
    Create,
    List,
}

impl StorageCliOpts {
    pub fn handle_command(&self) -> Result<()> {
        match &self.subcommand {
            StorageSubcommands::Create => {}
            StorageSubcommands::List {} => {}
        }

        Ok(())
    }
}
