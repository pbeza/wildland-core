use clap::{AppSettings, Parser};
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

#[derive(Parser)]
#[clap(
    about,
    arg_required_else_help = true,
    global_setting(AppSettings::NoAutoVersion)
)]
struct CliArgs {
    #[clap(long, short = 'V')]
    version: bool,
}

fn main() {
    let cli = CliArgs::parse();

    if cli.version {
        print_version();
    }
}
