//! The Send feature
use std::{convert::TryFrom, ffi::OsString};

use async_trait::async_trait;
use crypto::digest::Digest;

use crate::cli::{Command, MainArgs};
use crate::clipboard::Clipboard;
use crate::dispatch::Dispatch;
use crate::encrypt::{HashedSecret, Hasher, Secret};
use crate::{Error, Result};

use addons::Addons;
use file::{File, FileReady};

pub mod addons;
pub mod file;

/// The Send action runtime parameters
pub struct Action {
    /// The file to be sent.
    file: OsString,
    /// The description of this Gistit.
    description: Option<String>,
    /// The author information.
    author: Option<String>,
    /// The colorscheme to be displayed.
    theme: String,
    /// The custom lifespan of a Gistit snippet.
    lifespan: u16,
    /// The password to encrypt.
    secret: Option<String>,
    /// Whether or not to copy successfully sent gistit hash to clipboard.
    clipboard: bool,
    /// Common param to dry run
    #[doc(hidden)]
    dry_run: bool,
    /// Param to exit for some reason
    #[doc(hidden)]
    _early_exit: bool,
}

impl TryFrom<&MainArgs> for Action {
    type Error = Error;

    /// Parse [`MainArgs`] into the Send action or error out
    fn try_from(top_args: &MainArgs) -> std::result::Result<Self, Self::Error> {
        if let Command::Send(ref args) = top_args.action {
            Ok(Self {
                file: args.file.clone(),
                description: args.description.as_ref().cloned(),
                author: args.author.as_ref().cloned(),
                secret: args.secret.as_ref().cloned(),
                theme: args.theme.clone(),
                clipboard: args.clipboard,
                lifespan: args.lifespan,
                dry_run: top_args.dry_run,
                _early_exit: top_args.colorschemes,
            })
        } else {
            Err(Self::Error::Argument)
        }
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
impl Dispatch for Action {
    type Payload = Payload;

    /// Build each top level entity and run inner checks concurrently to assert valid input and
    /// output data.
    ///
    /// If all checks runs successfully, assemble the payload structure to later be dispatched
    /// by [`Dispatch::dispatch`]
    async fn prepare(&self) -> Result<Self::Payload> {
        let mut payload = Payload::with_none();
        // Check addons first and exit faster if there's a invalid input
        let addons = Addons::new(&self.theme, self.lifespan)
            .with_optional(self.description.clone(), self.author.clone())
            .check_consume()
            .await?;
        payload.with_addons(addons);
        // Perform the file check
        let file = File::from_path(self.file.clone())
            .await?
            .check_consume()
            .await?;
        // If secret provided, hash it and encrypt file
        if let Some(ref secret_str) = self.secret {
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
            Clipboard::try_new()?
                .check_consume_sync()
                .set(&payload_hash)?;
        };
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
