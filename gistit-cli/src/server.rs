use std::option_env;
use url::Url;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::file::{B64EncodedFileData, File};
use crate::{Error, Result};

#[cfg(debug_assertions)]
lazy_static! {
    static ref SERVER_URL_BASE: Url = Url::parse("http://localhost:4001/")
        .unwrap()
        .join("gistit-base/")
        .unwrap()
        .join("us-central1/")
        .unwrap();
}

#[cfg(not(debug_assertions))]
lazy_static! {
    static ref SERVER_URL_BASE: Url =
        Url::parse("https://us-central1-gistit-base.cloudfunctions.net/").unwrap();
}

const SERVER_SUBPATH_GET: &str = "get";
const SERVER_SUBPATH_LOAD: &str = "load";
const SERVER_SUBPATH_TOKEN: &str = "token";

lazy_static! {
    pub static ref SERVER_URL_GET: Url =
        Url::parse(option_env!("GISTIT_SERVER_URL").unwrap_or_else(|| SERVER_URL_BASE.as_str()))
            .expect("invalid `GISTIT_SERVER_URL` variable")
            .join(SERVER_SUBPATH_GET)
            .unwrap();
    pub static ref SERVER_URL_LOAD: Url =
        Url::parse(option_env!("GISTIT_SERVER_URL").unwrap_or_else(|| SERVER_URL_BASE.as_str()))
            .expect("invalid `GISTIT_SERVER_URL` variable")
            .join(SERVER_SUBPATH_LOAD)
            .unwrap();
    pub static ref SERVER_URL_TOKEN: Url =
        Url::parse(option_env!("GISTIT_SERVER_URL").unwrap_or_else(|| SERVER_URL_BASE.as_str()))
            .expect("invalid `GISTIT_SERVER_URL` variable")
            .join(SERVER_SUBPATH_TOKEN)
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

        File::from_bytes_encoded(self.inner.data.0.as_bytes(), &name)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Inner {
    pub name: String,
    pub lang: String,
    pub size: usize,
    pub data: B64EncodedFileData,
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
