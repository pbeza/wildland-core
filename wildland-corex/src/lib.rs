pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn get_version_verbose() -> Vec<(&'static str, &'static str)> {
    vec![
        ("CoreX", env!("CARGO_PKG_VERSION")),
        ("CatLib", wildland_catlib::get_version()),
        ("Wallet", wildland_wallet::get_version()),
        ("DFS", wildland_dfs::get_version()),
    ]
}
