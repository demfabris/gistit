//
//   ________.__          __  .__  __
//  /  _____/|__| _______/  |_|__|/  |_
// /   \  ___|  |/  ___/\   __\  \   __\
// \    \_\  \  |\___ \  |  | |  ||  |
//  \______  /__/____  > |__| |__||__|
//         \/        \/
//
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
// This would decrease readability
#![allow(clippy::module_name_repetitions)]
// Not my fault
#![allow(clippy::multiple_crate_versions)]
// Boring
#![allow(clippy::missing_panics_doc)]
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

pub mod encrypt;
pub mod errors;
pub mod file;

#[cfg(feature = "clipboard")]
pub mod clipboard;

#[cfg(feature = "host")]
pub mod ipc;
#[cfg(feature = "host")]
pub mod network;

pub use errors::Error;
pub type Result<T> = std::result::Result<T, Error>;
