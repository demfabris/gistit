use std::option_env;
use url::Url;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::file::{EncodedFileData, File};
use crate::{Error, Result};

const SERVER_URL_BASE: &str = "https://us-central1-gistit-base.cloudfunctions.net";
const SERVER_SUBPATH_GET: &str = "/gistit-base/us-central1/get";
const SERVER_SUBPATH_LOAD: &str = "/gistit-base/us-central1/load";

lazy_static! {
    pub static ref SERVER_URL_GET: Url =
        Url::parse(option_env!("GISTIT_SERVER_URL").unwrap_or(SERVER_URL_BASE))
            .expect("invalid `GISTIT_SERVER_URL` variable")
            .join(SERVER_SUBPATH_GET)
            .unwrap();
    pub static ref SERVER_URL_LOAD: Url =
        Url::parse(option_env!("GISTIT_SERVER_URL").unwrap_or(SERVER_URL_BASE))
            .expect("invalid `GISTIT_SERVER_URL` variable")
            .join(SERVER_SUBPATH_LOAD)
            .unwrap();
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Gistit {
    pub hash: String,
    pub author: String,
    pub description: Option<String>,
    pub timestamp: String,
    pub inner: Inner,
}

impl Gistit {
    /// Converts a [`Gistit`] into our [`File`] format
    ///
    /// # Errors
    ///
    /// Fails if the payload is somehow corrupted
    pub fn to_file(&self) -> Result<File> {
        let name = self.inner.name.clone();

        File::from_bytes_encoded(self.inner.data.inner.as_bytes(), &name)
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
    /// Converts [`Self`] into a [`Gistit`]
    ///
    /// # Errors
    ///
    /// Fails if payload is corrupted
    fn into_gistit(self) -> Result<Gistit>;
}

impl IntoGistit for Response {
    fn into_gistit(self) -> Result<Gistit> {
        match self {
            Self {
                error: Some(msg), ..
            } => Err(Error::Server(msg)),
            Self {
                success: Some(payload),
                ..
            } => Ok(payload),
            _ => unreachable!("gistit server is unreachable"),
        }
    }
}
