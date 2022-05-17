mod admin_manager;
mod error;
mod identity;
mod seed_phrase;

pub use admin_manager::AdminManager;
pub use error::*;
pub use identity::{Identity, IdentityType};
pub use seed_phrase::{SeedPhraseWords, SEED_PHRASE_LEN};

pub type AdminManagerResult<T> = std::result::Result<T, AdminManagerError>;
