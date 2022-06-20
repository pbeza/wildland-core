use clap::{AppSettings, Parser};

#[derive(clap::Subcommand)]
pub enum IdentitySubCommand {
    Generate,
    Restore { seed_phrase: String },
}

#[derive(clap::Subcommand)]
pub enum SubCommand {
    Identity {
        #[clap(subcommand)]
        identity_action: IdentitySubCommand,
    },
}

#[derive(Parser)]
#[clap(
    about,
    arg_required_else_help = true,
    global_setting(AppSettings::NoAutoVersion)
)]
pub struct CliArgs {
    #[clap(long, short = 'V')]
    pub version: bool,
    #[clap(subcommand)]
    pub sub_command_action: SubCommand,
}
