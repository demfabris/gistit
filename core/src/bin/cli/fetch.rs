//! The Fetch module

use std::sync::atomic::{AtomicU8, Ordering};

use async_trait::async_trait;
use clap::ArgMatches;
use console::style;
use dialoguer::{theme::ColorfulTheme, Password, Select};
use lazy_static::lazy_static;
use lib_gistit::errors::internal::InternalError;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;
use url::Url;

use lib_gistit::encrypt::Secret;
use lib_gistit::errors::fetch::FetchError;
use lib_gistit::errors::io::IoError;
use lib_gistit::errors::params::ParamsError;
use lib_gistit::file::{File, FileReady};
use lib_gistit::{Error, Result};

use crate::dispatch::{Dispatch, GistitPayload};
use crate::params::{FetchParams, Params};
use crate::settings::{get_runtime_settings, GistitFetch, Mergeable};
use crate::{gistit_line_out, gistit_warn};

lazy_static! {
    static ref GISTIT_SECRET_RETRY_COUNT: AtomicU8 = AtomicU8::new(0);
    static ref GISTIT_SERVER_GET_URL: Url = Url::parse(
        option_env!("GISTIT_SERVER_URL")
            .unwrap_or("https://us-central1-gistit-base.cloudfunctions.net")
    )
    .expect("GISTIT_SERVER_URL env variable is not valid URL")
    .join("get")
    .expect("to join 'get' function URL");
}

/// The Fetch action runtime parameters
#[derive(Debug, Clone)]
pub struct Action {
    /// The hash to fetch
    pub hash: Option<&'static str>,
    /// The gistit url
    pub url: Option<&'static str>,
    /// The secret key to fetch and decrypt a gistit
    pub secret: Option<&'static str>,
    /// The colorscheme to highlight in preview
    pub colorscheme: Option<&'static str>,
    /// Preview with no syntax highlighting
    pub no_syntax_highlighting: bool,
    /// Immediately preview the file
    pub preview: bool,
    /// Immediately save the file to local fs
    pub save: bool,
}

impl Action {
    /// Parse [`ArgMatches`] into the dispatchable Fetch action
    /// Here we also merge user settings while keeping this order of priority:
    /// arguments > local settings file > app defaults
    ///
    /// # Errors
    ///
    /// Fails with argument errors
    pub fn from_args(
        args: &'static ArgMatches,
    ) -> Result<Box<dyn Dispatch<InnerData = Config> + Send + Sync + 'static>> {
        let rhs_settings = get_runtime_settings()?.gistit_fetch.clone();

        let lhs_settings = Box::new(GistitFetch {
            colorscheme: args.value_of("theme").map(ToOwned::to_owned),
            preview: Some(args.is_present("preview")),
            save: Some(args.is_present("save")),
        });

        let merged = lhs_settings.merge(rhs_settings);
        let (colorscheme, preview, save) = (
            merged.colorscheme.ok_or(Error::Argument)?,
            merged.preview.ok_or(Error::Argument)?,
            merged.save.ok_or(Error::Argument)?,
        );

        Ok(Box::new(Self {
            hash: args.value_of("hash"),
            url: args.value_of("url"),
            secret: args.value_of("secret"),
            no_syntax_highlighting: args.is_present("no-syntax-highlighting"),
            colorscheme: Some(Box::leak(Box::new(colorscheme))),
            preview,
            save,
        }))
    }
}

pub struct Config {
    pub params: FetchParams,
    pub maybe_secret: Option<String>,
}

impl Config {
    /// Trivially initialize config structure
    #[must_use]
    const fn new(params: FetchParams, maybe_secret: Option<String>) -> Self {
        Self {
            params,
            maybe_secret,
        }
    }

    /// Converts `gistit-fetch` [`Config`] into json.
    /// If input is a URL it extracts the hash and it's safe to grab it
    /// directly from `url.path()` because it was previously checked to be valid.
    ///
    /// # Errors
    ///
    /// Fails with [`ParamsError`] error
    fn into_json(self) -> Result<serde_json::Value> {
        let final_hash = match &self.params {
            FetchParams {
                hash: Some(hash), ..
            } => (*hash).to_string(),
            FetchParams {
                url: Some(url),
                hash: None,
                ..
            } => Url::parse(url)
                .map_err(|err| ParamsError::InvalidUrl(err.to_string()))?
                .path()
                // Removing `/` prefix from URL parsing
                .split_at(1)
                .1
                .to_owned(),
            _ => unreachable!(),
        };
        Ok(json!({
            "hash": final_hash,
            "secret": self.maybe_secret,
        }))
    }
}

#[derive(Deserialize, Debug)]
struct Response {
    success: Option<GistitPayload>,
    error: Option<String>,
}

impl Response {
    fn into_inner(self) -> Result<GistitPayload> {
        match self {
            Self {
                success: Some(payload),
                ..
            } => Ok(payload),
            Self {
                error: Some(error_msg),
                ..
            } => Err(Error::IO(IoError::Request(error_msg))),
            _ => unreachable!("Gistit server is unreachable"),
        }
    }
}

