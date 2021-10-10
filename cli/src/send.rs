//! The Send feature
use async_trait::async_trait;

use std::{convert::TryFrom, ffi::OsString};

use crate::cli::{Command, MainArgs};
use crate::dispatch::Dispatch;
use crate::encrypt::Secret;
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
    early_exit: bool,
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
                early_exit: top_args.colorschemes,
            })
        } else {
            Err(Self::Error::Argument)
        }
    }
}

/// The parsed/checked data that should be dispatched
#[derive(Debug)]
pub struct Payload {
    file: File,
    addons: Addons,
    secret: Option<Secret>,
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
        let maybe_secret = if let Some(ref secret) = self.secret {
            Some(Secret::from_raw(secret)?.check_consume().await?)
        } else {
            None
        };
        let addons = Addons::new(&self.theme, self.lifespan)
            .with_optional(self.description.clone(), self.author.clone())
            .check_consume()
            .await?;
        Ok(Self::Payload {
            file,
            addons,
            secret: maybe_secret,
        })
    }
    async fn dispatch(&self, payload: Self::Payload) -> Result<()> {
        log::debug!("{:?}", payload);
        Ok(())
    }
}
