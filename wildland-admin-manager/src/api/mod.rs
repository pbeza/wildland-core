mod admin_manager;
mod result;
mod seed_phrase;

pub use admin_manager::AdminManager;
pub use result::*;
pub use seed_phrase::SeedPhrase;
pub use wildland_corex::{SeedPhraseWords, SEED_PHRASE_LEN};

#[derive(Clone, Copy, Debug)]
pub enum IdentityType {
    Master,
    Device,
}
