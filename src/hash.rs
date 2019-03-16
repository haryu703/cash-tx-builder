use sha2::{Sha256, Digest};

pub fn hash256(hasher: Sha256) -> Vec<u8> {
    Sha256::digest(&hasher.result()).to_vec()
}
