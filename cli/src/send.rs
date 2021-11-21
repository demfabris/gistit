//! The Send feature

use std::ffi::OsStr;

use async_trait::async_trait;
use clap::ArgMatches;
use crypto::digest::Digest;
use serde_json::json;

use crate::clipboard::Clipboard;
use crate::dispatch::Dispatch;
use crate::encrypt::{HashedSecret, Hasher, Secret};
use crate::params::{Params, SendParams};
use crate::{Error, Result};

use file::{File, FileReady};

pub mod file;

/// The Send action runtime parameters
pub struct Action<'a> {
    /// The file to be sent.
    pub file: &'a OsStr,
    /// The description of this Gistit.
    pub description: Option<&'a str>,
    /// The author information.
    pub author: Option<&'a str>,
    /// The colorscheme to be displayed.
    pub theme: &'a str,
    /// The password to encrypt.
    pub secret: Option<&'a str>,
    /// The custom lifespan of a Gistit snippet.
    pub lifespan: &'a str,
    /// Whether or not to copy successfully sent gistit hash to clipboard.
    pub clipboard: bool,
    /// dry_run
    #[doc(hidden)]
    pub dry_run: bool,
}

impl<'act, 'args> Action<'act> {
    /// Parse [`ArgMatches`] into the dispatchable Send action
    ///
    /// # Errors
    ///
    /// Fails with argument errors
    pub fn from_args(
        args: &'act ArgMatches<'args>,
    ) -> Result<Box<dyn Dispatch<InnerData = Payload> + 'act>> {
        Ok(Box::new(Self {
            file: args.value_of_os("file").ok_or(Error::Argument)?,
            description: args.value_of("description"),
            author: args.value_of("author"),
            theme: args.value_of("theme").ok_or(Error::Argument)?,
            secret: args.value_of("secret"),
            lifespan: args.value_of("lifespan").ok_or(Error::Argument)?,
            clipboard: args.is_present("clipboard"),
            dry_run: args.is_present("dry-run"),
        }))
    }
}

/// The parsed/checked data that should be dispatched
#[derive(Default)]
pub struct Payload {
    pub file: Option<Box<dyn FileReady + Send + Sync>>,
    pub params: Option<SendParams>,
    pub secret: Option<HashedSecret>,
    pub hash: Option<String>,
}

impl Payload {
    /// Trivially initialize payload structure
    #[must_use]
    fn with_none() -> Self {
        Self::default()
    }

    /// Append a checked instance of [`Params`].
    fn with_params(&mut self, params: SendParams) -> &mut Self {
        self.params = Some(params);
        self
    }

    /// Append a checked instance of [`File`] or [`EncryptedFile`]
    fn with_file(&mut self, file: Box<dyn FileReady + Send + Sync>) -> &mut Self {
        self.file = Some(file);
        self
    }

    /// Append a checked instance of [`HashedSecret`]
    fn with_secret(&mut self, secret: HashedSecret) -> &mut Self {
        self.secret = Some(secret);
        self
    }

    fn with_hash(&mut self, hash: impl Into<String>) -> &mut Self {
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
    async fn as_hash(&self) -> Result<String> {
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
    type InnerData = Payload;
    /// Build each top level entity and run inner checks concurrently to assert valid input and
    /// output data.
    ///
    /// If all checks runs successfully, assemble the payload structure to later be dispatched
    /// by [`Dispatch::dispatch`]
    async fn prepare(&self) -> Result<Self::InnerData> {
        let mut payload = Payload::with_none();
        // Check params first and exit faster if there's a invalid input
        let params = Params::from_send(self)?.check_consume()?;
        payload.with_params(params);
        // Perform the file check
        let file = File::from_path(self.file).await?.check_consume().await?;
        // If secret provided, hash it and encrypt file
        if let Some(secret_str) = self.secret {
            let secret = Secret::new(secret_str).check_consume()?.into_hashed()?;
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
    async fn dispatch(&self, payload: Self::InnerData) -> Result<()> {
        if self.dry_run {
            return Ok(());
        }
        let req = reqwest::Client::new();
        let res = req.post("https://us-central1-gistit-base.cloudfunctions.net/load");
        Ok(())
    }
}
