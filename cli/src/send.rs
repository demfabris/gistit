//! The Send feature
use clap::ArgMatches;
use std::convert::TryFrom;
use std::ffi::OsStr;

use async_trait::async_trait;
use crypto::digest::Digest;

use crate::clipboard::Clipboard;
use crate::dispatch::Dispatch;
use crate::encrypt::{HashedSecret, Hasher, Secret};
use crate::{Error, Result};

use addons::Addons;
use file::{File, FileReady};

pub mod addons;
pub mod file;

/// The Send action runtime parameters
pub struct Action<'a> {
    /// The file to be sent.
    file: &'a OsStr,
    /// The description of this Gistit.
    description: Option<&'a str>,
    /// The author information.
    author: Option<&'a str>,
    /// The colorscheme to be displayed.
    theme: &'a str,
    /// The password to encrypt.
    secret: Option<&'a str>,
    /// The custom lifespan of a Gistit snippet.
    lifespan: &'a str,
    /// Whether or not to copy successfully sent gistit hash to clipboard.
    clipboard: bool,
    /// dry_run
    #[doc(hidden)]
    dry_run: bool,
}

impl<'act, 'args> TryFrom<&'act ArgMatches<'args>> for Action<'act> {
    type Error = Error;

    /// Parse [`ArgMatches`] into the Send action or error out
    fn try_from(args: &'act ArgMatches<'args>) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            file: args.value_of_os("file").ok_or(Self::Error::Argument)?,
            description: args.value_of("description"),
            author: args.value_of("author"),
            theme: args.value_of("theme").ok_or(Self::Error::Argument)?,
            secret: args.value_of("secret"),
            lifespan: args.value_of("lifespan").ok_or(Self::Error::Argument)?,
            clipboard: args.is_present("clipboard"),
            dry_run: args.is_present("dry-run"),
        })
    }
}

/// The parsed/checked data that should be dispatched
#[derive(Default)]
pub struct Payload {
    pub file: Option<Box<dyn FileReady + Send + Sync>>,
    pub addons: Option<Addons>,
    pub secret: Option<HashedSecret>,
    pub hash: Option<String>,
}

impl Payload {
    /// Trivially initialize payload structure
    #[must_use]
    pub fn with_none() -> Self {
        Self::default()
    }

    /// Append a checked instance of [`Addons`].
    pub fn with_addons(&mut self, addons: Addons) -> &mut Self {
        self.addons = Some(addons);
        self
    }

    /// Append a checked instance of [`File`] or [`EncryptedFile`]
    pub fn with_file(&mut self, file: Box<dyn FileReady + Send + Sync>) -> &mut Self {
        self.file = Some(file);
        self
    }

    /// Append a checked instance of [`HashedSecret`]
    pub fn with_secret(&mut self, secret: HashedSecret) -> &mut Self {
        self.secret = Some(secret);
        self
    }

    pub fn with_hash(&mut self, hash: impl Into<String>) -> &mut Self {
        self.hash = Some(hash.into());
        self
    }

    /// Hash payload fields.
    /// Reads the inner file contents into a buffer and digest it into the hasher.
    /// If a secret was provided it should be digested by the hasher as well.
    ///
    /// Returns the hashed string hex encoded
    ///
    /// # Errors
    ///
    /// Fails with [`std::io::Error`]
    pub async fn as_hash(&self) -> Result<String> {
        let file_buf = self
            .file
            .as_ref()
            .expect("to have a file")
            .to_bytes()
            .await?;
        let maybe_secret_str = self.secret.as_ref().map_or("", |s| s.to_str());
        // Digest and collect output
        let hash = Hasher::default()
            .digest_buf(&file_buf)
            .digest_str(maybe_secret_str)
            .consume()
            .result_str();
        Ok(hash)
    }
}

/// The dispatch implementation for Send action
#[async_trait]
impl Dispatch for Action<'_> {
    type Payload = Payload;

    /// Build each top level entity and run inner checks concurrently to assert valid input and
    /// output data.
    ///
    /// If all checks runs successfully, assemble the payload structure to later be dispatched
    /// by [`Dispatch::dispatch`]
    async fn prepare(&self) -> Result<Self::Payload> {
        let mut payload = Payload::with_none();
        // Check addons first and exit faster if there's a invalid input
        let addons = Addons::from_action(self)?.check_consume().await?;
        payload.with_addons(addons);
        // Perform the file check
        let file = File::from_path(self.file).await?.check_consume().await?;
        // If secret provided, hash it and encrypt file
        if let Some(secret_str) = self.secret {
            let secret = Secret::new(secret_str)
                .check_consume()
                .await?
                .into_hashed()?;
            let encrypted_file = file.into_encrypted(secret.to_str()).await?;
            payload
                .with_file(Box::new(encrypted_file))
                .with_secret(secret);
        } else {
            payload.with_file(Box::new(file));
        }
        let payload_hash = payload.as_hash().await?;
        payload.with_hash(&payload_hash);
        if self.clipboard {
            Clipboard::new(payload_hash)
                .try_into_selected()?
                .into_provider()
                .set_contents()?;
        }
        Ok(payload)
    }
    async fn dispatch(&self, payload: Self::Payload) -> Result<()> {
        if self.dry_run {
            return Ok(());
        }
        let file = payload.file.unwrap().to_bytes().await?;
        log::trace!("{:?}", file);

        Ok(())
    }
}
