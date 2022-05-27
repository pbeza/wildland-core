use anyhow::Result;
use clap::{Parser, Subcommand};
use wildland_admin_manager::admin_manager::{AdminManager, Identity};
use wildland_admin_manager_api::AdminManager as AdminManagerApi;

#[derive(Parser, Debug)]
pub struct IdentityCliOpts {
    #[clap(subcommand)]
    pub subcommand: IdentitySubcommand,
}

#[derive(Subcommand, Debug)]
pub enum IdentitySubcommand {
    Generate,
    Restore { seed_phrase: String },
}

impl IdentityCliOpts {
    pub fn handle_command(&self) -> Result<()> {
        let mut admin_manager = AdminManager::<Identity>::default();

        match &self.subcommand {
            IdentitySubcommand::Generate => generate_identity(&mut admin_manager)?,
            IdentitySubcommand::Restore { seed_phrase } => {
                restore_identity(seed_phrase, admin_manager)?
            }
        }

        Ok(())
    }
}

fn restore_identity(
    seed_phrase: &String,
    mut admin_manager: AdminManager<Identity>,
) -> Result<(), anyhow::Error> {
    let seed = seed_phrase
        .split(' ')
        .map(|elem| elem.to_string())
        .collect::<Vec<_>>()
        .try_into();

    let identity =
        admin_manager.create_master_identity_from_seed_phrase("name".into(), seed.unwrap())?;

    Ok(println!("{identity}"))
}

fn generate_identity(admin_manager: &mut AdminManager<Identity>) -> Result<(), anyhow::Error> {
    let seed_phrase = AdminManager::create_seed_phrase()?;
    let identity =
        admin_manager.create_master_identity_from_seed_phrase("name".into(), seed_phrase)?;

    Ok(println!("{identity}"))
}
