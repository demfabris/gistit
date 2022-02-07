//
//   ________.__          __  .__  __
//  /  _____/|__| _______/  |_|__|/  |_
// /   \  ___|  |/  ___/\   __\  \   __\
// \    \_\  \  |\___ \  |  | |  ||  |
//  \______  /__/____  > |__| |__||__|
//         \/        \/
//
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![cfg_attr(
    test,
    allow(
        unused,
        clippy::all,
        clippy::pedantic,
        clippy::nursery,
        clippy::dbg_macro,
        clippy::unwrap_used,
        clippy::missing_docs_in_private_items,
    )
)]
//! Define some common data structures and project directories for Gistit
pub mod dir;

use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Base64Encoded(pub String);

impl AsRef<[u8]> for Base64Encoded {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
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
    pub fn data(&self) -> &Base64Encoded {
        &self.inner.data
    }

    pub fn name(&self) -> &str {
        &self.inner.name
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Inner {
    pub name: String,
    pub lang: String,
    pub size: usize,
    pub data: Base64Encoded,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    IO(std::io::Error),
    Directory(&'static str),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::IO(err),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self { kind }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "gistit reference error"
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "gistit reference error")
    }
}
