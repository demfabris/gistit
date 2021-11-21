//! The Send feature

use std::ffi::OsStr;
use std::time::{SystemTime, UNIX_EPOCH};

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
pub struct Payload {
    pub file: Box<dyn FileReady + Send + Sync>,
    pub params: SendParams,
    pub maybe_secret: Option<HashedSecret>,
}

impl Payload {
    /// Trivially initialize payload structure
    #[must_use]
    fn new(
        file: Box<dyn FileReady + Send + Sync>,
        params: SendParams,
        maybe_secret: Option<HashedSecret>,
    ) -> Self {
        Self {
            file,
            params,
            maybe_secret,
        }
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
    async fn hash(&self) -> Result<String> {
        let file_buf = self.file.to_bytes().await?;
        let maybe_secret_str = self.maybe_secret.as_ref().map_or("", |s| s.to_str());
        // Digest and collect output
        let hash = Hasher::default()
            .digest_buf(&file_buf)
            .digest_str(maybe_secret_str)
            .consume()
            .result_str();
        Ok(hash)
    }

    async fn into_json(self, hash: &str) -> Result<serde_json::Value> {
        let file_ref = self.file.inner();
        Ok(json!({
            "hash": hash,
            "author": self.params.author,
            "description": self.params.description,
            "colorscheme": self.params.colorscheme,
            "lifespan": self.params.lifespan,
            "secret": self.maybe_secret.map(|t| t.to_str().to_owned()),
            "timestamp": SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Check your system time")
                .as_millis()
                .to_string(),
            "gistit": {
                "name": file_ref.name(),
                "lang": file_ref.lang(),
                "size": file_ref.inner.metadata().await?.len(),
                "data": base64::encode(self.file.to_bytes().await?)
            }
        }))
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
        // Check params first and exit faster if there's a invalid input
        let params = Params::from_send(self)?.check_consume()?;
        // If secret provided, hash it and encrypt file
        let (file, maybe_hashed_secret): (Box<dyn FileReady + Send + Sync>, Option<HashedSecret>) = {
            let file = File::from_path(self.file).await?.check_consume().await?;

            if let Some(secret_str) = self.secret {
                let hashed_secret = Secret::new(secret_str).check_consume()?.into_hashed()?;
                let encrypted_file = file.into_encrypted(hashed_secret.to_str()).await?;
                (Box::new(encrypted_file), Some(hashed_secret))
            } else {
                (Box::new(file), None)
            }
        };
        let payload = Payload::new(file, params, maybe_hashed_secret);
        Ok(payload)
    }
    async fn dispatch(&self, payload: Self::InnerData) -> Result<()> {
        if self.dry_run {
            return Ok(());
        }
        let hash = payload.hash().await?;
        let json = payload.into_json(&hash).await?;
        if self.clipboard {
            Clipboard::new(hash)
                .try_into_selected()?
                .into_provider()
                .set_contents()?;
        }
        Ok(())
    }
}
