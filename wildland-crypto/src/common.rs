use salsa20::XNonce;

pub fn generate_random_nonce() -> XNonce {
    let mut rng = crypto_box::rand_core::OsRng;
    crypto_box::generate_nonce(&mut rng)
}
