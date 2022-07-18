use crate::AdminManagerResult;
use wildland_corex::MnemonicPhrase;

#[derive(Debug, Clone)]
pub struct GenerateMnemonicResponse(pub MnemonicPhrase);

pub trait UserApi {
    /// Generate a random mnemonic
    fn generate_random_mnemonic(&self) -> AdminManagerResult<GenerateMnemonicResponse>;

    /// Create a user from provided entropy (it could be Ethereum signature or any random bits)
    fn create_user_from_entropy(&self, entropy: &[u8]) -> AdminManagerResult<()>;

    /// Create a user from mnemonic
    fn create_user_from_mnemonic(&self, mnemonic: MnemonicPhrase) -> AdminManagerResult<()>;
}
