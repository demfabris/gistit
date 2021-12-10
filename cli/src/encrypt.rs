//! The encryption module
//!
//! ## Hashing
//!
//! Md5 digestion algorithm was used to implement the Gistit payload hashing procedure since
//! we just need a fast hash algorithm to uniquely identify our Gistits and here security is not
//! an concern. Also, 128bits is fine and doesn't look too long of a hex-string when copying
//! and sharing.
//!
//! ## Secrets
//!
//! Scrypt algorithm was used to hash the provided secret. The params values choosen
//! `N = 2^2`, `R = 8`, and `P = 1` as discussed [here](https://blog.filippo.io/the-scrypt-parameters/).
//!
//! ## Encryption
//!
//! The encryption/decryption process relies on `AesGcm` algorithm with 256-bit key and 96-bit
//! nounce. See [`aes_gcm`] for more info.

use aes_gcm::aead::{Aead, NewAead};
use aes_gcm::{Aes256Gcm, Key, Nonce};

use async_trait::async_trait;
use crypto::digest::Digest;
use crypto::md5::Md5;
use crypto::scrypt::{scrypt_simple, ScryptParams};

use crate::errors::encryption::EncryptionError;
use crate::Result;

/// Allowed secret character range
const ALLOWED_SECRET_CHAR_LENGTH_RANGE: std::ops::RangeInclusive<usize> = 5..=50;

#[doc(hidden)]
const SCRYPT_PARAM_P: u32 = 1;

#[doc(hidden)]
const SCRYPT_PARAM_R: u32 = 8;

#[doc(hidden)]
const SCRYPT_PARAM_LOG_N: u8 = 2;

/// The data structure to hold the provided secret
#[derive(Clone, Default, Debug)]
pub struct Secret {
    inner: String,
}

impl Secret {
    /// Create a new [`Secret`] from a raw secret string slice
    #[must_use]
    pub fn new(secret: &str) -> Self {
        Self {
            inner: secret.to_owned(),
        }
    }

    /// Executes the hash and return a [`HashedSecret`]
    ///
    /// # Errors
    ///
    /// Fails with [`EncryptionError`] error
    pub fn into_hashed(self) -> Result<HashedSecret> {
        let (log_n, r, p) = (SCRYPT_PARAM_LOG_N, SCRYPT_PARAM_R, SCRYPT_PARAM_P);
        let params = ScryptParams::new(log_n, r, p);
        let scrypt_hash = scrypt_simple(self.inner.as_str(), &params)?;
        Ok(HashedSecret { inner: scrypt_hash })
    }

    /// Perform needed checks, consume `Self` and return.
    ///
    /// # Errors
    ///
    /// Fails with [`EncryptionError`] error
    pub fn check_consume(self) -> Result<Self> {
        <Self as Check>::length(&self)?;
        Ok(self)
    }
}

/// The hashed secret
#[derive(Clone, Default, Debug)]
pub struct HashedSecret {
    inner: String,
}

impl HashedSecret {
    /// Returns a reference to the raw secret
    #[must_use]
    pub fn to_str(&self) -> &str {
        &self.inner
    }

    /// Returns hashed secret as a byte vector
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.inner.clone().into_bytes()
    }
}

#[async_trait]
trait Check {
    /// Check for allowed secret length
    fn length(&self) -> Result<()>;
}

#[async_trait]
impl Check for Secret {
    fn length(&self) -> Result<()> {
        if ALLOWED_SECRET_CHAR_LENGTH_RANGE.contains(&self.inner.len()) {
            Ok(())
        } else {
            Err(EncryptionError::SecretLength.into())
        }
    }
}

/// Digests a slice of byte arrays into [`Md5`] and outputs the resulting string
#[must_use]
pub fn digest_md5_multi(inputs: &[&[u8]]) -> String {
    let mut hasher = Md5::new();
    inputs.iter().for_each(|&i| hasher.input(i));
    hasher.result_str()
}

/// Digests a single byte array into [`Md5`] and outputs the resulting string
#[must_use]
pub fn digest_md5(input: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.input(input);
    hasher.result_str()
}

/// Encrypts `raw_data` with a randomly generated `nounce` and a Md5 hash of the provided secret
///
/// # Errors
///
/// Fails with [`EncryptionError`] if the encryption parameters are of unexpected sizes/ranges.
pub fn encrypt_aes256_u12nonce(secret: &[u8], raw_data: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    let hashed_key = digest_md5(secret);
    let key = Key::from_slice(hashed_key.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let magic: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&magic);

    let encrypted = cipher
        .encrypt(nonce, raw_data.as_ref())
        .map_err(EncryptionError::Cipher)?;

    Ok((encrypted, nonce.to_vec()))
}

