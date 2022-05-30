use anyhow::Result;
use clap::{Parser, Subcommand};
use wildland_admin_manager::{admin_manager::AdminManager, api::AdminManager as AdminManagerApi};

#[derive(Parser, Debug)]
pub struct IdentityCliOpts {
    #[clap(subcommand)]
    pub subcommand: IdentitySubcommand,
}

#[derive(Subcommand, Debug)]
pub enum IdentitySubcommand {
    Generate {
    },
    Restore {
        #[clap(long)]
        seed: String,
    },
    List {

    }
}

impl IdentityCliOpts {
    pub fn handle_command(&self) -> Result<()> {
        let mut admin_manager = AdminManager::default();

        match &self.subcommand {
            IdentitySubcommand::Generate { } => generate_identity(&mut admin_manager)?,
            IdentitySubcommand::Restore { seed } => {
                restore_identity(seed, admin_manager)?
            },
            IdentitySubcommand::List { } => {
                todo!()
            }
        }

        Ok(())
    }
}

fn restore_identity(
    seed_phrase: &String,
    mut admin_manager: AdminManager,
) -> Result<(), anyhow::Error> {
    let seed = seed_phrase
        .split(' ')
        .map(|elem| elem.to_string())
        .collect::<Vec<_>>()
        .try_into()?;

    let identity =
        admin_manager.create_master_identity_from_seed_phrase(&seed)?;

    Ok(println!("{identity}"))
}

fn generate_identity(
    admin_manager: &mut AdminManager,
) -> Result<(), anyhow::Error> {
    let seed_phrase = AdminManager::create_seed_phrase()?;
    let identity =
        admin_manager.create_master_identity_from_seed_phrase(&seed_phrase)?;

    Ok(println!("{identity}"))
}
