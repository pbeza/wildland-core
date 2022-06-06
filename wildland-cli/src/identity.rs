use anyhow::Result;
use clap::{Parser, Subcommand};
use std::sync::{Arc, Mutex};
use wildland_admin_manager::api::Identity;
use wildland_admin_manager::{admin_manager::AdminManager, api::AdminManager as AdminManagerApi};
use wildland_corex::SeedPhraseWords;
use wildland_corex::{list_keypairs, ManifestSigningKeypair, SigningKeypair};
use yansi::Paint;

#[derive(Parser, Debug)]
pub struct IdentityCliOpts {
    #[clap(subcommand)]
    pub subcommand: IdentitySubcommand,
}

#[derive(Subcommand, Debug)]
pub enum IdentitySubcommand {
    Generate,
    Restore {
        #[clap(long)]
        seed: String,
    },
    List,
}

impl IdentityCliOpts {
    pub fn handle_command(&self) -> Result<()> {
        let mut admin_manager = AdminManager::default();

        match &self.subcommand {
            IdentitySubcommand::Generate => generate_identity(&mut admin_manager)?,
            IdentitySubcommand::Restore { seed } => restore_identity(seed, admin_manager)?,
            IdentitySubcommand::List => {
                let ids = list_keypairs(wildland_corex::WalletType::File)?;

                match ids.len() {
                    0 => {
                        println!("❌ No identities found");
                    }
                    1 => {
                        println!("🔑 Found 1 identity");
                        print_identities(ids);
                    }
                    _ => {
                        println!("🔑 Found {} identities", ids.len());
                        print_identities(ids);
                    }
                }
            }
        }

        Ok(())
    }
}

fn restore_identity(
    seed_phrase: &str,
    mut admin_manager: AdminManager,
) -> Result<(), anyhow::Error> {
    let seed = seed_phrase
        .split(' ')
        .map(|elem| elem.to_string())
        .collect::<Vec<_>>()
        .try_into()?;

    let identity = admin_manager.create_master_identity_from_seed_phrase(&seed)?;

    println!(
        "🎉 {:?} identity restored successfully.",
        Paint::blue(identity.lock().unwrap().get_identity_type()).bold()
    );

    Ok(())
}

fn generate_identity(admin_manager: &mut AdminManager) -> Result<(), anyhow::Error> {
    let seed = AdminManager::create_seed_phrase()?;
    let identity = admin_manager.create_master_identity_from_seed_phrase(&seed)?;

    println!(
        "🎉 New {:?} identity has been created and securely stored.",
        Paint::blue(identity.lock().unwrap().get_identity_type()).bold()
    );

    print_seedphrase(identity);

    Ok(())
}

fn nasty_padding(s: &SeedPhraseWords, idx: usize) -> usize {
    [&s[idx], &s[4 + idx], &s[8 + idx]]
        .iter()
        .max_by_key(|p| p.len())
        .unwrap()
        .len()
        + 2
}

fn print_identities(ids: Vec<ManifestSigningKeypair>) {
    ids.into_iter().for_each(|kp| {
        println!();
        println!("\tType: {}", Paint::blue("Master").bold());
        println!("\tFingerprint: {}", kp.fingerprint());
    })
}

fn print_seedphrase(identity: Arc<Mutex<dyn Identity>>) {
    println!("🔑 Seed phrase (write it down)");

    for i in 0..3 {
        print!("\t");
        for j in 0..4 {
            let seed_phrase = identity.lock().unwrap().get_seed_phrase();

            print!(
                "{: <padding$}",
                seed_phrase[i * 4 + j],
                padding = nasty_padding(&seed_phrase, j)
            );
        }
        println!();
    }
}
