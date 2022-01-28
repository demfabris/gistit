use async_trait::async_trait;
use clap::ArgMatches;
use console::style;
use lazy_static::lazy_static;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use url::Url;

use lib_gistit::file::File;
use lib_gistit::ipc::{self, Instruction};

use crate::dispatch::{get_runtime_dir, Dispatch, GistitPayload};
use crate::params::Check;
use crate::{prettyln, ErrorKind, Result};

lazy_static! {
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
    pub hash: &'static str,
    pub colorscheme: Option<&'static str>,
    pub save: bool,
}

impl Action {
    pub fn from_args(
        args: &'static ArgMatches,
    ) -> Result<Box<dyn Dispatch<InnerData = Config> + Send + Sync + 'static>> {
        Ok(Box::new(Self {
            hash: args.value_of("HASH").ok_or(ErrorKind::Argument)?,
            colorscheme: args.value_of("colorscheme"),
            save: args.is_present("save"),
        }))
    }
}

#[derive(Debug, Serialize)]
pub struct Config {
    hash: &'static str,
}

impl Config {
    fn into_json(self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
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

#[async_trait]
impl Dispatch for Action {
    type InnerData = Config;

    async fn prepare(&'static self) -> Result<Self::InnerData> {
        <Self as Check>::check(self)?;
        let config = Config { hash: self.hash };
        Ok(config)
    }

    async fn dispatch(&'static self, config: Self::InnerData) -> Result<()> {
        let runtime_dir = get_runtime_dir()?;
        let mut bridge = ipc::client(&runtime_dir)?;
        let hash = config.hash;

        if bridge.alive() {
            bridge.connect_blocking()?;
            bridge
                .send(Instruction::Get {
                    hash: hash.to_owned(),
                })
                .await?;
        } else {
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
                }
                StatusCode::NOT_FOUND => return Err(ErrorKind::FetchNotFound.into()),
                _ => return Err(ErrorKind::FetchUnexpectedResponse.into()),
            }
        }

        println!(
            r#"
SUCCESS:
    hash: {}

You can preview it online at: {}/{}
"#,
            style(&hash).blue().bold(),
            "https://gistit.vercel.app",
            style(&hash).blue().bold(),
        );
        Ok(())
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
    let save_location = std::env::temp_dir(); // TODO: improve this

    let file_path = save_location.join(file.name());
    file.save_as(&file_path)?;
    Ok(())
}
