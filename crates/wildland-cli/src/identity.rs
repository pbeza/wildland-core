use anyhow::Result;
use clap::{Parser, Subcommand};

use wildland_admin_manager::{admin_manager::AdminManager, api::AdminManagerApi};
use wildland_corex::create_file_wallet;

#[derive(Parser, Debug)]
pub struct IdentityCliOpts {
    #[clap(subcommand)]
    pub subcommand: IdentitySubcommand,
}

#[derive(Subcommand, Debug)]
pub enum IdentitySubcommand {
    Generate {},
    Restore {},
    List,
}

impl IdentityCliOpts {
    pub fn handle_command(&self) -> Result<()> {
        let admin_manager = AdminManager::with_wallet(create_file_wallet().unwrap());

        match &self.subcommand {
            IdentitySubcommand::Generate {} => {}
            IdentitySubcommand::Restore {} => {}
            IdentitySubcommand::List => {
                let ids = admin_manager.list_secrets().unwrap();

                match ids.len() {
                    0 => {
                        println!("❌ No identities found");
                    }
                    1 => {
                        println!("🔑 Found 1 identity");
                    }
                    _ => {
                        println!("🔑 Found {} identities", ids.len());
                    }
                }
            }
        }

        Ok(())
    }
}
