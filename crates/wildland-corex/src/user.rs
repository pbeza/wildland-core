use crate::CoreXResult;
use std::ops::Deref;
use wildland_crypto::identity::{generate_random_mnemonic_phrase, Identity, MnemonicPhrase};

#[derive(Debug, Clone)]
pub enum CreateUserPayload {
    Entropy(Vec<u8>),
    Mnemonic(Box<MnemonicPhrase>),
}

pub fn generate_random_mnemonic() -> CoreXResult<MnemonicPhrase> {
    let mnemonic = generate_random_mnemonic_phrase()?;
    Ok(mnemonic)
}

pub fn create_user(payload: CreateUserPayload) -> CoreXResult<()> {
    // TODO check if user already exists
    match payload {
        CreateUserPayload::Entropy(entropy) => {
            Identity::try_from(entropy.as_slice())?;
            // TODO derive forest and device id and store it in LSS
            Ok(())
        }
        CreateUserPayload::Mnemonic(mnemonic) => {
            Identity::try_from(mnemonic.deref())?;
            // TODO derive forest and device id and store it in LSS
            Ok(())
        }
    }
}
