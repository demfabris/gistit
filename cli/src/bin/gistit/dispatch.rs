//! The Dispatch module

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use lib_gistit::file::{EncodedFileData, EncryptedFile, File, FileReady};
use lib_gistit::Result;

use crate::gistit_line_out;

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
    fn hash(&self) -> String;
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct GistitPayload {
    pub hash: String,
    pub author: String,
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
        let name = self.gistit.name.clone();
        if let Some(secret) = &self.secret {
            gistit_line_out!("Decrypting...");
            Ok(Box::new(
                EncryptedFile::from_bytes_encoded(self.gistit.data.inner.as_bytes())
                    .await?
                    .with_name(name)
                    .into_decrypted(secret)
                    .await?,
            ))
        } else {
            Ok(Box::new(
                File::from_bytes_encoded(self.gistit.data.inner.as_bytes())
                    .await?
                    .with_name(name),
            ))
        }
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct GistitInner {
    pub name: String,
    pub lang: String,
    pub size: u64,
    pub data: EncodedFileData,
}
