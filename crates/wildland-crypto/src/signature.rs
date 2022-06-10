use cryptoxide::ed25519::SIGNATURE_LENGTH;
use hex::ToHex;

pub fn encode_signature(signature: [u8; SIGNATURE_LENGTH]) -> String {
    signature.encode_hex::<String>()
}
