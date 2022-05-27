use super::SeedPhraseResult;
use crate::{admin_manager::AdminManager, api::AdminManager as AdminManagerApi};

fn swift_create_seed_phrase() -> SeedPhraseResult {
    AdminManager::create_seed_phrase().into()
}

#[swift_bridge::bridge]
mod ffi_bridge {
    extern "Rust" {
        type SeedPhraseResult;
        fn swift_create_seed_phrase() -> SeedPhraseResult;
        fn is_ok(self: &SeedPhraseResult) -> bool;
    }
}
