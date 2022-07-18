use wildland_corex::ManifestSigningKeypair;

use crate::AdminManagerResult;
pub use wildland_corex::WildlandIdentity;

pub trait AdminManagerApi {
    /// Sends a 6-digit verification code to provided email address.
    fn request_verification_email(&mut self) -> AdminManagerResult<()>;

    /// Sets new unverified email
    fn set_email(&mut self, email: String);

    /// Checks whether verification code entered by a user is the same as generated one for a set email
    /// Returns error when email is not set
    fn verify_email(&mut self, verification_code: String) -> AdminManagerResult<()>;

    /// List all keypairs stored in wallet that Admin Manager was created with
    fn list_secrets(&self) -> AdminManagerResult<Vec<ManifestSigningKeypair>>;
}