/// Decrypts `encrypted_data` given the `magic` and a Md5 hash of the provided secret.
/// Expects the same `nounce` (`magic`) and `secret` as given in the encryption process.
///
/// # Errors
///
/// Fails with [`EncryptionError`] if the parameters are invalid or incorrect.
pub fn decrypt_aes256_u12nonce(
    secret: &[u8],
    encrypted_data: &[u8],
    magic: &[u8; 12],
) -> Result<Vec<u8>> {
    let hashed_key = digest_md5(secret);
    let key = Key::from_slice(hashed_key.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(magic);

    let decrypted = cipher
        .decrypt(nonce, encrypted_data.as_ref())
        .map_err(EncryptionError::Cipher)?;

    Ok(decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RSCRYPT_SHORT_HASH_LENGTH: usize = 86;

    #[test]
    fn encrypt_secret_scrypt_basic_hashing() {
        let short_secret = Secret::new("foobar").into_hashed().unwrap();
        assert_eq!(short_secret.to_str().len(), RSCRYPT_SHORT_HASH_LENGTH);
    }

    #[test]
    fn encrypt_digestors_functions_consistent() {
        let alright = "Mathew McConaughey";
        let hash1 = digest_md5(alright.as_bytes());
        let hash2 = digest_md5_multi(&[alright.as_bytes()]);
        assert_eq!(hash1, hash2);

        let never = "gonna give you up";
        let composed = format!("{}{}", alright, never);
        let hash3 = digest_md5(composed.as_bytes());
        let hash4 = digest_md5_multi(&[alright.as_bytes(), never.as_bytes()]);
        assert_eq!(hash3, hash4);
    }

    #[test]
    fn encrypt_basic_encrypt_decrypt() {
        let secret = "foobar";
        let raw_data = "I'm a file, for real";

        let (encrypted_data, magic) =
            encrypt_aes256_u12nonce(secret.as_bytes(), raw_data.as_bytes()).unwrap();

        let magic: [u8; 12] = magic.try_into().unwrap();
        let decrypted_data =
            decrypt_aes256_u12nonce(secret.as_bytes(), encrypted_data.as_slice(), &magic).unwrap();

        assert_eq!(raw_data.as_bytes(), decrypted_data);
    }

    #[test]
    #[should_panic = "invalid nounce"]
    fn encrypt_fails_with_invalid_nounce() {
        let secret = "foobar";
        let raw_data = "I'm a file, for real";

        let (encrypted_data, magic) =
            encrypt_aes256_u12nonce(secret.as_bytes(), raw_data.as_bytes()).unwrap();

        let magic: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let decrypted_data =
            decrypt_aes256_u12nonce(secret.as_bytes(), encrypted_data.as_slice(), &magic)
                .expect("invalid nounce");
    }

    #[test]
    #[should_panic = "invalid secret"]
    fn encrypt_fails_with_invalid_secret() {
        let secret = "foobar";
        let raw_data = "I'm a file, for real";

        let (encrypted_data, magic) =
            encrypt_aes256_u12nonce(secret.as_bytes(), raw_data.as_bytes()).unwrap();

        let magic: [u8; 12] = magic.try_into().unwrap();
        let decrypted_data =
            decrypt_aes256_u12nonce("foobah".as_bytes(), encrypted_data.as_slice(), &magic)
                .expect("invalid secret");
    }

    #[test]
    fn encrypt_data_is_actually_encrypted() {
        let secret = "foobar";
        let raw_data = "I'm a file, for real".as_bytes();

        let (encrypted_data, magic) = encrypt_aes256_u12nonce(secret.as_bytes(), raw_data).unwrap();

        assert_ne!(raw_data, encrypted_data);
    }

    #[test]
    fn encrypt_output_data_is_different_every_time() {
        let secret = "foobar";
        let raw_data = "I'm a file, for real".as_bytes();

        let (encrypted_data1, magic) =
            encrypt_aes256_u12nonce(secret.as_bytes(), raw_data).unwrap();
        let (encrypted_data2, magic) =
            encrypt_aes256_u12nonce(secret.as_bytes(), raw_data).unwrap();

        assert_ne!(encrypted_data1, encrypted_data2);
    }

    #[test]
    #[should_panic = "invalid nounce"]
    fn encrypt_different_magic_doesnt_decrypt_each_other() {
        let secret = "foobar";
        let raw_data = "I'm a file, for real".as_bytes();

        let (encrypted_data1, magic1) =
            encrypt_aes256_u12nonce(secret.as_bytes(), raw_data).unwrap();
        let (encrypted_data2, magic2) =
            encrypt_aes256_u12nonce(secret.as_bytes(), raw_data).unwrap();

        let magic: [u8; 12] = magic1.try_into().unwrap();
        let decrypted_data1 =
            decrypt_aes256_u12nonce(secret.as_bytes(), encrypted_data1.as_slice(), &magic).unwrap();
        assert_eq!(raw_data, decrypted_data1.as_slice());

        let decrypted_data2 =
            decrypt_aes256_u12nonce(secret.as_bytes(), encrypted_data2.as_slice(), &magic)
                .expect("invalid nounce");
    }
}
