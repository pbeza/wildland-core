use anyhow::Result;
use clap::{Parser, Subcommand};
use wildland_admin_manager::admin_manager::AdminManager;
use wildland_admin_manager::api::AdminManagerApi;
use wildland_corex::{list_keypairs, ManifestSigningKeypair, WalletKeypair};
use wildland_corex::{SeedPhrase, SeedPhraseWords};
use yansi::Paint;

#[derive(Parser, Debug)]
pub struct IdentityCliOpts {
    #[clap(subcommand)]
    pub subcommand: IdentitySubcommand,
}

#[derive(Subcommand, Debug)]
pub enum IdentitySubcommand {
    Generate {
        #[clap(long)]
        name: String,
    },
    Restore {
        #[clap(long)]
        seed: String,
        #[clap(long)]
        name: String,
    },
    List,
}

impl IdentityCliOpts {
    pub fn handle_command(&self) -> Result<()> {
        let mut admin_manager = AdminManager::default();

        match &self.subcommand {
            IdentitySubcommand::Generate { name } => generate_identity(&mut admin_manager, name)?,
            IdentitySubcommand::Restore { seed, name } => {
                restore_identity(admin_manager, seed, name)?
            }
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
    admin_manager: AdminManager,
    seed_phrase: &str,
    name: &str,
) -> Result<(), anyhow::Error> {
    let seed = seed_phrase
        .split(' ')
        .map(|elem| elem.to_string())
        .collect::<Vec<_>>()
        .try_into()?;

    let (forest_id, device_id) =
        admin_manager.create_wildland_identities(&seed, name.to_string())?;

    println!(
        "🎉 New {:?} identity has been restored and securely stored.",
        Paint::blue(forest_id.lock().unwrap().get_identity_type()).bold()
    );

    println!(
        "🎉 New {:?} identity has been restored and securely stored.",
        Paint::blue(device_id.lock().unwrap().get_identity_type()).bold()
    );

    Ok(())
}

fn generate_identity(admin_manager: &mut AdminManager, name: &str) -> Result<(), anyhow::Error> {
    let seed = AdminManager::create_seed_phrase()?;
    let (forest_id, device_id) =
        admin_manager.create_wildland_identities(&seed, name.to_string())?;

    let forest_id = forest_id.lock().unwrap();
    let device_id = device_id.lock().unwrap();

    println!(
        "🎉 New {:?} identity {} has been created and securely stored.",
        Paint::blue(forest_id.get_identity_type()).bold(),
        Paint::yellow(format!("0x{}", forest_id.get_fingerprint_string())).bold(),
    );

    println!(
        "🎉 New {:?} identity {} has been created and securely stored.",
        Paint::blue(device_id.get_identity_type()).bold(),
        Paint::yellow(format!("0x{}", device_id.get_fingerprint_string())).bold(),
    );

    print_seedphrase(&seed);

    Ok(())
}

fn print_identities(ids: Vec<ManifestSigningKeypair>) {
    ids.into_iter().for_each(|kp| {
        println!();
        println!("\tType: {:?}", Paint::blue(kp.get_key_type()).bold());
        println!("\tFingerprint: {}", kp.fingerprint());
    })
}

fn print_seedphrase(seed_phrase: &SeedPhrase) {
    println!("🔑 Seed phrase (write it down)");

    for i in 0..3 {
        print!("\t");
        for j in 0..4 {
            let words: SeedPhraseWords = seed_phrase.into();

            print!(
                "{: >2}. {: <padding$}",
                i * 4 + j + 1,
                words[i * 4 + j],
                padding = nasty_padding(&words, j)
            );
        }
        println!();
    }
}

fn nasty_padding(s: &SeedPhraseWords, idx: usize) -> usize {
    [&s[idx], &s[4 + idx], &s[8 + idx]]
        .iter()
        .max_by_key(|p| p.len())
        .unwrap()
        .len()
        + 2
}
