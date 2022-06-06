use thiserror::Error;

// TODO unify with crate::error::CryptoError when is needed
#[derive(Error, Debug, PartialEq, Eq)]
pub enum CryptoError {
    #[error("Key has incorrect length - should be 32 bytes long. Key = {0}")]
    CannotCreateKeypairError(String),
    #[error("Cannot encrypt message {0}")]
    CannotEncryptMessageError(String),
    #[error("Cannot decrypt message from ciphertext {0}")]
    CannotDecryptMessageError(String),
}
