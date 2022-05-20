mod admin_manager;
mod error;
mod identity;

pub use admin_manager::AdminManager;
pub use error::*;
pub use identity::{Identity, IdentityType};

pub type AdminManagerResult<T> = std::result::Result<T, AdminManagerError>;

pub const SEED_PHRASE_LEN: usize = 12;
pub type SeedPhraseWords = [String; SEED_PHRASE_LEN];
