use clap::{AppSettings, Parser};
use wildland_admin_manager::{
    admin_manager::{AdminManager, Identity},
    api::AdminManager as ApiAdminManager,
};
use yansi::Paint;

fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn print_version() {
    println!(
        "[{}] Wildland CLI version {}",
        Paint::green("+").bold(),
        get_version()
    );
    println!(
        "[{}] Admin Manager version {}",
        Paint::green("+").bold(),
        wildland_admin_manager::get_version(),
    );

    println!("[{}] Core library version:", Paint::blue("+").bold());
    wildland_corex::get_version_verbose()
        .iter()
        .for_each(|(name, version)| {
            println!(
                "[{}] * {} version {}",
                Paint::blue("+").bold(),
                name,
                version
            )
        });
}

#[derive(clap::Subcommand)]
enum IdentitySubCommand {
    Generate,
    Restore { seed_phrase: String },
}

#[derive(clap::Subcommand)]
enum SubCommand {
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
struct CliArgs {
    #[clap(long, short = 'V')]
    version: bool,
    #[clap(subcommand)]
    sub_command_action: SubCommand,
}

fn main() {
    let cli = CliArgs::parse();

    if cli.version {
        print_version();
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
                    .split(" ")
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
