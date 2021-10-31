//! The encryption module
//!
//! # Contents
//!
//! Here you'll find every entity related to secret parsing, hashing and encrypting operations.
//! Some 'good enough' defaults were used to achieve ease to use with a hopefully scalable
//! implementation in the event of this becomming a library as well.
//!
//! ## Md5
//!
//! Md5 digestion algorithm was used to implement the payload hashing procedure since we just need
//! a sensible hash to uniquely identify our Gistits and here security is not an concern. Also,
//! 128bits is fine and doesn't look too long of a string when copying and sharing.
//!
//! ## Scrypt
//!
//! Scrypt algorithm was used to hash the provided secret (aka password). The params values choosen
//! `N = 2^20`, `R = 8`, and `P = 1` as discussed [here](https://blog.filippo.io/the-scrypt-parameters/).
use async_trait::async_trait;
use crypto::aes;
use crypto::buffer::{BufferResult, ReadBuffer, RefReadBuffer, RefWriteBuffer, WriteBuffer};
use crypto::digest::Digest;
use crypto::hmac::Hmac;
use crypto::mac::{Mac, MacResult};
use crypto::md5::Md5;
use crypto::scrypt::{scrypt_simple, ScryptParams};

use crate::errors::encryption::EncryptionError;
use crate::Result;

/// Allowed secret character range
const ALLOWED_SECRET_CHAR_LENGTH_RANGE: std::ops::RangeInclusive<usize> = 5..=50;

#[doc(alias = "constants")]
const SCRYPT_PARAM_P: u32 = 1;
#[doc(alias = "constants")]
const SCRYPT_PARAM_R: u32 = 8;
#[doc(alias = "constants")]
const SCRYPT_PARAM_LOG_N: u8 = 2;

/// Initialization vector size
const INIT_VECTOR_SIZE: usize = 16;

/// The initialization vector type. Used in cbc encryption algorithm
type InitVector = [u8; INIT_VECTOR_SIZE];

/// The data structure to hold the provided secret and it's Scrypt generated hash
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
    /// Fails with [`Encryption`] error
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
    /// Fails with [`Encryption`] error
    pub async fn check_consume(self) -> Result<Self> {
        log::trace!("[SECRET]");
        <Self as Check>::length(&self).await?;
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
    async fn length(&self) -> Result<()>;
}

#[async_trait]
impl Check for Secret {
    async fn length(&self) -> Result<()> {
        if ALLOWED_SECRET_CHAR_LENGTH_RANGE.contains(&self.inner.len()) {
            log::trace!("[OK]: Secret length");
            Ok(())
        } else {
            Err(EncryptionError::SecretLength.into())
        }
    }
}

/// The hasher structure, generic over the digest algorithm
#[derive(Clone, Debug)]
pub struct Hasher<A>
where
    A: Digest + Sync + Send,
{
    inner: A,
    raw: Vec<u8>,
}

impl<A> Hasher<A>
where
    A: Digest + Sync + Send,
{
    /// Creates a new Hasher with digestion algorithm
    pub fn new(digest: A) -> Self {
        Self {
            inner: digest,
            raw: Vec::new(),
        }
    }

    /// Parses a vector of bytes if non-zero length
    pub fn digest_buf(mut self, buf: impl AsRef<[u8]>) -> Self {
        let buf_ref = buf.as_ref();
        if !buf_ref.is_empty() {
            self.raw.extend(buf_ref);
            Digest::input(&mut self.inner, buf_ref);
        }
        self
    }

    /// Parses a string slice if non-zero length
    pub fn digest_str(mut self, string: impl AsRef<str>) -> Self {
        let str_ref = string.as_ref();
        if !str_ref.is_empty() {
            self.raw.extend(str_ref.as_bytes());
            Digest::input_str(&mut self.inner, str_ref);
        }
        self
    }

    /// Converts into [`Hmac`] and applies digested data (if any)
    pub fn into_hmac(mut self, key: impl AsRef<[u8]>) -> Hmac<A> {
        self.inner.reset();
        let mut hmac = Hmac::new(self.inner, key.as_ref());
        if !self.raw.is_empty() {
            hmac.input(&self.raw);
        }
        hmac
    }

    /// Consumes self and return inner digestor
    pub fn consume(self) -> A {
        self.inner
    }
}

// TODO: Branch the digestor into features
impl Default for Hasher<Md5> {
    /// Defaults to Md5 digestion algorithm
    fn default() -> Self {
        let md5 = Md5::new();
        Self {
            inner: md5,
            raw: Vec::new(),
        }
    }
}

/// The first [`Cryptor`] state
#[doc(hidden)]
#[derive(Debug, Clone, Default)]
pub struct Uninitialized;

/// The encrypting [`Cryptor`] state
#[doc(hidden)]
pub struct Encrypting {
    executor: Box<dyn crypto::symmetriccipher::Encryptor>,
}

/// The decrypting [`Cryptor`] state
#[doc(hidden)]
pub struct Decrypting {
    executor: Box<dyn crypto::symmetriccipher::Decryptor>,
}

/// The done [`Cryptor`] state
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct Done {
    output: Vec<u8>,
}

/// Marker state trait
pub trait State {}
impl State for Uninitialized {}
impl State for Encrypting {}
impl State for Decrypting {}
impl State for Done {}

