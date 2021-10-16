//! The encryption module
use async_trait::async_trait;
use crypto::digest::Digest;
use crypto::md5::Md5;
use crypto::scrypt::{scrypt_simple, ScryptParams};
use crypto::symmetriccipher::{Decryptor, Encryptor};

use crate::{Error, Result};

/// Allowed secret character range
const ALLOWED_SECRET_CHAR_LENGTH_RANGE: std::ops::RangeInclusive<usize> = 5..=50;
/// Scrypt algorithm param `p` and `r` random value range
const SCRYPT_PARAM_RANGE: std::ops::RangeInclusive<u32> = 2..=16;

/// The structure that stores encrypted hash
#[derive(Clone, Default, Debug)]
pub struct Secret {
    raw: String,
    scrypt_hash: Option<String>,
}

impl Secret {
    /// Create a new [`Secret`] from a raw secret string slice
    ///
    /// # Errors
    ///
    /// Fails with [`Encryption`] error
    pub fn from_raw(secret: &str) -> Result<Self> {
        let mut rng = rand::thread_rng();
        let (log_n, r, p) = (
            2,
            rand::Rng::gen_range(&mut rng, SCRYPT_PARAM_RANGE),
            rand::Rng::gen_range(&mut rng, SCRYPT_PARAM_RANGE),
        );
        let params = ScryptParams::new(log_n, r, p);
        let scrypt_hash = scrypt_simple(secret, &params)?;
        Ok(Self {
            raw: secret.to_owned(),
            scrypt_hash: Some(scrypt_hash),
        })
    }

    /// Returns a reference to the raw secret
    ///
    /// # Panics
    ///
    /// Will panic if attempt to get hash before hashing
    #[must_use]
    pub fn get_hash(&self) -> &str {
        let hash = self
            .scrypt_hash
            .as_ref()
            .expect("this secret has not been hashed yet!");
        hash
    }

    /// Perform needed checks, consume `Self` and return.
    ///
    /// # Errors
    ///
    /// Fails with [`Encryption`] error
    pub async fn check_consume(self) -> Result<Self> {
        <Self as Check>::length(&self).await?;
        Ok(self)
    }
}

#[async_trait]
trait Check {
    /// Check for allowed secret length
    async fn length(&self) -> Result<()>;
}

#[async_trait]
impl Check for Secret {
    async fn length(&self) -> Result<()> {
        if ALLOWED_SECRET_CHAR_LENGTH_RANGE.contains(&self.raw.len()) {
            Ok(())
        } else {
            Err(Error::Encryption(
                "invalid password length. MIN = 5 MAX = 50".to_owned(),
            ))
        }
    }
}

#[derive(Clone, Debug)]
pub struct Hasher<T: Digest + Send + Sync>(T);

impl<T: Digest + Send + Sync> Hasher<T> {
    /// Creates a new Hasher, default algorithm = [`Md5`]
    pub fn new(hasher: T) -> Self {
        Self(hasher)
    }

    pub fn digest_buf(&mut self, buf: impl AsRef<[u8]>) {
        Digest::input(&mut self.0, buf.as_ref());
    }

    pub fn digest_str(&mut self, string: impl AsRef<str>) {
        Digest::input_str(&mut self.0, string.as_ref());
    }

    pub fn consume(self) -> T {
        self.0
    }
}

impl Default for Hasher<Md5> {
    fn default() -> Self {
        let md5 = Md5::new();
        Self(md5)
    }
}

pub async fn symmetric_cipher_encrypt() {
    todo!()
}
