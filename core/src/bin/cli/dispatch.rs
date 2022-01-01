//! The Dispatch module

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use lib_gistit::file::{EncodedFileData, EncryptedFile, File, FileReady};
use lib_gistit::Result;

use crate::gistit_line_out;

#[cfg(test)]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(doc)]
use lib_gistit::errors::io::IoError;

#[async_trait]
pub trait Dispatch {
    type InnerData;

    /// Perform the checks needed
    async fn prepare(&'static self) -> Result<Self::InnerData>;

    /// Execute the action
    async fn dispatch(&'static self, payload: Self::InnerData) -> Result<()>;
}

#[async_trait]
pub trait Hasheable {
    fn hash(&self) -> String;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
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
                EncryptedFile::from_bytes_encoded(self.gistit.data.inner.as_bytes(), &name)
                    .await?
                    .with_name(&name)
                    .into_decrypted(secret)
                    .await?,
            ))
        } else {
            Ok(Box::new(
                File::from_bytes_encoded(self.gistit.data.inner.as_bytes(), &name)
                    .await?
                    .with_name(&name),
            ))
        }
    }
}

#[cfg(test)]
impl GistitPayload {
    fn with_test_info() -> Self {
        Self {
            hash: "#125b0aeb7fa1bd1e597c9d2ea062a555".to_owned(),
            author: "Matthew McConaughey".to_owned(),
            description: Some("A gistit".to_owned()),
            colorscheme: "ansi".to_owned(),
            lifespan: 3600,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("System time to work")
                .as_millis()
                .to_string(),
            ..Self::default()
        }
    }

    fn with_secret(self, secret: &str) -> Self {
        Self {
            secret: Some(secret.to_string()),
            ..self
        }
    }

    fn with_inner(self, inner: GistitInner) -> Self {
        Self {
            gistit: inner,
            ..self
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub struct GistitInner {
    pub name: String,
    pub lang: String,
    pub size: u64,
    pub data: EncodedFileData,
}

#[cfg(test)]
const FILE_HEADER_ENCRYPTION_PADDING: &str = "########";
#[cfg(test)]
const EXAMPLE_RUST_FILE: &str = r#"// Wow we are testing
fn main() {
    println!("Hello Test");
}"#;

#[cfg(test)]
impl GistitInner {
    fn new(name: &str, lang: &str, size: u64, data: EncodedFileData) -> Self {
        Self {
            name: name.to_owned(),
            lang: name.to_owned(),
            size,
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use lib_gistit::encrypt::encrypt_aes256_u12nonce;
    use lib_gistit::errors::file::FileError;
    use lib_gistit::errors::Error;
    use rand::distributions::Alphanumeric;
    use rand::Rng;

    use super::*;

    #[tokio::test]
    async fn dispatch_gistit_payload_to_file_unencrypted() {
        let encoded_data = base64::encode(EXAMPLE_RUST_FILE);
        let theirs = File::from_bytes_encoded(encoded_data.as_bytes(), "foo.rs")
            .await
            .unwrap()
            .with_name("foo.rs");
        let payload = GistitPayload::with_test_info().with_inner(GistitInner::new(
            &theirs.name(),
            theirs.lang(),
            theirs.size().await,
            theirs.to_encoded_data(),
        ));
        assert_eq!(payload.gistit.data.inner.len(), encoded_data.len());
        // Expect a randomly named file ending with 'foo.rs'
        assert!(payload.gistit.name.contains("foo.rs"));
        let ours = *payload.to_file().await.unwrap().inner().await.unwrap();
        assert_eq!(ours.name(), "foo.rs");
        assert_eq!(ours.lang(), "rust");
        assert_eq!(
            ours.data(),
            base64::decode(encoded_data).unwrap().as_slice()
        );
    }

    #[tokio::test]
    async fn dispatch_gistit_payload_file_encrypted() {
        let (data, nonce) =
            encrypt_aes256_u12nonce("secret".as_bytes(), EXAMPLE_RUST_FILE.as_bytes()).unwrap();
        let mut payload_data = nonce.clone();
        payload_data.extend(FILE_HEADER_ENCRYPTION_PADDING.as_bytes());
        payload_data.extend(&data);
        let payload = GistitPayload::with_test_info()
            .with_secret("secret")
            .with_inner(GistitInner::new(
                "foo.rs",
                "rust",
                data.len() as u64,
                EncodedFileData {
                    inner: base64::encode(payload_data),
                },
            ));
        // Decryption succeded
        let file = payload.to_file().await.unwrap();
        assert_eq!(file.data(), EXAMPLE_RUST_FILE.as_bytes());
    }

    #[tokio::test]
    #[should_panic = "encryption header is corrupted"]
    async fn dispatch_gistit_payload_file_decryption_fails_if_corrupted_header() {
        let (data, nonce) =
            encrypt_aes256_u12nonce("secret".as_bytes(), EXAMPLE_RUST_FILE.as_bytes()).unwrap();
        let mut payload_data = nonce.clone();
        // Corrupting
        payload_data.extend([1, 2, 3]);
        payload_data.extend(FILE_HEADER_ENCRYPTION_PADDING.as_bytes());
        payload_data.extend(&data);
        let payload = GistitPayload::with_test_info()
            .with_secret("secret")
            .with_inner(GistitInner::new(
                "foo.rs",
                "rust",
                data.len() as u64,
                EncodedFileData {
                    inner: base64::encode(payload_data),
                },
            ));
        let _ = payload
            .to_file()
            .await
            .expect("encryption header is corrupted");
    }

    #[tokio::test]
    #[should_panic = "decryption fails due to invalid nonce"]
    async fn dispatch_gistit_payload_file_decryption_fails_invalid_nonce() {
        let (data, nonce) =
            encrypt_aes256_u12nonce("secret".as_bytes(), EXAMPLE_RUST_FILE.as_bytes()).unwrap();
        let mut payload_data = rand::random::<[u8; 12]>().to_vec();
        // Corrupting
        payload_data.extend(FILE_HEADER_ENCRYPTION_PADDING.as_bytes());
        payload_data.extend(&data);
        let payload = GistitPayload::with_test_info()
            .with_secret("secret")
            .with_inner(GistitInner::new(
                "foo.rs",
                "rust",
                data.len() as u64,
                EncodedFileData {
                    inner: base64::encode(payload_data),
                },
            ));
        let _ = payload
            .to_file()
            .await
            .expect("decryption fails due to invalid nonce");
    }

    #[tokio::test]
    #[should_panic = "decryption fails due to invalid secret"]
    async fn dispatch_gistit_payload_file_decryption_fails_invalid_secret() {
        let (data, nonce) =
            encrypt_aes256_u12nonce("secret".as_bytes(), EXAMPLE_RUST_FILE.as_bytes()).unwrap();
        let mut payload_data = rand::random::<[u8; 12]>().to_vec();
        // Corrupting
        payload_data.extend(FILE_HEADER_ENCRYPTION_PADDING.as_bytes());
        payload_data.extend(&data);
        let payload = GistitPayload::with_test_info()
            .with_secret("the wrong secret")
            .with_inner(GistitInner::new(
                "foo.rs",
                "rust",
                data.len() as u64,
                EncodedFileData {
                    inner: base64::encode(payload_data),
                },
            ));
        let _ = payload
            .to_file()
            .await
            .expect("decryption fails due to invalid secret");
    }
}
