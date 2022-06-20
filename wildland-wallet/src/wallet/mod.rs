use crate::{Wallet, WalletError};
pub mod file;

pub type WalletFactoryType = &'static dyn Fn() -> Result<Box<dyn Wallet>, WalletError>;
