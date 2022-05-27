use crate::{bridge, container, forest, identity, storage};
use clap::{AppSettings, Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum RootSubcommands {
    Identity(identity::IdentityCliOpts),
    Forest(forest::ForestCliOpts),
    Container(container::ContainerCliOpts),
    Storage(storage::StorageCliOpts),
    Bridge(bridge::BridgeCliOpts),
}

#[derive(Parser)]
#[clap(
    about,
    // arg_required_else_help = true,
    global_setting(AppSettings::NoAutoVersion)
)]
pub struct CliOpts {
    #[clap(long)] //, short = 'V')]
    pub version: bool,

    #[clap(short = 'v')]
    pub verbose: bool,

    #[clap(subcommand)]
    pub subcommand: RootSubcommands,
}
