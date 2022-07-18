use wildland_corex::{MnemonicPhrase, WildlandUser};
use crate::AdminManagerResult;

#[derive(Debug, Clone)]
pub struct CreateUserResponse(MnemonicPhrase);

pub trait UserApi {
    /// Create a user from randomly generated bits
    fn create_random_user(&self) -> AdminManagerResult<WildlandUser>;

    /// Create a user from provided entropy (it could be Ethereum signature or any random bits)
    fn create_user_from_entropy(&self, entropy: &[u8]) -> AdminManagerResult<()>;

    /// Create a user from mnemonic
    fn create_user_from_mnemonic(&self, mnemonic: MnemonicPhrase) -> AdminManagerResult<()>;
}