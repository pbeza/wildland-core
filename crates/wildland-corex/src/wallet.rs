use crate::CoreXError;
use wildland_wallet::Wallet;

type FactoryResult = Result<Box<dyn Wallet>, CoreXError>;
pub type WalletFactoryType = &'static dyn Fn() -> FactoryResult;

pub fn file_wallet_factory() -> FactoryResult {
    wildland_wallet::wallet::file::file_wallet_factory().map_err(CoreXError::WalletCreationError)
}
