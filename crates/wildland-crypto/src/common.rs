#[cfg(test)]
pub mod test_utilities {
    use salsa20::XNonce;

    pub fn generate_random_nonce() -> XNonce {
        let mut rng = rand_core::OsRng;
        crypto_box::generate_nonce(&mut rng)
    }
    pub static MNEMONIC_PHRASE: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    pub static SIGNING_PUBLIC_KEY: &str =
        "1f8ce714b6e52d7efa5d5763fe7412c345f133c9676db33949b8d4f30dc0912f";
    pub static SIGNING_SECRET_KEY: &str =
        "e02cdfa23ad7d94508108ad41410e556c5b0737e9c264d4a2304a7a45894fc57";
    pub static ENCRYPTION_PUBLIC_KEY_1: &str =
        "293ef40c8098e209c814f56ecd02de93ebaab104c0de5f563c2a5910188b1d66";
    pub static ENCRYPTION_SECRET_KEY_1: &str =
        "f8942c3ed54ed783fc6142335b88ae557b2fdf555808c276ed37d8dd5394fc57";
    pub static ENCRYPTION_PUBLIC_KEY_2: &str =
        "91c75a549f124a16ee82253b042543b39706197eeca625f89f716307b0ed9516";
    pub static ENCRYPTION_SECRET_KEY_2: &str =
        "c039f516d8f23bdf2c9ce2b9911fd8a0ef91f7bb012bd7f2695653ce5094fc57";
    pub static TIMESTAMP: &str = "1648541699814";
}
