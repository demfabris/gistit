//! The Send feature
use std::{convert::TryFrom, ffi::OsString};

use async_trait::async_trait;
use crypto::digest::Digest;

use crate::cli::{Command, MainArgs};
use crate::clipboard::Clipboard;
use crate::dispatch::Dispatch;
use crate::encrypt::{Hasher, Secret};
use crate::{Error, Result};

use addons::Addons;
use file::File;

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
#[derive(Debug)]
pub struct Payload {
    pub file: File,
    pub addons: Addons,
    pub secret: Option<Secret>,
    pub hash: Option<String>,
}

impl Payload {
    pub const fn new(file: File, addons: Addons) -> Self {
        Self {
            file,
            addons,
            secret: None,
            hash: None,
        }
    }
    #[allow(clippy::missing_const_for_fn)]
    pub fn with_secret(&mut self, secret: Secret) {
        self.secret = Some(secret);
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
    pub async fn hash(&mut self) -> Result<()> {
        let mut hasher = Hasher::default();
        let file_buf = self.file.as_buf().await?;
        hasher.digest_buf(file_buf);
        // If secret was provided, digest it as well
        if let Some(ref secret) = self.secret {
            hasher.digest_str(secret.get_hash());
        }
        self.hash = Some(hasher.consume().result_str());
        Ok(())
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
        let file = File::from_path(self.file.clone())
            .await?
            .check_consume()
            .await?;
        let addons = Addons::new(&self.theme, self.lifespan)
            .with_optional(self.description.clone(), self.author.clone())
            .check_consume()
            .await?;
        let mut payload = Self::Payload::new(file, addons);
        // If a secret was provided, check if it's under spec
        if let Some(ref secret_str) = self.secret {
            let secret = Secret::from_raw(secret_str)?.check_consume().await?;
            payload.with_secret(secret);
        }
        if self.clipboard {
            Clipboard::try_new();
        };
        Payload::hash(&mut payload).await?;
        Ok(payload)
    }
    async fn dispatch(&self, payload: Self::Payload) -> Result<()> {
        if self.dry_run {
            return Ok(());
        }
        log::debug!("{:?}", payload);
        Ok(())
    }
}
