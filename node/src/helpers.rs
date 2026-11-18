use rand::Rng;
use sha2::{ Digest, Sha256 };
use std::path::Path;

pub fn get_random_sha256() -> [u8; 32] {
    let mut bytes = [0u8; 32];
    rand::fill(&mut bytes);

    Sha256::digest(bytes).into()
}

pub fn get_hash_string(input: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

