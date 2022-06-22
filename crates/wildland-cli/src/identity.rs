use anyhow::Result;
use clap::{Parser, Subcommand};
use wildland_admin_manager::{admin_manager::AdminManager, api::AdminManagerApi};
use wildland_corex::{file_wallet_factory, SeedPhrase, SeedPhraseWords};
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
        let mut admin_manager = AdminManager::with_wallet_factory(&file_wallet_factory);

        match &self.subcommand {
            IdentitySubcommand::Generate { name } => generate_identity(&mut admin_manager, name)?,
            IdentitySubcommand::Restore { seed, name } => {
                restore_identity(admin_manager, seed, name)?
            }
            IdentitySubcommand::List => {
                // TODO
                // let ids = wallet.lock().unwrap().list_secrets()?;

                // match ids.len() {
                //     0 => {
                //         println!("❌ No identities found");
                //     }
                //     1 => {
                //         println!("🔑 Found 1 identity");
                //         print_identities(&ids);
                //     }
                //     _ => {
                //         println!("🔑 Found {} identities", ids.len());
                //         print_identities(&ids);
                //     }
                // }
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

    let identities = admin_manager.create_wildland_identities(&seed, name.to_string())?;

    println!(
        "🎉 New {:?} identity has been restored and securely stored.",
        Paint::blue(identities.forest_id.lock().unwrap().get_type()).bold()
    );

    println!(
        "🎉 New {:?} identity has been restored and securely stored.",
        Paint::blue(identities.device_id.lock().unwrap().get_type()).bold()
    );

    Ok(())
}

fn generate_identity(admin_manager: &mut AdminManager, name: &str) -> Result<(), anyhow::Error> {
    let seed = admin_manager.create_seed_phrase()?;
    let identities = admin_manager.create_wildland_identities(&seed, name.to_string())?;

    let forest_id = identities.forest_id.lock().unwrap();
    let device_id = identities.device_id.lock().unwrap();

    println!(
        "🎉 New {:?} identity {} has been created and securely stored.",
        Paint::blue(forest_id.get_type()).bold(),
        Paint::yellow(format!("0x{}", forest_id.get_fingerprint_string())).bold(),
    );

    println!(
        "🎉 New {:?} identity {} has been created and securely stored.",
        Paint::blue(device_id.get_type()).bold(),
        Paint::yellow(format!("0x{}", device_id.get_fingerprint_string())).bold(),
    );

    print_seedphrase(&seed);

    Ok(())
}

// TODO REMOVE OR WHAT?
// fn print_identities(ids: &[ManifestSigningKeypair]) {
//     ids.iter().for_each(|kp| {
//         println!();
//         println!("\tType: {:?}", Paint::blue(kp.get_key_type()).bold());
//         println!("\tFingerprint: {}", kp.fingerprint());
//     })
// }

fn print_seedphrase(seed_phrase: &SeedPhrase) {
    let words: SeedPhraseWords = seed_phrase.into();

    let string_repr = (1..=12)
        .zip(words.iter())
        .fold(String::new(), |mut acc, (idx, word)| {
            if idx % 4 == 1 {
                // add new line and starting tab
                acc += "\n\t";
            }
            acc += &format!("{idx: >2}. {word: <8}");
            acc
        });

    println!("🔑 Seed phrase (write it down) {string_repr}");
}
