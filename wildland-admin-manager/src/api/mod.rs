mod admin_manager;
mod result;

pub use admin_manager::{
    AdminManager as AdminManagerApi, IdentityPair, MasterIdentity, WildlandIdentity,
};
pub use result::*;
pub use wildland_corex::{
    MasterIdentityApi, SeedPhrase, SeedPhraseWords, WildlandIdentityApi, WildlandIdentityType,
    SEED_PHRASE_LEN,
};
