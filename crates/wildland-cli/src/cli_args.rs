use crate::{bridge, container, forest, identity, storage, version};
use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum RootSubcommands {
    Identity(identity::IdentityCliOpts),
    Forest(forest::ForestCliOpts),
    Container(container::ContainerCliOpts),
    Storage(storage::StorageCliOpts),
    Bridge(bridge::BridgeCliOpts),
    Version(version::VersionCliOpts),
}

#[derive(Parser, Debug)]
#[clap(about, arg_required_else_help = false)]
pub struct CliOpts {
    #[clap(long, short = 'v')]
    pub verbose: bool,

    #[clap(subcommand)]
    pub subcommand: RootSubcommands,
}
