use std::sync::atomic::AtomicU8;

use async_trait::async_trait;
use clap::ArgMatches;
use console::style;
use lazy_static::lazy_static;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;
use url::Url;

use lib_gistit::file::File;

use crate::dispatch::{Dispatch, GistitPayload};
use crate::params::Check;
use crate::settings::{get_runtime_settings, GistitFetch, Mergeable};
use crate::{prettyln, ErrorKind, Result};

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

#[derive(Debug, Clone)]
pub struct Action {
    pub hash: Option<&'static str>,
    pub url: Option<&'static str>,
    pub colorscheme: Option<&'static str>,
    pub save: bool,
}

impl Action {
    pub fn from_args(
        args: &'static ArgMatches,
    ) -> Result<Box<dyn Dispatch<InnerData = Config> + Send + Sync + 'static>> {
        let rhs_settings = get_runtime_settings()?.gistit_fetch.clone();

        let lhs_settings = Box::new(GistitFetch {
            colorscheme: args.value_of("colorscheme").map(ToOwned::to_owned),
            save: Some(args.is_present("save")),
        });

        let merged = lhs_settings.merge(rhs_settings);
        let (colorscheme, save) = (
            merged.colorscheme.ok_or(ErrorKind::Argument)?,
            merged.save.ok_or(ErrorKind::Argument)?,
        );

        Ok(Box::new(Self {
            hash: args.value_of("hash"),
            url: args.value_of("url"),
            colorscheme: Some(Box::leak(Box::new(colorscheme))),
            save,
        }))
    }
}

#[derive(Debug)]
pub struct Config {
    hash: Option<&'static str>,
    url: Option<&'static str>,
}

impl Config {
    fn into_json(self) -> Result<serde_json::Value> {
        let final_hash = match self {
            Self {
                hash: Some(hash), ..
            } => hash.to_string(),
            Self {
                url: Some(url),
                hash: None,
                ..
            } => Url::parse(url)?
                .path()
                // Removing `/` prefix from URL parsing
                .split_at(1)
                .1
                .to_owned(),
            _ => unreachable!(),
        };
        Ok(json!({
            "hash": final_hash,
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
            Self { error: Some(_), .. } => Err(ErrorKind::FetchUnexpectedResponse.into()),
            _ => unreachable!("Gistit server is unreachable"),
        }
    }
}

fn preview_gistit(action: &Action, payload: &GistitPayload, file: &File) -> Result<bool> {
    let mut header_string = style(&payload.gistit.name).green().to_string();
    header_string.push_str(&format!(" | {}", style(&payload.author).blue().bold()));

    if let Some(ref description) = payload.description {
        header_string.push_str(&format!(" | {}", style(description).italic()));
    }
    // If user provided colorscheme we overwrite the stored one
    let colorscheme = action.colorscheme.unwrap_or("ansi");

    let input = bat::Input::from_reader(file.data())
        .name(&payload.gistit.name)
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

fn save_gistit(file: &File) -> Result<()> {
    let save_location = get_runtime_settings()?
        .clone()
        .gistit_global
        .unwrap_or_default()
        .save_location
        .ok_or(ErrorKind::Settings)?;

    let file_path = save_location.join(file.name());
    file.save_as(&file_path)?;
    Ok(())
}

#[async_trait]
impl Dispatch for Action {
    type InnerData = Config;

    async fn prepare(&'static self) -> Result<Self::InnerData> {
        <Self as Check>::check(self)?;
        let config = Config {
            hash: self.hash,
            url: self.url,
        };
        Ok(config)
    }

    async fn dispatch(&'static self, config: Self::InnerData) -> Result<()> {
        prettyln!("Contacting host...");
        let req = reqwest::Client::new()
            .post(GISTIT_SERVER_GET_URL.to_string())
            .json(&config.into_json()?)
            .send()
            .await?;

        match req.status() {
            StatusCode::OK => {
                let response: Response = req.json().await?;
                let payload = response.into_inner()?;
                let gistit = payload.to_file()?;

                if self.save {
                    save_gistit(&gistit)?;
                } else {
                    preview_gistit(self, &payload, &gistit)?;
                }

                println!(
                    r#"
SUCCESS:
    hash: {}

You can preview it online at: {}/{}
                    "#,
                    style(&payload.hash).blue().bold(),
                    "https://gistit.vercel.app",
                    style(&payload.hash).blue().bold(),
                );
                Ok(())
            }
            StatusCode::NOT_FOUND => Err(ErrorKind::FetchNotFound.into()),
            _ => Err(ErrorKind::FetchUnexpectedResponse.into()),
        }
    }
}
