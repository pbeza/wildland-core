use anyhow::Result;
use clap::StructOpt;
use cli_args::{CliArgs, IdentitySubCommand, SubCommand};
use wildland_admin_manager::{
    admin_manager::{AdminManager, Identity},
    api::AdminManager as AdminManagerApi,
};

mod cli_args;
mod version;

fn main() -> Result<()> {
    let cli = CliArgs::parse();

    if cli.version {
        version::print_version();
    } else {
        let mut admin_manager = AdminManager::<Identity>::default();
        match cli.sub_command_action {
            SubCommand::Identity {
                identity_action: IdentitySubCommand::Generate,
            } => {
                let seed_phrase = AdminManager::create_seed_phrase()?;
                let identity = admin_manager
                    .create_master_identity_from_seed_phrase("name".into(), seed_phrase)?;
                println!("{identity}")
            }
            SubCommand::Identity {
                identity_action: IdentitySubCommand::Restore { seed_phrase },
            } => {
                let seed = seed_phrase
                    .split(' ')
                    .map(|elem| elem.to_string())
                    .collect::<Vec<_>>()
                    .try_into()?;
                let identity =
                    admin_manager.create_master_identity_from_seed_phrase("name".into(), seed)?;
                println!("{identity}")
            }
        }
    }

    Ok(())
}
