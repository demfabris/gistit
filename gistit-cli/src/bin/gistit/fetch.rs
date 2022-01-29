use async_trait::async_trait;
use clap::ArgMatches;
use console::style;
use reqwest::StatusCode;
use serde::Serialize;

use gistit_ipc::{self, Instruction};

use libgistit::file::File;
use libgistit::project::runtime_dir;
use libgistit::server::{Gistit, IntoGistit, Response, SERVER_URL_GET};

use crate::dispatch::Dispatch;
use crate::param::check;
use crate::{prettyln, ErrorKind, Result};

#[derive(Debug, Clone)]
pub struct Action {
    pub hash: &'static str,
    pub colorscheme: &'static str,
    pub save: bool,
}

impl Action {
    pub fn from_args(
        args: &'static ArgMatches,
    ) -> Result<Box<dyn Dispatch<InnerData = Config> + Send + Sync + 'static>> {
        Ok(Box::new(Self {
            hash: args.value_of("HASH").ok_or(ErrorKind::Argument)?,
            colorscheme: args.value_of("colorscheme").unwrap_or("ansi"),
            save: args.is_present("save"),
        }))
    }
}

#[derive(Debug, Serialize)]
pub struct Config {
    hash: &'static str,
    colorscheme: &'static str,
    save: bool,
}

impl IntoGistit for Config {
    fn into_gistit(self) -> Result<Gistit> {
        Ok(Gistit {
            hash: self.hash.to_owned(),
            ..Gistit::default()
        })
    }
}

#[async_trait]
impl Dispatch for Action {
    type InnerData = Config;

    async fn prepare(&self) -> Result<Self::InnerData> {
        let hash = check::hash(self.hash)?;

        let colorscheme = check::colorscheme(self.colorscheme)?;

        Ok(Config {
            hash,
            colorscheme,
            save: self.save,
        })
    }

    async fn dispatch(&self, config: Self::InnerData) -> Result<()> {
        let runtime_dir = runtime_dir()?;
        let hash = config.hash;
        let mut bridge = ipc::client(&runtime_dir)?;

        if bridge.alive() {
            bridge.connect_blocking()?;
            bridge
                .send(Instruction::Get {
                    hash: config.hash.to_owned(),
                })
                .await?;
        } else {
            prettyln!("Contacting host...");

            let req = reqwest::Client::new()
                .post(SERVER_URL_GET)
                .json(&config.into_gistit()?)
                .send()
                .await?;

            match req.status() {
                StatusCode::OK => {
                    let payload = req.json::<Response>().await?.into_gistit()?;
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

fn preview_gistit(action: &Action, gistit: &Gistit, file: &File) -> Result<bool> {
    let mut header_string = style(&gistit.inner.name).green().to_string();
    header_string.push_str(&format!(" | {}", style(&gistit.author).blue().bold()));

    if let Some(ref description) = gistit.description {
        header_string.push_str(&format!(" | {}", style(description).italic()));
    }

    let colorscheme = action.colorscheme;

    let input = bat::Input::from_reader(file.data())
        .name(&gistit.inner.name)
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
