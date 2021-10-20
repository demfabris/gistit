//! Clipboard module
use std::process::Command;

use cli_clipboard::{ClipboardContext, ClipboardProvider};

use crate::Result;

pub struct Clipboard {
    ctx: ClipboardContext,
    value: Option<String>,
}

impl std::fmt::Debug for Clipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.value.as_ref().unwrap_or(&"Null".to_owned()))
    }
}

impl Clipboard {
    /// Try to access clipboard context
    ///
    /// # Errors
    ///
    /// Fails with [`Error::Clipboard`] error
    pub fn try_new() -> Result<Self> {
        let ctx = ClipboardContext::new()?;
        Ok(Self { ctx, value: None })
    }

    /// Perform checks, in this case they are not required. Missing clipboard feature will not
    /// halt execution
    #[must_use]
    pub fn check_consume_sync(self) -> Self {
        <Self as Check>::session();
        self
    }

    /// Returns a mutable reference to the inner clipboard context
    pub fn ctx_mut(&mut self) -> &mut ClipboardContext {
        &mut self.ctx
    }

    /// Set contents of the clipboard in context
    ///
    /// # Errors
    ///
    /// Fails with [`Clipboard`] error
    pub fn set(&mut self, contents: impl Into<String>) -> Result<()> {
        self.ctx.clear()?;
        let string = contents.into();
        log::trace!("Attempting to set clipboard: {}", string);
        self.ctx.set_contents(string.clone())?;
        self.value = Some(string);
        Ok(())
    }

    /// Get the content of what was last set into the clipboard by this program
    pub fn inner_value(&self) -> Option<&str> {
        self.value.as_ref().map(AsRef::as_ref)
    }
}

trait Check {
    /// Check wether or not in a SSH session
    fn session();
}

impl Check for Clipboard {
    fn session() {
        if std::env::var("SSH_CLIENT").is_ok() || std::env::var("SSH_CONNECTION").is_ok() {
            log::debug!("SSH session detected");

            #[cfg(target_os = "linux")]
            {
                if std::env::var("DISPLAY").is_err() {
                    println!("No display detected");
                }
                if Command::new("which").arg("xclip").output().is_err() {
                    println!("No xclip binary detected");
                };
                // TODO: more env checks here
            }
        }
    }
}
