use wildland_crypto::identity::{Identity, MnemonicPhrase};
use crate::CoreXResult;

pub struct WildlandUser {
    mnemonic: MnemonicPhrase
}

impl WildlandUser {
    fn mnemonic(&self) -> &MnemonicPhrase {
        &self.mnemonic
    }
}

impl From<Identity> for WildlandUser {
    fn from(identity: Identity) -> Self {
        WildlandUser {
            mnemonic: identity.get_mnemonic_phrase()
        }
    }
}

#[derive(Debug, Clone)]
pub enum CreateUserPayload {
    Random,
    Entropy(Vec<u8>),
    Mnemonic(MnemonicPhrase)
}

pub fn create_user(payload: CreateUserPayload) -> CoreXResult<WildlandUser> {
    match payload {
        CreateUserPayload::Random => {
            let user = Identity::create_random()?.into();
            Ok(user)
        },
        CreateUserPayload::Entropy(entropy) => {
            let user = Identity::try_from(entropy.as_slice())?.into();
            Ok(user)
        }
        CreateUserPayload::Mnemonic(mnemonic) => {
            let user = Identity::try_from(mnemonic)?.into();
            Ok(user)
        }
    }
}
