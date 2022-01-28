use std::path::{Path, PathBuf};

use directories::{BaseDirs, ProjectDirs};

use crate::{ErrorKind, Result};

const APPLICATION: &str = "Gistit";
const ORGANIZATION: &str = "fabricio7p";
const QUALIFIER: &str = "io";

pub fn runtime_dir() -> Result<PathBuf> {
    let dirs = BaseDirs::new().ok_or(ErrorKind::Unknown)?;
    Ok(dirs
        .runtime_dir()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::temp_dir()))
}

pub fn config_dir() -> Result<PathBuf> {
    Ok(ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .ok_or(ErrorKind::Unknown)?
        .config_dir()
        .to_path_buf())
}
