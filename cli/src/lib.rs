//! Gistit library

#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
// This lint causes clippy to yell on `argh` expanded macro
#![allow(clippy::default_trait_access)]
// This is boring
#![allow(clippy::module_name_repetitions)]
// Test env should be chill
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

pub mod cli;
pub mod clipboard;
pub mod dispatch;
pub mod encrypt;
pub mod send;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unsuported file format")]
    UnsuportedFile(String),
    #[error("failed to read file")]
    Read(#[from] std::io::Error),
    #[error("failed to parse command arguments")]
    Argument,
    #[error("invalid addons setup")]
    InvalidAddons(String),
    #[error("invalid secret")]
    Encryption(String),
    #[error("hashing failed")]
    Hashing(String),
    #[error("clipboard error")]
    Clipboard(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
