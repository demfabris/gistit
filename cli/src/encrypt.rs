//! The encryption module
use async_trait::async_trait;
use crypto::scrypt::{scrypt_simple, ScryptParams};

use crate::{Error, Result};

/// Allowed secret character range
const ALLOWED_SECRET_CHAR_LENGTH_RANGE: std::ops::RangeInclusive<usize> = 5..=50;
/// Scrypt algorithm param `p` and `r` random value range
const SCRYPT_PARAM_RANGE: std::ops::RangeInclusive<u32> = 0..=std::u32::MAX / 128;

/// The structure that stores encrypted hash
#[derive(Clone, Default, Debug)]
pub struct Secret {
    raw: String,
    hash: String,
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
            rand::Rng::gen_range::<u8, _>(&mut rng, 0..64),
            rand::Rng::gen_range(&mut rng, SCRYPT_PARAM_RANGE),
            rand::Rng::gen_range(&mut rng, SCRYPT_PARAM_RANGE),
        );
        let params = ScryptParams::new(log_n, r, p);
        let hash = scrypt_simple(secret, &params)?;
        Ok(Self {
            raw: secret.to_owned(),
            hash,
        })
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
