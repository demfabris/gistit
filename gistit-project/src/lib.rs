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

pub const APPLICATION: &str = "Gistit";
pub const ORGANIZATION: &str = "demfabris";
pub const QUALIFIER: &str = "io";

pub mod path {
    use std::fs;
    use std::path::{Path, PathBuf};

    use directories::{BaseDirs, ProjectDirs};

    use crate::{ErrorKind, Result};

    use super::{APPLICATION, ORGANIZATION, QUALIFIER};

    /// Initialize needed project directories if not present
    ///
    /// # Errors
    ///
    /// Fails if can't create folder in home config directory
    pub fn init() -> Result<()> {
        let config = config()?;
        if fs::metadata(&config).is_err() {
            fs::create_dir(&config)?;
        }

        let data = data()?;
        if fs::metadata(&data).is_err() {
            fs::create_dir(&data)?;
        }

        Ok(())
    }

    /// Returns the runtime path of this program
    /// Fallbacks to a temporary folder
    ///
    /// # Errors
    ///
    /// Fails if the system doesn't have a HOME directory
    pub fn runtime() -> Result<PathBuf> {
        let dirs = BaseDirs::new().ok_or(ErrorKind::Directory("can't open home directory"))?;

        Ok(dirs
            .runtime_dir()
            .map_or_else(std::env::temp_dir, Path::to_path_buf))
    }

    /// Returns the config path of this program
    ///
    /// # Errors
    ///
    /// Fails if the system doesn't have a HOME directory
    pub fn config() -> Result<PathBuf> {
        Ok(ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .ok_or(ErrorKind::Directory("can't open home directory"))?
            .config_dir()
            .to_path_buf())
    }

    /// Returns the data path of this program
    ///
    /// # Errors
    ///
    /// Fails if the system doesn't have a HOME directory
    pub fn data() -> Result<PathBuf> {
        Ok(ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .ok_or(ErrorKind::Directory("can't open home directory"))?
            .data_dir()
            .to_path_buf())
    }
}

pub mod env {
    pub const GISTIT_RUNTIME_VAR: &str = "GISTIT_RUNTIME";

    pub const GISTIT_CONFIG_VAR: &str = "GISTIT_CONFIG";
}

pub mod var {
    /// Max gistit size allowed in bytes
    pub const GISTIT_MAX_SIZE: usize = 50_000;
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("project directory error: {0}")]
    Directory(&'static str),

    #[error("io error: {0}")]
    IO(#[from] std::io::Error),
}