fn preview_gistit(action: &Action, payload: &GistitPayload, file: &File) -> Result<bool> {
    let mut header_string = style(file.name()).green().to_string();
    header_string.push_str(&format!(
        " | {}",
        style(payload.author.clone()).blue().bold()
    ));

    if let Some(description) = payload.description.clone() {
        header_string.push_str(&format!(" | {}", style(description).italic()));
    }
    // If user provided colorscheme we overwrite the stored one
    let colorscheme = action
        .colorscheme
        .unwrap_or_else(|| payload.colorscheme.as_str());

    let input = bat::Input::from_reader(file.data())
        .name(file.name())
        .title(header_string);

    Ok(bat::PrettyPrinter::new()
        .header(true)
        .grid(true)
        .input(input)
        .line_numbers(true)
        .theme(colorscheme)
        .use_italics(true)
        .paging_mode(bat::PagingMode::QuitIfOneScreen)
        .print()?)
}

async fn save_gistit(file: &File) -> Result<()> {
    let save_location = get_runtime_settings()?
        .clone()
        .gistit_global
        .unwrap_or_default()
        .save_location
        .ok_or_else(|| {
            Error::Internal(InternalError::Memory(
                "Couldn't read save file directory from memory".to_owned(),
            ))
        })?;
    let file_path = save_location.join(file.name());
    file.save_as(&file_path).await
}

fn print_success(hash: &str, prevent_ask_tip: bool) {
    let tip = if prevent_ask_tip {
        ""
    } else {
        "\n(You can disable the asking behavior by using one of the flags: '--save', '--preview')\n"
    };
    println!(
        r#"
{}:
    hash: {}
    url: {}{}
{}"#,
        style("SUCCESS").green(),
        style(hash).yellow(),
        style("https://gistit.vercel.app/").blue(),
        style(hash).blue(),
        style(tip).italic(),
    );
}

#[async_trait]
impl Dispatch for Action {
    type InnerData = Config;

    async fn prepare(&'static self) -> Result<Self::InnerData> {
        let params = Params::from_fetch(self)?.check_consume()?;
        if let Some(secret_str) = self.secret {
            Secret::new(secret_str).check_consume()?;
        }
        let config = Config::new(params, self.secret.map(ToOwned::to_owned));
        Ok(config)
    }

    async fn dispatch(&'static self, config: Self::InnerData) -> Result<()> {
        let json = config.into_json()?;
        // TODO: branch this into '#' and '@'
        gistit_line_out!("Contacting host...");
        let first_try = reqwest::Client::new()
            .post(GISTIT_SERVER_GET_URL.to_string())
            .json(&json)
            .send()
            .await?;

        match first_try.status() {
            StatusCode::OK => {
                let response: Response = first_try.json().await?;
                let payload = response.into_inner()?;
                let gistit = payload.to_file().await?;
                let file = gistit.inner().await.expect("File to be open");
                let prevent_ask_tip = self.preview || self.save;
                print_success(&payload.hash, prevent_ask_tip);

                if self.preview {
                    preview_gistit(self, &payload, &file)?;
                }
                if self.save {
                    save_gistit(&file).await?;
                }
                if !self.save && !self.preview {
                    // Ask
                    let choice_idx = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("Select what to do next:")
                        .item("save locally")
                        .item("preview in terminal")
                        .item("open in web browser")
                        .interact()?;
                    match choice_idx {
                        // Save locally only
                        0 => save_gistit(&file).await?,
                        // Preview with 'bat' only
                        1 => {
                            preview_gistit(self, &payload, &file)?;
                        }
                        // Open in web browser
                        2 => {
                            webbrowser::open(&format!(
                                "https://gistit.vercel.app/{}",
                                &payload.hash
                            ))
                            .expect("to open web browser");
                        }
                        _ => unreachable!(),
                    }
                }
                Ok(())
            }
            StatusCode::UNAUTHORIZED => {
                // Password is incorrect or missing. Check retry counter
                let count = GISTIT_SECRET_RETRY_COUNT.fetch_add(1, Ordering::Relaxed);
                if count <= 2 {
                    let prompt_msg = if self.secret.is_some() {
                        gistit_warn!(style("\u{1f512}Secret is invalid").yellow());
                        style("\nTry again").bold().to_string()
                    } else {
                        gistit_warn!(
                            style("\u{1f512}A secret is required for this Gistit").yellow()
                        );
                        style("\nSecret").bold().to_string()
                    };

                    let new_secret = Password::new()
                        .with_prompt(prompt_msg)
                        .interact()
                        .map_err(|err| Error::IO(IoError::StdinWrite(err.to_string())))?;
                    drop(first_try);

                    // Rebuild the action object and recurse down the same path
                    let mut action = self.clone();
                    action.secret = Some(Box::leak(Box::new(new_secret)));
                    let action = Box::leak(Box::new(action.clone()));
                    let new_config = Dispatch::prepare(&*action).await?;
                    Dispatch::dispatch(&*action, new_config).await?;
                    Ok(())
                } else {
                    // Enough retries
                    Err(Error::Fetch(FetchError::ExaustedSecretRetries))
                }
            }
            StatusCode::NOT_FOUND => Err(Error::Fetch(FetchError::NotFound)),
            _ => Err(Error::Fetch(FetchError::UnexpectedResponse)),
        }
    }
}
