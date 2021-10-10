//! Clipboard module

use async_trait::async_trait;
use cli_clipboard::{ClipboardContext, ClipboardProvider};

use crate::{Error, Result};

#[derive(Default)]
pub struct Clipboard {
    ctx: Option<ClipboardContext>,
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
    #[must_use]
    pub fn try_new() -> Option<Self> {
        let ctx = match ClipboardContext::new() {
            Ok(t) => Some(t),
            Err(e) => {
                log::debug!("{:?}", e);
                // TODO: impl proper error message
                None
            }
        };
        Some(Self { ctx, value: None })
    }

    /// Perform checks, in this case they are not required. Missing clipboard feature will not
    /// halt execution
    pub async fn check_consume(self) -> Self {
        <Self as Check>::session().await;
        self
    }
}

#[async_trait]
trait Check {
    /// Check wether or not in a SSH session
    async fn session();
}

#[async_trait]
impl Check for Clipboard {
    async fn session() {
        if std::env::var("SSH_CLIENT").is_ok() || std::env::var("SSH_CONNECTION").is_ok() {
            log::debug!("under ssh session");

            #[cfg(target_os = "linux")]
            {
                if std::env::var("DISPLAY").is_err() {
                    println!("No display detected");
                }
            }
        }
    }
}

const SPAWN_CLIPBOARD_CTX_ERROR: &str = "could not access clipboard";
