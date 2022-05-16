use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum CryptoError {
    #[error("Key has incorrect length - should be 32 bytes long. Key = {0}")]
    CannotCreateKeyPairError(String),
    #[error("Cannot encrypt message {0}")]
    CannotEncryptMessageError(String),
    #[error("Cannot decrypt message from ciphertext {0}")]
    CannotDecryptMessageError(String),
}