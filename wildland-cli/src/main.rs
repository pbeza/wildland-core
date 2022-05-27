use anyhow::Result;
use clap::StructOpt;
use cli_args::{CliOpts, RootSubcommands};

mod bridge;
mod cli_args;
mod container;
mod forest;
mod identity;
mod storage;
mod version;

fn main() -> Result<()> {
    let cli = CliOpts::parse();

    match &cli.subcommand {
        RootSubcommands::Identity(opts) => opts.handle_command(),
        RootSubcommands::Forest(opts) => opts.handle_command(),
        RootSubcommands::Container(opts) => opts.handle_command(),
        RootSubcommands::Storage(opts) => opts.handle_command(),
        RootSubcommands::Bridge(opts) => opts.handle_command(),
    }?;

    Ok(())
}
