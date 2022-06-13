use anyhow::Result;
use clap::Parser;
use serde_json::json;
use std::collections::HashMap;
use yansi::Paint;

fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[derive(Debug, Parser)]
pub struct VersionCliOpts {
    #[clap(long)]
    json: bool,
}

impl VersionCliOpts {
    pub fn handle_command(&self) -> Result<()> {
        match self.json {
            true => print_version_json(),
            false => print_version(),
        }
        Ok(())
    }
}

pub fn print_version_json() {
    let core_modules: HashMap<_, _> = HashMap::from_iter(
        wildland_corex::get_version_verbose()
            .iter()
            .map(|(name, version)| (name.to_lowercase(), *version)),
    );

    println!(
        "{}",
        json!({
            "cli": get_version(),
            "admin_manager": wildland_admin_manager::get_version(),
            "core": core_modules,
        })
    );
}

pub fn print_version() {
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
