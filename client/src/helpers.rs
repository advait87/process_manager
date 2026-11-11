use rand::Rng;
use sha2::{ Digest, Sha256 };

pub fn get_random_sha256() -> [u8; 32] {
    let mut bytes = [0u8; 32];
    rand::fill(&mut bytes);

    Sha256::digest(bytes).into()
}
