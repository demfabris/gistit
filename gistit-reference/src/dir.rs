use std::fs;
use std::path::{Path, PathBuf};

use directories::{BaseDirs, ProjectDirs};

use crate::{ErrorKind, Result};

const APPLICATION: &str = "Gistit";
const ORGANIZATION: &str = "demfabris";
const QUALIFIER: &str = "io";

/// Initialize needed project directories if not present
///
/// # Errors
///
/// Fails if can't create folder in home config directory
pub fn init_dirs() -> Result<()> {
    let config = config_dir()?;
    if fs::metadata(&config).is_err() {
        fs::create_dir(&config)?;
    }

    let data = data_dir()?;
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
pub fn runtime_dir() -> Result<PathBuf> {
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
pub fn config_dir() -> Result<PathBuf> {
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
pub fn data_dir() -> Result<PathBuf> {
    Ok(ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .ok_or(ErrorKind::Directory("can't open home directory"))?
        .data_dir()
        .to_path_buf())
}
