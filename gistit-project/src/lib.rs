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

    use super::{env, Error, Result};

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
        let default = BaseDirs::new()
            .ok_or(Error::Directory("can't open home directory"))?
            .runtime_dir()
            .map_or_else(std::env::temp_dir, Path::to_path_buf);
        Ok(env::var_or_default(env::GISTIT_RUNTIME_VAR, default))
    }

    /// Returns the config path of this program
    ///
    /// # Errors
    ///
    /// Fails if the system doesn't have a HOME directory
    pub fn config() -> Result<PathBuf> {
        let default = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .ok_or(Error::Directory("can't open home directory"))?
            .config_dir()
            .to_path_buf();
        Ok(env::var_or_default(env::GISTIT_CONFIG_VAR, default))
    }

    /// Returns the data path of this program
    ///
    /// # Errors
    ///
    /// Fails if the system doesn't have a HOME directory
    pub fn data() -> Result<PathBuf> {
        let default = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .ok_or(Error::Directory("can't open home directory"))?
            .data_dir()
            .to_path_buf();
        Ok(env::var_or_default(env::GISTIT_DATA_VAR, default))
    }
}

pub mod env {
    use std::env;
    use std::path::{Path, PathBuf};

    pub const GISTIT_RUNTIME_VAR: &str = "GISTIT_RUNTIME";

    pub const GISTIT_CONFIG_VAR: &str = "GISTIT_CONFIG";

    pub const GISTIT_DATA_VAR: &str = "GISTIT_DATA";

    pub const GISTIT_SERVER_URL: &str = "GISTIT_SERVER_URL";

    #[must_use]
    pub fn var_or_default(var: &str, default: PathBuf) -> PathBuf {
        env::var_os(var)
            .as_ref()
            .map_or(default, |t| Path::new(t).to_path_buf())
    }
}

pub mod var {
    /// Max gistit size allowed in bytes
    pub const GISTIT_MAX_SIZE: usize = 50_000;

    /// Default server base url
    pub const GISTIT_SERVER_URL_BASE: &str = "https://us-central1-gistit-base.cloudfunctions.net/";
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("project directory error: {0}")]
    Directory(&'static str),

    #[error("io error: {0}")]
    IO(#[from] std::io::Error),
}
