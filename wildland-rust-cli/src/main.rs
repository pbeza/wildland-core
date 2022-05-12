use clap::StructOpt;
use cli_args::{CliArgs, IdentitySubCommand, SubCommand};
use wildland_admin_manager::{
    admin_manager::{AdminManager, Identity},
    api::AdminManager as ApiAdminManager,
};

mod cli_args;
mod version;

fn main() {
    let cli = CliArgs::parse();

    if cli.version {
        version::print_version();
    } else {
        let mut admin_manager = AdminManager::<Identity>::default();
        match cli.sub_command_action {
            SubCommand::Identity {
                identity_action: IdentitySubCommand::Generate,
            } => {
                let identity = admin_manager.create_master_identity("name".into());
                println!("{identity}")
            }
            SubCommand::Identity {
                identity_action: IdentitySubCommand::Restore { seed_phrase },
            } => {
                let seed: Vec<String> = seed_phrase
                    .split(' ')
                    .map(|elem| elem.to_string())
                    .collect();
                match seed.try_into() {
                    Ok(seed) => {
                        let identity = admin_manager
                            .create_master_identity_from_seed_phrase("name".into(), seed);
                        println!("{identity}")
                    }
                    Err(e) => println!("Could not parse seed phrase: {e:?}"),
                }
            }
        }
    }
}
