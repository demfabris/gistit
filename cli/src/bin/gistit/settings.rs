//! Settings module

use std::path::PathBuf;

use clap::crate_authors;
use directories::ProjectDirs;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use lib_gistit::encrypt::digest_md5;
use lib_gistit::file::{File, FileReady};
use lib_gistit::Result;

#[doc(hidden)]
const GISTIT_QUALIFIER: &str = "io";

lazy_static! {
    static ref GISTIT_ORGANIZATION: &'static str = crate_authors!("-");
}

#[doc(hidden)]
const GISTIT_APPLICATION: &str = "Gistit";

#[doc(hidden)]
const GISTIT_SETTINGS_FILE_NAME: &str = "settings.yaml";

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
    pub colorscheme: Option<String>,
    pub author: Option<String>,
    pub lifespan: Option<String>,
    pub clipboard: Option<bool>,
}

impl Default for GistitSend {
    fn default() -> Self {
        Self {
            colorscheme: Some(String::from("ansi")),
            author: None,
            lifespan: Some(String::from("3600")),
            clipboard: Some(false),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GistitFetch {
    pub colorscheme: Option<String>,
    pub save: Option<bool>,
    pub preview: Option<bool>,
}

impl Default for GistitFetch {
    fn default() -> Self {
        Self {
            colorscheme: Some(String::from("ansi")),
            save: Some(false),
            preview: Some(false),
        }
    }
}

/// Trait to merge optional fields having preference over `self`
pub trait Mergeable: Default {
    fn merge(self: Box<Self>, maybe_rhs: Option<Self>) -> Self;
}

impl Mergeable for GistitGlobal {
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
        Self {
            colorscheme: self.colorscheme.or(rhs.colorscheme),
            author: self.author.or(rhs.author),
            lifespan: self.lifespan.or(rhs.lifespan),
            clipboard: self.clipboard.or(rhs.clipboard),
        }
    }
}

impl Mergeable for GistitFetch {
    fn merge(self: Box<Self>, maybe_rhs: Option<Self>) -> Self {
        let rhs = maybe_rhs.unwrap_or_default();
        Self {
            colorscheme: self.colorscheme.or(rhs.colorscheme),
            save: self.save.or(rhs.save),
            preview: self.preview.or(rhs.preview),
        }
    }
}

/// # Errors
/// asd
#[must_use]
pub fn project_dirs() -> ProjectDirs {
    ProjectDirs::from(GISTIT_QUALIFIER, &GISTIT_ORGANIZATION, GISTIT_APPLICATION)
        .expect("To read project directory")
}

impl Settings {
    /// # Errors
    ///
    /// Asd
    pub async fn merge_local(self) -> Result<Self> {
        let path = project_dirs().config_dir().join(GISTIT_SETTINGS_FILE_NAME);

        if let Ok(handler) = File::from_path(&path).await {
            // Checking Md5Sum is quicker than matching fields one by one
            if user_has_default_settings(handler.data()) {
                return Ok(self);
            }

            let theirs: Self = serde_yaml::from_slice(handler.data()).expect("to deserialize"); // TODO: proper error
            let global = theirs.gistit_global.map_or(GistitGlobal::default(), |t| {
                Box::new(t).merge(self.gistit_global)
            });
            let send = theirs.gistit_send.map_or(GistitSend::default(), |t| {
                Box::new(t).merge(self.gistit_send)
            });
            let fetch = theirs.gistit_fetch.map_or(GistitFetch::default(), |t| {
                Box::new(t).merge(self.gistit_fetch)
            });

            Ok(Self {
                gistit_global: Some(global),
                gistit_send: Some(send),
                gistit_fetch: Some(fetch),
            })
        } else {
            Ok(self)
        }
    }

    // reset config file via a flag. prompt dialog to confirm
    /// # Errors
    /// asd
    pub fn reset() -> Result<()> {
        todo!()
    }
}

fn user_has_default_settings(theirs: &[u8]) -> bool {
    digest_md5(theirs) == digest_md5(SETTINGS_FILE_TEMPLATE.as_bytes())
}

const SETTINGS_FILE_TEMPLATE: &str = r#"
---
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
  # Must be a valid path with read/write permissions.
  # (leave null to use default)
  save_location: null

gistit_send:
  # Default colorscheme to be used when sending.
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

  # Annotate sent gistits with an author name.
  # Defaults to a random generated `adjective-noun`
  # (leave null to use default)
  author: null

  # How long the gistit will be avaiable to be fetched.
  # VALUES: 300 - 3600
  lifespan: 3600

  # Always attempt to copy the sent gistit hash to system clipboard.
  # WARNING: This feature doesn't always work and can prevent you from executing
  # gistit-cli.
  #
  # see how it works at: https://gistit.io/docs/clipboard
  clipboard: false

gistit_fetch:
  # Default colorscheme to preview gistits on your terminal.
  # Same colorschemes supported as `gistit_send`.
  colorscheme: "ansi"

  # Automatically save fetched gistits to local fs.
  # Save location is specified in `global > save_location`
  # (setting this flag will stop asking behavior)
  save: false

  # Automatically preview a fetched gistit.
  # This will launch bat to preview the file in your terminal.
  # (setting this flag will stop asking behavior)
  preview: false
"#;
