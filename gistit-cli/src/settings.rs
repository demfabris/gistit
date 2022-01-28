//! The settings module
//!
//! Here we manage the local settings file and merge it's params with a given gistit action

use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use lib_gistit::file::File;

use crate::LOCALFS_SETTINGS;
use crate::{ErrorKind, Result, GISTIT_APPLICATION, GISTIT_ORGANIZATION, GISTIT_QUALIFIER};

#[doc(hidden)]
const GISTIT_SETTINGS_FILE_NAME: &str = "settings.yaml";

/// Structured settings params that can be set in `settings.yaml`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub gistit_send: Option<GistitSend>,
    pub gistit_fetch: Option<GistitFetch>,
    pub gistit_global: Option<GistitGlobal>,
}

impl ToString for Settings {
    fn to_string(&self) -> String {
        format!("Mem location: {:p}", self)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            gistit_send: Some(GistitSend::default()),
            gistit_fetch: Some(GistitFetch::default()),
            gistit_global: Some(GistitGlobal::default()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GistitGlobal {
    pub save_location: Option<PathBuf>,
}

impl Default for GistitGlobal {
    fn default() -> Self {
        Self {
            save_location: Some(project_dirs().data_dir().to_path_buf()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GistitSend {
    pub author: Option<String>,
    pub clipboard: Option<bool>,
}

impl Default for GistitSend {
    fn default() -> Self {
        Self {
            author: names::Generator::default().next(),
            clipboard: Some(false),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GistitFetch {
    pub colorscheme: Option<String>,
    pub save: Option<bool>,
}

impl Default for GistitFetch {
    fn default() -> Self {
        Self {
            colorscheme: Some(String::from("ansi")),
            save: Some(false),
        }
    }
}

pub trait Mergeable: Default {
    fn merge(self: Box<Self>, maybe_rhs: Option<Self>) -> Self;
}

impl Mergeable for GistitGlobal {
    /// Merge optional `Self` fields to `rhs` having preference over it.
    fn merge(self: Box<Self>, maybe_rhs: Option<Self>) -> Self {
        let rhs = maybe_rhs.unwrap_or_default();
        Self {
            save_location: self.save_location.or(rhs.save_location),
        }
    }
}

impl Mergeable for GistitSend {
    fn merge(self: Box<Self>, maybe_rhs: Option<Self>) -> Self {
        let rhs = maybe_rhs.unwrap_or_default();
        let clipboard = map_false_to_none(self.clipboard);
        Self {
            author: self.author.or(rhs.author),
            clipboard: clipboard.or(rhs.clipboard),
        }
    }
}

impl Mergeable for GistitFetch {
    fn merge(self: Box<Self>, maybe_rhs: Option<Self>) -> Self {
        let rhs = maybe_rhs.unwrap_or_default();
        let save = map_false_to_none(self.save);
        Self {
            colorscheme: self.colorscheme.or(rhs.colorscheme),
            save: save.or(rhs.save),
        }
    }
}

// Disallowing this so we can keep `const`
#[allow(clippy::option_if_let_else)]
const fn map_false_to_none(arg: Option<bool>) -> Option<bool> {
    if let Some(flag) = arg {
        if flag {
            Some(true)
        } else {
            None
        }
    } else {
        None
    }
}

/// Get the runtime settings read from local config directory
///
/// # Errors
///
/// Fails with [`InternalError`] if something goes wrong when loading this settings
pub fn get_runtime_settings() -> Result<&'static Settings> {
    Ok(LOCALFS_SETTINGS.get().ok_or(ErrorKind::Settings)?)
}

/// Return platform specific project directories
#[must_use]
pub fn project_dirs() -> ProjectDirs {
    ProjectDirs::from(GISTIT_QUALIFIER, &GISTIT_ORGANIZATION, GISTIT_APPLICATION)
        .expect("To read project directory")
}

/// Validate the global settings values.
/// We validate it here because it's not part of any specific gistit action.
///
/// # Errors
///
/// Fails with [`SettingsError`] if some parameter does not meet requirements
fn validate_global_settings(global: &GistitGlobal) -> Result<()> {
    if let Some(ref save_location) = global.save_location {
        if fs::metadata(save_location).is_err() {
            fs::create_dir(save_location)?;
        }
    }
    Ok(())
}

impl Settings {
    /// Merge default settings with the one found in the user system.
    /// The preference follows: arguments > settings file > app defaults.
    ///
    /// # Errors
    ///
    /// Fails with [`SettingsError`] if any invalid value is found in the settings file.
    pub fn merge_local(self) -> Result<Self> {
        let path = project_dirs().config_dir().join(GISTIT_SETTINGS_FILE_NAME);

        if let Ok(handler) = File::from_path(&path) {
            // Checking Md5Sum is quicker than matching fields one by one
            if user_has_default_settings(handler.data()) {
                return Ok(self);
            }

            let theirs: Self = serde_yaml::from_slice(handler.data())?;

            let global = theirs.gistit_global.map_or(GistitGlobal::default(), |t| {
                Box::new(t).merge(self.gistit_global)
            });
            let send = theirs.gistit_send.map_or(GistitSend::default(), |t| {
                Box::new(t).merge(self.gistit_send)
            });
            let fetch = theirs.gistit_fetch.map_or(GistitFetch::default(), |t| {
                Box::new(t).merge(self.gistit_fetch)
            });
            validate_global_settings(&global)?;

            Ok(Self {
                gistit_global: Some(global),
                gistit_send: Some(send),
                gistit_fetch: Some(fetch),
            })
        } else {
            Ok(self)
        }
    }

    /// Creates a new default settings file with [`SETTINGS_FILE_TEMPLATE`] contents and saves it
    /// in the project `config_dir` path.
    /// This is useful to reset the file to defaults as well.
    ///
    /// # Errors
    ///
    /// Fails with [`IoError`] if something goes wrong opening/writing to the file.
    pub fn save_new() -> Result<()> {
        Ok(File::from_bytes(
            SETTINGS_FILE_TEMPLATE.as_bytes().to_owned(),
            GISTIT_SETTINGS_FILE_NAME,
        )?
        .save_as(&project_dirs().config_dir().join(GISTIT_SETTINGS_FILE_NAME))?)
    }
}

/// Check if local settings file match app defaults. Useful to skip deserializing if the user has
/// default settings in usage.
fn user_has_default_settings(theirs: &[u8]) -> bool {
    let theirs_hash = md5::compute(theirs);
    &format!("{:x}", theirs_hash) == DEFAULT_SETTINGS_MD5
}

const DEFAULT_SETTINGS_MD5: &str = "db9fdcdb6f1314f3388659349436a786";

/// Default settings file content as str
pub const SETTINGS_FILE_TEMPLATE: &str = r#"---
gistit_global:
  # The place to save gistits, defaults to project data directory, e.g:
  #
  # Linux:
  # `$XDG_DATA_HOME/_project_path_ or $HOME/.local/share/_project_path_`
  #
  # Windows:
  # `{FOLDERID_RoamingAppData}\_project_path_\data`
  #
  # MacOs:
  # `$HOME/Library/Application Support/_project_path_`
  #
  # Must be a valid **ABSOLUTE** path with read/write permissions.
  # (leave null to use default)
  save_location: null

gistit_send:
  # Annotate sent gistits with an author name.
  # Defaults to a random generated `adjective-noun`
  # (leave null to use default)
  author: null

  # Always attempt to copy the sent gistit hash to system clipboard.
  # WARNING: This feature doesn't always work and can prevent you from executing
  # gistit-cli.
  #
  # see how it works at: https://gistit.io/docs/clipboard
  clipboard: false

gistit_fetch:
  # Default colorscheme to preview gistits on your terminal.
  # Supported colorschemes:
  # --- 1337
  # --- Coldark-Cold
  # --- Coldark-Dark
  # --- DarkNeon
  # --- Dracula
  # --- GitHub
  # --- Monokai Extended
  # --- Monokai Extended Bright
  # --- Monokai Extended Light
  # --- Monokai Extended Origin
  # --- Nord
  # --- OneHalfDark
  # --- OneHalfLight
  # --- Solarized (dark)
  # --- Solarized (light)
  # --- Sublime Snazzy
  # --- TwoDark
  # --- Visual Studio Dark+
  # --- ansi
  # --- base16
  # --- base16-256
  # --- gruvbox-dark
  # --- gruvbox-light
  # --- zenburn
  colorscheme: "ansi"

  # Automatically save fetched gistits to local fs.
  # Save location is specified in `global > save_location`
  # (setting this flag will stop asking behavior)
  save: false
  "#;
