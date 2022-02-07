use std::path::PathBuf;

use async_trait::async_trait;
use clap::ArgMatches;
use console::style;
use reqwest::StatusCode;
use serde::Serialize;

use gistit_ipc::{self, Instruction};
use gistit_reference::dir::{data_dir, runtime_dir};
use gistit_reference::Gistit;

use libgistit::file::File;
use libgistit::server::{IntoGistit, Response, SERVER_URL_GET};

use crate::dispatch::Dispatch;
use crate::param::check;
use crate::{finish, progress, updateln, warnln, Error, Result};

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
            hash: args
                .value_of("HASH")
                .ok_or(Error::Argument("missing arugment", "--hash"))?,
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
        progress!("Preparing");
        let hash = check::hash(self.hash)?;
        let colorscheme = check::colorscheme(self.colorscheme)?;
        updateln!("Prepared");

        Ok(Config {
            hash,
            colorscheme,
            save: self.save,
        })
    }

    async fn dispatch(&self, config: Self::InnerData) -> Result<()> {
        progress!("Fetching");
        let runtime_dir = runtime_dir()?;
        let mut bridge = gistit_ipc::client(&runtime_dir)?;

        if bridge.alive() {
            bridge.connect_blocking()?;
            bridge
                .send(Instruction::Get {
                    hash: config.hash.to_owned(),
                })
                .await?;
        } else {
            let response = reqwest::Client::new()
                .post(SERVER_URL_GET.to_string())
                .json(&config.into_gistit()?)
                .send()
                .await?;
            updateln!("Fetched");

            match response.status() {
                StatusCode::OK => {
                    let gistit = response.json::<Response>().await?.into_gistit()?;
                    let file = File::from_bytes_encoded(gistit.data(), gistit.name())?;

                    if self.save {
                        let saved_file = save_gistit(&file)?;
                        warnln!("gistit saved at: `{}`", saved_file.to_string_lossy());
                        finish!("💾  Saved");
                    } else {
                        finish!("👀  Preview");
                        preview_gistit(self, &gistit, &file)?;
                    }
                }
                StatusCode::NOT_FOUND => {
                    return Err(Error::Server("gistit hash not found".to_owned()));
                }
                _ => return Err(Error::Server("unexpected response".to_owned())),
            }
        }

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

fn save_gistit(file: &File) -> Result<PathBuf> {
    let save_location = data_dir()?;

    let file_path = save_location.join(file.name());
    file.save_as(&file_path)?;
    Ok(file_path)
}
