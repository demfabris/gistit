//! Gistit library

#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
// This lint causes clippy to yell on `argh` expanded macro
#![allow(clippy::default_trait_access)]
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
pub mod send;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to read file")]
    Read(#[from] std::io::Error),
    #[error("Failed to parse command arguments")]
    Argument,
}

pub type Result<T> = std::result::Result<T, Error>;
