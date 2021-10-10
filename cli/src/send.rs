//! The Send feature

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
pub(crate) struct Payload {
    file: File,
    addons: addons::Addons,
    secret: crate::encrypt::Secret,
}

/// The dispatch implementation for Send action
#[async_trait::async_trait]
impl Dispatch for Action {
    async fn prepare(&self) -> Result<()> {
        // Check file input
        let _file = File::from_path(self.file.clone())
            .await?
            .check_consume()
            .await?;
        // Check secret input
        if let Some(ref secret) = self.secret {
            let _secret = Secret::from_raw(secret)?.check_consume().await?;
        }
        // Check addons inputs
        let _addons = Addons::new(&self.theme, self.lifespan)
            .with_optional(self.description.clone(), self.author.clone())
            .check_consume()
            .await?;

        Ok(())
    }
    async fn dispatch(&self) -> Result<()> {
        Ok(())
    }
}
