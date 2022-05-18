use yansi::Paint;

fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
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
