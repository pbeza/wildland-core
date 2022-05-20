mod admin_manager;
mod error;
mod identity;

pub use admin_manager::AdminManager;
pub use error::*;
pub use identity::{Identity, IdentityType};
pub use wildland_corex::{SeedPhraseWords, SEED_PHRASE_LEN};

pub type AdminManagerResult<T> = std::result::Result<T, AdminManagerError>;
