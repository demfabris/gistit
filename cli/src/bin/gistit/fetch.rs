//! The Fetch module

use std::sync::atomic::{AtomicU8, Ordering};

use async_trait::async_trait;
use clap::ArgMatches;
use console::style;
use lazy_static::lazy_static;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;
use url::Url;

use lib_gistit::encrypt::Secret;
use lib_gistit::errors::fetch::FetchError;
use lib_gistit::errors::internal::InternalError;
use lib_gistit::errors::io::IoError;
use lib_gistit::errors::params::ParamsError;
use lib_gistit::file::FileReady;
use lib_gistit::{Error, Result};

use crate::dispatch::{Dispatch, GistitPayload};
use crate::params::{FetchParams, Params};
use crate::settings::{GistitFetch, Mergeable};
use crate::{gistit_line_out, gistit_warn, LOCALFS_SETTINGS};

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

impl<'args> Action {
    /// Parse [`ArgMatches`] into the dispatchable Fetch action
    /// Here we also merge user settings while keeping this order of priority:
    /// arguments > local settings file > app defaults
    ///
    /// # Errors
    ///
    /// Fails with argument errors
    pub fn from_args(
        args: &'static ArgMatches<'args>,
    ) -> Result<Box<dyn Dispatch<InnerData = Config> + Send + Sync + 'static>> {
        let rhs_settings = LOCALFS_SETTINGS
            .get()
            .ok_or_else(|| Error::Internal(InternalError::Memory("-".to_owned())))?
            .gistit_fetch
            .clone();

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
    /// Fails with [`InvalidUrl`] error
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

#[async_trait]
impl Dispatch for Action {
    type InnerData = Config;

    async fn prepare(&self) -> Result<Self::InnerData> {
        let params = Params::from_fetch(self)?.check_consume()?;
        if let Some(secret_str) = self.secret {
            Secret::new(secret_str).check_consume()?;
        }
        let config = Config::new(params, self.secret.map(ToOwned::to_owned));
        Ok(config)
    }

    async fn dispatch(&self, config: Self::InnerData) -> Result<()> {
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

                let mut header_string = style(file.name()).green().to_string();
                header_string.push_str(&format!(" | {}", style(payload.author).blue().bold()));

                if let Some(description) = payload.description {
                    header_string.push_str(&format!(" | {}", style(description).italic()));
                }
                // If user provided colorscheme we overwrite the stored one
                let colorscheme = self
                    .colorscheme
                    .unwrap_or_else(|| payload.colorscheme.as_str());

                // TODO: branch into preview or save it local fs. wait for flags to be ready
                let input = bat::Input::from_reader(file.data())
                    .name(file.name())
                    .title(header_string);

                bat::PrettyPrinter::new()
                    .header(true)
                    .grid(true)
                    .input(input)
                    .line_numbers(true)
                    // .language(&payload.gistit.lang)
                    .theme(colorscheme)
                    .use_italics(true)
                    .paging_mode(bat::PagingMode::QuitIfOneScreen)
                    .print()
                    .unwrap();
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

                    let new_secret = dialoguer::Password::new()
                        .with_prompt(prompt_msg)
                        .interact()
                        .map_err(|err| Error::IO(IoError::StdinWrite(err.to_string())))?;
                    drop(first_try);

                    // Rebuild the action object and recurse down the same path
                    let mut action = self.clone();
                    action.secret = Some(Box::leak(Box::new(new_secret)));

                    let new_config = Dispatch::prepare(&action).await?;
                    Dispatch::dispatch(&action, new_config).await?;
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
