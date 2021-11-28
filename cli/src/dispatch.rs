//! The Dispatch trait

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::errors::io::IoError;
use crate::file::{EncryptedFile, File, FileReady};
use crate::{Error, Result};

#[async_trait]
pub trait Dispatch {
    type InnerData;

    /// Perform the checks needed
    async fn prepare(&self) -> Result<Self::InnerData>;

    /// Execute the action
    async fn dispatch(&self, payload: Self::InnerData) -> Result<()>;
}

#[async_trait]
pub trait Hasheable {
    async fn hash(&self) -> Result<String>;
}

#[macro_export]
macro_rules! dispatch_from_args {
    ($mod:path, $args:expr) => {{
        use $mod as module;
        let action = module::Action::from_args($args)?;
        let payload = Dispatch::prepare(&*action).await?;
        Dispatch::dispatch(&*action, payload).await?;
    }};
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct GistitPayload {
    pub hash: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub colorscheme: String,
    pub lifespan: u16,
    pub secret: Option<String>,
    pub timestamp: String,
    pub gistit: GistitInner,
}

impl GistitPayload {
    /// Gets a copy of payload data as base64 decoded bytes
    ///
    /// # Errors
    ///
    /// Fails with [`IoError`]
    pub async fn to_file(&self) -> Result<Box<dyn FileReady + Send + Sync>> {
        if let Some(secret) = &self.secret {
            Ok(Box::new(
                EncryptedFile::from_bytes(self.gistit.to_data_decoded()?)
                    .await?
                    .into_decrypted(secret)
                    .await?,
            ))
        } else {
            Ok(Box::new(
                File::from_bytes(self.gistit.to_data_decoded()?).await?,
            ))
        }
    }
}

/// Type alias for a base64 encoded String
pub type Base64String = String;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct GistitInner {
    pub name: String,
    pub lang: String,
    pub size: u64,
    pub data: Base64String,
}

impl GistitInner {
    /// # Errors
    /// asd
    pub fn to_data_decoded(&self) -> Result<Vec<u8>> {
        base64::decode(self.data.clone()).map_err(|err| Error::IO(IoError::Other(err.to_string())))
    }
}
