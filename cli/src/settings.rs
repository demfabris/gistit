//! Settings module

use std::path::PathBuf;

use directories::ProjectDirs;

use crate::errors::io::IoError;
use crate::{Error, Result};

#[doc(hidden)]
const GISTIT_QUALIFIER: &str = "io";

#[doc(hidden)]
const GISTIT_ORGANIZATION: &str = "Fabricio7p";

#[doc(hidden)]
const GISTIT_APPLICATION: &str = "Gistit";

#[derive(Clone, Debug)]
pub struct Settings {
    config_dir: PathBuf,
    data_dir: PathBuf,
    gistit_send: GistitSend,
    gistit_fetch: GistitFetch,
    global: GistitGlobal,
}

#[derive(Clone, Debug, Default)]
pub struct GistitGlobal {
    pub save_location: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct GistitSend {
    pub colorscheme: String,
    pub author: Option<String>,
    pub lifespan: u16,
    pub clipboard: bool,
}

impl Default for GistitSend {
    fn default() -> Self {
        Self {
            colorscheme: "ansi".to_owned(),
            author: None,
            lifespan: 3600,
            clipboard: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GistitFetch {
    pub colorscheme: String,
    pub save: bool,
    pub preview: bool,
}

impl Default for GistitFetch {
    fn default() -> Self {
        Self {
            colorscheme: "ansi".to_owned(),
            save: false,
            preview: false,
        }
    }
}

impl Settings {
    /// # Errors
    ///
    /// Asd
    pub fn new() -> Result<Self> {
        let project = ProjectDirs::from(GISTIT_QUALIFIER, GISTIT_ORGANIZATION, GISTIT_APPLICATION)
            .ok_or_else(|| {
                Error::IO(IoError::Other(
                    "Can't access config file directory".to_owned(),
                ))
            })?;

        Ok(Self {
            config_dir: project.config_dir().to_path_buf(),
            data_dir: project.data_dir().to_path_buf(),
            gistit_send: GistitSend::default(),
            gistit_fetch: GistitFetch::default(),
            global: GistitGlobal::default(),
        })
    }

    // reset config file via a flag. prompt dialog to confirm
    pub fn reset() -> Result<()> {
        todo!()
    }
}
