//! Gistit library

#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
// This lint causes clippy to yell on `argh` expanded macro
#![allow(clippy::default_trait_access)]
// This is boring
#![allow(clippy::module_name_repetitions)]
// Not my fault
#![allow(clippy::multiple_crate_versions)]
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
pub mod dispatch;
pub mod encrypt;
pub mod errors;
pub mod send;

#[cfg(feature = "fetch")]
pub mod fetch;

#[cfg(feature = "clipboard")]
pub mod clipboard;

use errors::Error;
pub type Result<T> = std::result::Result<T, Error>;
