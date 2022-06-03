use anyhow::Result;
use clap::StructOpt;
use cli_args::{CliArgs, IdentitySubCommand, SubCommand};
use wildland_admin_manager::{
    admin_manager::{create_seed_phrase, AdminManager},
    api::{AdminManager as AdminManagerApi, EmailClient},
};

mod cli_args;
mod version;

struct EmailClientStub;
impl EmailClient for EmailClientStub {
    fn send(
        &self,
        _address: &str,
        _message: &str,
    ) -> wildland_admin_manager::api::AdminManagerResult<()> {
        todo!()
    }
}

fn main() -> Result<()> {
    let cli = CliArgs::parse();

    if cli.version {
        version::print_version();
    } else {
        let mut admin_manager = AdminManager::new(EmailClientStub);
        match cli.sub_command_action {
            SubCommand::Identity {
                identity_action: IdentitySubCommand::Generate,
            } => {
                let seed_phrase = create_seed_phrase()?;
                let identity = admin_manager
                    .create_master_identity_from_seed_phrase("name".into(), &seed_phrase)?;
                let identity = identity.lock().unwrap();
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
                    admin_manager.create_master_identity_from_seed_phrase("name".into(), &seed)?;
                let identity = identity.lock().unwrap();
                println!("{identity}")
            }
        }
    }

    Ok(())
}
