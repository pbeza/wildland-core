use crate::CoreXError;
use wildland_wallet::Wallet;

type FactoryResult = Result<Box<dyn Wallet>, CoreXError>;

pub fn create_file_wallet() -> FactoryResult {
    wildland_wallet::wallet::file::create_file_wallet().map_err(CoreXError::WalletCreationError)
}
