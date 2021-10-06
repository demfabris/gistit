//! The Send feature

use std::{convert::TryFrom, ffi::OsString};

use crate::cli::{Command, MainArgs};
use crate::dispatch::Dispatch;
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
    /// The custom lifetime of a Gistit snippet.
    lifetime: u16,
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
/// Parse [`MainArgs`] into the Send action or error out
impl TryFrom<&MainArgs> for Action {
    type Error = Error;
    fn try_from(top_args: &MainArgs) -> std::result::Result<Self, Self::Error> {
        if let Command::Send(ref args) = top_args.action {
            Ok(Self {
                file: args.file.clone(),
                description: args.description.as_ref().cloned(),
                author: args.author.as_ref().cloned(),
                secret: args.secret.as_ref().cloned(),
                theme: args.theme.clone(),
                clipboard: args.clipboard,
                lifetime: args.lifetime,
                dry_run: top_args.dry_run,
                early_exit: top_args.colorschemes,
            })
        } else {
            Err(Self::Error::Argument)
        }
    }
}
/// The dispatch implementation for Send action
#[async_trait::async_trait]
impl Dispatch for Action {
    async fn prepare(&self) -> Result<()> {
        let addons = Addons::from(self);
        let file = File::from_action(self).await?;
        let _ = tokio::try_join!(
            <Addons as addons::Check>::description(&addons),
            <Addons as addons::Check>::author(&addons),
            <Addons as addons::Check>::colorscheme(&addons),
            <Addons as addons::Check>::lifetime(&addons),
            <File as file::Check>::metadata(&file),
            <File as file::Check>::extension(&file),
        )?;
        Ok(())
    }
    async fn dispatch(&self) -> Result<()> {
        Ok(())
    }
}