/// The encrypting/decrypting agent.
/// Implemented using a type state machine to provide a safer and one-way API
#[derive(Debug, Clone)]
pub struct Cryptor<'k, S>
where
    S: State,
{
    state: Box<S>,
    key: &'k [u8],
    iv: InitVector,
}

/// Helper function to initialize [`Cryptor`]
#[must_use]
pub fn cryptor_simple(key: &str) -> Cryptor<'_, Uninitialized> {
    Cryptor::begin(Uninitialized::default(), key.as_bytes())
}

impl<'k, S> Cryptor<'k, S>
where
    S: State + Default,
{
    /// Constructs a [`Cryptor`] in [`Uninitialized`] state to begin operate
    pub fn begin(state: S, key: &'k [u8]) -> Self {
        let mut rng = rand::thread_rng();
        let iv: InitVector = rand::Rng::gen(&mut rng);
        Self {
            state: Box::new(state),
            key,
            iv,
        }
    }

    /// Converts self into [`Cryptor`] in [`Encrypting`] state
    #[must_use]
    pub fn into_encryptor(self) -> Cryptor<'k, Encrypting> {
        let executor = aes::cbc_encryptor(
            crypto::aes::KeySize::KeySize256,
            self.key,
            &self.iv,
            crypto::blockmodes::PkcsPadding,
        );
        Cryptor {
            state: Box::new(Encrypting { executor }),
            key: self.key,
            iv: self.iv,
        }
    }

    /// Converts self into [`Cryptor`] in [`Decrypting`] state
    #[must_use]
    pub fn into_decryptor(self) -> Cryptor<'k, Decrypting> {
        let executor = aes::cbc_decryptor(
            crypto::aes::KeySize::KeySize256,
            self.key,
            &self.iv,
            crypto::blockmodes::PkcsPadding,
        );
        Cryptor {
            state: Box::new(Decrypting { executor }),
            key: self.key,
            iv: self.iv,
        }
    }
}

impl<'k> Cryptor<'k, Encrypting> {
    /// Encrypts the input and returns [`Cryptor`] in [`Done`] state or fails
    ///
    /// # Errors
    ///
    /// Fails with [`Encryption`] error which is derived from [`SymmetricCipherError`]
    pub fn encrypt(&mut self, input: &[u8]) -> Result<Cryptor<'k, Done>> {
        let mut read_buf = RefReadBuffer::new(input);
        let mut buffer = [0; 4096];
        let mut write_buf = RefWriteBuffer::new(&mut buffer);
        let mut output: Vec<u8> = Vec::new();
        loop {
            let result = self
                .state
                .executor
                .encrypt(&mut read_buf, &mut write_buf, true)
                .map_err(EncryptionError::CipherError)?;
            output.extend(
                write_buf
                    .take_read_buffer()
                    .take_remaining()
                    .iter()
                    .copied(),
            );
            if let BufferResult::BufferUnderflow = result {
                break;
            }
        }
        Ok(Cryptor {
            state: Box::new(Done { output }),
            key: self.key,
            iv: self.iv,
        })
    }
}

impl<'k> Cryptor<'k, Decrypting> {
    /// Decrypts the input and returns [`Cryptor`] in [`Done`] state or fails
    ///
    /// # Errors
    ///
    /// Fails with [`Encryption`] error which is derived from [`SymmetricCipherError`]
    pub fn decrypt(&mut self, input: &[u8]) -> Result<Cryptor<'k, Done>> {
        let mut read_buf = RefReadBuffer::new(input);
        let mut buffer = [0; 4096];
        let mut write_buf = RefWriteBuffer::new(&mut buffer);
        let mut output: Vec<u8> = Vec::new();
        loop {
            let result = self
                .state
                .executor
                .decrypt(&mut read_buf, &mut write_buf, true)
                .map_err(EncryptionError::CipherError)?;
            output.extend(
                write_buf
                    .take_read_buffer()
                    .take_remaining()
                    .iter()
                    .copied(),
            );
            if let BufferResult::BufferUnderflow = result {
                break;
            }
        }
        Ok(Cryptor {
            state: Box::new(Done { output }),
            key: self.key,
            iv: self.iv,
        })
    }
}

impl<'k> Cryptor<'k, Done> {
    /// Reset [`Cryptor`] state to [`Uninitialized`] keeping the key
    #[must_use]
    pub fn reset(&self) -> Cryptor<'k, Uninitialized> {
        Cryptor {
            state: Box::new(Uninitialized {}),
            key: self.key,
            iv: self.iv,
        }
    }

    /// Returns a reference to the encrypted/decrypted data
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.state.output.as_ref()
    }

    /// Returns a byte slice from a [`Hmac`] digested with algorithm `A`
    #[must_use]
    pub fn into_hmac_with<A>(&self, digestor: A) -> Hmac<A>
    where
        A: Digest + Send + Sync,
    {
        Hasher::new(digestor)
            .digest_buf(self.as_bytes())
            .into_hmac(self.key)
    }

    /// Returns a byte slice from a [`Hmac`] digested with default [`Md5`] algorithm
    #[must_use]
    pub fn hmac_raw_default(&self) -> Hmac<Md5> {
        Hasher::default()
            .digest_buf(self.as_bytes())
            .into_hmac(self.key)
    }

    /// Compares a raw hmac bytes with the expected
    pub fn verify(&self, rhs: impl AsRef<[u8]>) -> bool {
        let expected = self.hmac_raw_default().result();
        let provided = MacResult::new(rhs.as_ref());
        expected == provided
    }
}
