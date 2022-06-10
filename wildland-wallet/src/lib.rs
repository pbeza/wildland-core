pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// TODO: Fill the enum with actual wallets
pub enum WalletType {
    DummyWallet1,
    DummyWallet2,
}
