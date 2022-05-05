use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum CorexCommonError {
    #[error("Environment variable: {0} not found")]
    EnvVarNotFountError(String),
    #[error(
        "Keys have incorrect length. Each should be 32 bytes long. Public key {0}, Secret key {1}"
    )]
    CannotCreateKeyPairError(String, String),
}
