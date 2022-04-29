#![allow(dead_code)]

pub mod primitives;

pub mod wildland_admin_manager_api {
    use crate::primitives::wildland_primitives::{
        Credentials, Forest, Identity, SeedPhrase, WalletType,
    };

    pub trait AdminManagerApi {
        /// The method generates the seed phrase required to generate the master private key. Also
        /// based on the seed phrase the UI will present to the user the 12 words (the 12 words
        /// allow to restore the master key i.e. on other devices).
        fn generate_seed_phrase() -> SeedPhrase;

        /// Creates the master identity based on the provided seed phrase (whether it's a newly
        /// generated seed phrase or manually entered in the recovery flow. The keys (ie. public
        /// private keypair) are stored in the Wallet component.
        fn create_master_identity_from_seed_phrase(seed: SeedPhrase) -> Identity;

        /// Similar method to: `create_master_identity_from_seed_phrase`, used to generate a keypair
        /// using an external wallet (e.g. Metamask, Ledger, etc.)
        fn create_master_identity_from_external_wallet(wallet_type: WalletType) -> Identity;

        /// Creates and stores in the Wallet component the device key pair.
        fn generate_device_key_pair(device_name: String) -> Identity;

        /// (probably an optional method for the public API) Fetches credentials to access the
        /// Foundation's catalog. Possibly this method can be wrapped along with the
        /// `fetch/create_forest` method by a higher level method. The credentials are stored
        /// internally by the AdminManager. The authorization would involve signing a challenge.
        fn _fetch_catalog_credentials(master: Identity) -> Credentials;

        /// The method attempts to retrieve the Forest manifest from the local /etc/resolv.conf. If
        /// not found locally, it should try to acquire the manifest from the catalog through the
        /// Active Catalog Backend client. The credentials for ACB should be determined using
        /// internal _fetch_catalog_credentials method or passed as an argument to this method
        /// (TBD).
        fn fetch_forest(master: Identity, credentials: Option<Credentials>) -> Forest;

        /// The assumption is that Cargo obtains catalog credentials from the Foundation backend
        /// (Forest Directory Service). It seems that in Cargo we assume that the user can have only
        /// one forest.
        fn create_forest(
            master: Identity,
            name: String,
            credentials: Option<Credentials>,
        ) -> Forest;
    }
}
