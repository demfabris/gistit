//! Hashing

use sha2::{Digest, Sha256};

pub fn compute(data: impl AsRef<[u8]>, author: &str, description: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.update(author);
    hasher.update(description);

    format!("{:x}", hasher.finalize())
}
