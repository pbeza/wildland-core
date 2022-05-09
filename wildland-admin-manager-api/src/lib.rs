#![allow(dead_code)]

mod api {
    use std::rc::Rc;

    pub trait Container {
        fn set_paths(&mut self, paths: Vec<String>);
        fn add_path(&mut self, path: String);
        fn get_paths(&self) -> Vec<String>;
        fn set_owner(&mut self, owner: String);
        fn get_owner(&self) -> String;
    }

    pub trait Forest<C: Container> {
        fn add_container(&mut self, container: Rc<C>);
        fn get_containers(&self) -> Vec<Rc<C>>;
        fn set_owner(&mut self, owner: String); // master's fingerprint
        fn get_owner(&self) -> String;
        fn set_devices(&mut self, fingerprints: Vec<String>); // list of fingerprints
        fn get_devices(&self) -> Vec<String>;
    }

    pub trait SeedPhrase {
        fn set_words(&mut self, words: Vec<String>);
        fn get_words(&self) -> Vec<String>;
    }

    pub trait Identity {
        fn set_identity(&mut self, identity: IdentityType);
        fn get_identity(&self) -> IdentityType;
        fn set_name(&mut self, name: String);
        fn get_name(&self) -> String;
        fn set_pubkey(&mut self, pubkey: Vec<u8>);
        fn get_pubkey(&self);
        fn set_fingerprint(&self, fingerprint: Vec<u8>);
        fn get_fingerprint(&self) -> Vec<u8>;
    }

    pub trait Credentials {}

    pub enum WalletType {
        MetaMask,
        Ledger,
        Tresor,
        Yubi,
    }

    pub enum IdentityType {
        Master,
        Device,
    }

    pub trait AdminManagerApi<C: Container, S: SeedPhrase, I: Identity, CR: Credentials> {
        /// The method generates the seed phrase required to generate the master private key. Also
        /// based on the seed phrase the UI will present to the user the 12 words (the 12 words
        /// allow to restore the master key i.e. on other devices).
        fn generate_seed_phrase() -> S;

        /// Creates the master identity based on the provided seed phrase (whether it's a newly
        /// generated seed phrase or manually entered in the recovery flow. The keys (ie. public
        /// private keypair) are stored in the Wallet component.
        fn create_master_identity_from_seed_phrase(seed: S) -> I;

        /// Similar method to: `create_master_identity_from_seed_phrase`, used to generate a keypair
        /// using an external wallet (e.g. Metamask, Ledger, etc.)
        fn create_master_identity_from_external_wallet(wallet_type: WalletType) -> I;

        /// Creates and stores in the Wallet component the device key pair.
        fn generate_device_key_pair(device_name: String) -> I;

        /// (probably an optional method for the public API) Fetches credentials to access the
        /// Foundation's catalog. Possibly this method can be wrapped along with the
        /// `fetch/create_forest` method by a higher level method. The credentials are stored
        /// internally by the AdminManager. The authorization would involve signing a challenge.
        fn _fetch_catalog_credentials(master: I) -> CR;

        /// The method attempts to retrieve the Forest manifest from the local /etc/resolv.conf. If
        /// not found locally, it should try to acquire the manifest from the catalog through the
        /// Active Catalog Backend client. The credentials for ACB should be determined using
        /// internal _fetch_catalog_credentials method or passed as an argument to this method
        /// (TBD).
        fn fetch_forest(master: I, credentials: Option<CR>) -> dyn Forest<C>;

        /// The assumption is that Cargo obtains catalog credentials from the Foundation backend
        /// (Forest Directory Service). It seems that in Cargo we assume that the user can have only
        /// one forest.
        fn create_forest(master: I, name: String, credentials: Option<CR>) -> dyn Forest<C>;
    }
}

mod admin_manager {
    use std::rc::Rc;

    pub struct Container {
        pub paths: Vec<String>,
        pub owner: String,
    }

    impl crate::api::Container for Container {
        fn set_paths(&mut self, paths: Vec<String>) {
            self.paths = paths;
        }
        fn add_path(&mut self, path: String) {
            self.paths.push(path);
        }
        fn get_paths(&self) -> Vec<String> {
            self.paths.clone()
        }
        fn set_owner(&mut self, owner: String) {
            self.owner = owner;
        }
        fn get_owner(&self) -> String {
            self.owner.clone()
        }
    }

    pub struct Forest<C: crate::api::Container> {
        pub containers: Vec<Rc<C>>,
        pub owner: String,
        pub devices: Vec<String>,
    }

    impl crate::api::Forest<Container> for Forest<Container> {
        fn add_container(&mut self, container: Rc<Container>) {
            self.containers.push(container);
        }
        fn get_containers(&self) -> Vec<Rc<Container>> {
            self.containers.clone()
        }
        fn set_owner(&mut self, owner: String) {
            self.owner = owner;
        }
        fn get_owner(&self) -> String {
            self.owner.clone()
        }
        fn set_devices(&mut self, fingerprints: Vec<String>) {
            self.devices = fingerprints;
        }
        fn get_devices(&self) -> Vec<String> {
            self.devices.clone()
        }
    }
}
