use crate::CoreXError;
use wildland_crypto::identity::{self, Identity, MnemonicPhrase};

pub fn try_identity_from_mnemonic(mnemonic: &MnemonicPhrase) -> Result<Identity, CoreXError> {
    Identity::try_from(mnemonic).map_err(CoreXError::from)
}

pub fn generate_random_mnemonic() -> Result<MnemonicPhrase, CoreXError> {
    identity::generate_random_mnemonic().map_err(CoreXError::from)
}
