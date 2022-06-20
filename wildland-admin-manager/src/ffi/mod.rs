use crate::admin_manager::AdminManager;
use crate::api::{AdminManagerError, SeedPhrase};
use ffi_macro::binding_wrapper;

// Define Error type and `()` type.
type ErrorType = AdminManagerError;
type VoidType = ();

#[binding_wrapper]
mod ffi_binding {
    extern "Rust" {
        // type IdentityPair;
        type AdminManager;
        // fn create_file_wallet_admin_manager() -> Result<AdminManager<FileWallet>>;
        // fn send_verification_code(self: &mut AdminManager) -> Result<VoidType>;
        // fn set_email(self: &mut AdminManager, email: String);
        // fn verify_email(self: &mut AdminManager, verification_code: String) -> Result<VoidType>;
        // fn create_wildland_identities(
        //     self: &mut AdminManager,
        //     seed: &SeedPhrase,
        //     device_name: String,
        // ) -> Result<IdentityPair>;

        type SeedPhrase;
        fn get_string(self: &SeedPhrase) -> String;
        fn get_vec(self: &SeedPhrase) -> Vec<String>;

        // type WildlandIdentityType;
        // fn get_identity_type(self: &Arc<Mutex<dyn WildlandIdentityApi>>) -> WildlandIdentityType;
        // fn get_name(self: &Arc<Mutex<dyn WildlandIdentityApi>>) -> String;
        // fn set_name(self: &mut Arc<Mutex<dyn WildlandIdentityApi>>, name: String);
        // fn get_public_key(self: &Arc<Mutex<dyn WildlandIdentityApi>>) -> Vec<u8>;

        // type SeedPhraseWords;
        // fn get_seed_phrase(self: &Arc<Mutex<dyn MasterIdentityApi>>) -> SeedPhraseWords;

        type VoidType;
        type ErrorType;
        fn to_string(self: &ErrorType) -> String;
        fn code(self: &ErrorType) -> u32;
    }
}
