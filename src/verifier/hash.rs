use sha2::{Digest, Sha256};

pub fn sha256_vec(input: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}

pub fn sha256_hex(input: &[u8]) -> String {
    hex::encode(sha256_vec(input))
}
