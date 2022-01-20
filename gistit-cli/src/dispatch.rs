use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use lib_gistit::file::{EncodedFileData, File};

use crate::Result;

#[cfg(test)]
use std::time::{SystemTime, UNIX_EPOCH};

#[async_trait]
pub trait Dispatch {
    type InnerData;

    async fn prepare(&'static self) -> Result<Self::InnerData>;

    async fn dispatch(&'static self, payload: Self::InnerData) -> Result<()>;
}

pub trait Hasheable {
    fn hash(&self) -> String;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub struct GistitPayload {
    pub hash: String,
    pub author: String,
    pub description: Option<String>,
    pub timestamp: String,
    pub gistit: GistitInner,
}

impl GistitPayload {
    pub fn to_file(&self) -> Result<File> {
        let name = self.gistit.name.clone();
        Ok(File::from_bytes_encoded(
            self.gistit.data.inner.as_bytes(),
            &name,
        )?)
    }
}

#[cfg(test)]
impl GistitPayload {
    fn with_test_info() -> Self {
        Self {
            hash: "#125b0aeb7fa1bd1e597c9d2ea062a555".to_owned(),
            author: "Matthew McConaughey".to_owned(),
            description: Some("A gistit".to_owned()),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("System time to work")
                .as_millis()
                .to_string(),
            ..Self::default()
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
    pub size: usize,
    pub data: EncodedFileData,
}

#[cfg(test)]
impl GistitInner {
    fn new(name: &str, lang: &str, size: usize, data: EncodedFileData) -> Self {
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

    use super::*;

    const FILE_HEADER_ENCRYPTION_PADDING: &str = "########";
    const EXAMPLE_RUST_FILE: &str = r#"// Wow we are testing
fn main() {
    println!("Hello Test");
}"#;

    #[tokio::test]
    async fn dispatch_gistit_payload_to_file_unencrypted() {
        let encoded_data = base64::encode(EXAMPLE_RUST_FILE);
        let theirs = File::from_bytes_encoded(encoded_data.as_bytes(), "foo.rs").unwrap();
        let payload = GistitPayload::with_test_info().with_inner(GistitInner::new(
            &theirs.name(),
            theirs.lang(),
            theirs.size(),
            theirs.to_encoded_data(),
        ));
        assert_eq!(payload.gistit.data.inner.len(), encoded_data.len());
        // Expect a randomly named file ending with 'foo.rs'
        assert!(payload.gistit.name.contains("foo.rs"));
        let ours = payload.to_file().unwrap();
        assert_eq!(ours.name(), "foo.rs");
        assert_eq!(ours.lang(), "rust");
        assert_eq!(
            ours.data(),
            base64::decode(encoded_data).unwrap().as_slice()
        );
    }
}
