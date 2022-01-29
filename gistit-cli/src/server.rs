use serde::{Deserialize, Serialize};

use lib_gistit::file::{EncodedFileData, File};

use crate::{ErrorKind, Result};

pub const SERVER_URL_GET: &str = "https://us-central1-gistit-base.cloudfunctions.net/get";
pub const SERVER_URL_LOAD: &str = "https://us-central1-gistit-base.cloudfunctions.net/load";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Gistit {
    pub hash: String,
    pub author: String,
    pub description: Option<String>,
    pub timestamp: String,
    pub inner: Inner,
}

impl Gistit {
    pub fn to_file(&self) -> Result<File> {
        let name = self.inner.name.clone();

        Ok(File::from_bytes_encoded(
            self.inner.data.inner.as_bytes(),
            &name,
        )?)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Inner {
    pub name: String,
    pub lang: String,
    pub size: usize,
    pub data: EncodedFileData,
}

#[derive(Deserialize, Debug, Default)]
pub struct Response {
    success: Option<Gistit>,
    error: Option<String>,
}

pub trait IntoGistit {
    fn into_gistit(self) -> Result<Gistit>;
}

impl IntoGistit for Response {
    fn into_gistit(self) -> Result<Gistit> {
        match self {
            Self {
                success: Some(payload),
                ..
            } => Ok(payload),
            Self {
                error: Some(msg), ..
            } => Err(ErrorKind::Server(msg).into()),
            _ => unreachable!("gistit server is unreachable"),
        }
    }
}
