use std::fs;
use std::path::{Path, PathBuf};

use directories::{BaseDirs, ProjectDirs};

use crate::{Error, Result};

const APPLICATION: &str = "Gistit";
const ORGANIZATION: &str = "fabricio7p";
const QUALIFIER: &str = "io";

/// Initialize non existent project directories
///
/// # Errors
///
/// Fails if can't create folder in home config directory
pub fn init_dirs() -> Result<()> {
    let config = config_dir()?;
    if fs::metadata(&config).is_err() {
        fs::create_dir(&config)?;
    }

    Ok(())
}

/// Returns the runtime path of this program
///
/// # Errors
///
/// Fails if the machine doesn't have a HOME directory
pub fn runtime_dir() -> Result<PathBuf> {
    let dirs = BaseDirs::new().ok_or(Error::Unknown)?;

    Ok(dirs
        .runtime_dir()
        .map_or_else(std::env::temp_dir, Path::to_path_buf))
}

/// Returns the config path of this program
///
/// # Errors
///
/// Fails if the machine doesn't have a HOME directory
pub fn config_dir() -> Result<PathBuf> {
    Ok(ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .ok_or(Error::Unknown)?
        .config_dir()
        .to_path_buf())
}
