pub mod wildland_primitives {
    #[repr(C)]
    pub enum WalletType {
        MetaMask,
        Ledger,
        Tresor,
        Yubi,
    }

    #[repr(C)]
    pub enum IdentityType {
        Master,
        Device,
    }

    #[repr(C)]
    pub struct SeedPhrase {
        words: Vec<String>,
    }

    #[repr(C)]
    pub struct Identity {
        identity_type: IdentityType,
        name: String,
        pubkey: Vec<u8>,
        fingerprint: Vec<u8>,
    }

    #[repr(C)]
    pub struct Credentials {
        // ???
    }

    #[repr(C)]
    pub struct Forest {
        name: String,
        owner: Vec<u8>, // master's fingerprint
        devices: Vec<Vec<u8>>, // list of fingerprints
                        // ???
    }
}
