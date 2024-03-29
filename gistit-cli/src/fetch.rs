use std::path::PathBuf;

use async_trait::async_trait;
use clap::ArgMatches;
use console::style;
use reqwest::StatusCode;
use serde::Serialize;

use gistit_proto::ipc::{self, Instruction};
use gistit_proto::payload::Gistit;
use gistit_proto::prost::Message;

use gistit_project::path;

use crate::dispatch::Dispatch;
use crate::file::File;
use crate::param::check;
use crate::server::SERVER_URL_GET;
use crate::{errorln, finish, interruptln, progress, updateln, warnln, Error, Result};

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
            colorscheme: args
                .value_of("colorscheme")
                .unwrap_or("Monokai Extended Origin"), // This is the most decent looking
            save: args.is_present("save"),
        }))
    }
}

#[derive(Debug, Serialize)]
pub struct Config {
    hash: &'static str,
    colorscheme: &'static str,
    save: bool,
    runtime_path: PathBuf,
    config_path: PathBuf,
    data_path: PathBuf,
}

impl TryFrom<&Config> for Gistit {
    type Error = Error;

    fn try_from(value: &Config) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            hash: value.hash.to_owned(),
            ..Self::default()
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
            runtime_path: path::runtime()?,
            config_path: path::config()?,
            data_path: path::data()?,
        })
    }

    async fn dispatch(&self, config: Self::InnerData) -> Result<()> {
        progress!("Fetching");
        let mut bridge = gistit_ipc::client(&config.runtime_path)?;

        if bridge.alive() {
            warnln!("gistit-daemon running, looking in the DHT");
            bridge.connect_blocking()?;
            bridge
                .send(Instruction::request_fetch(self.hash.to_owned()))
                .await?;

            if let ipc::instruction::Kind::FetchResponse(ipc::instruction::FetchResponse {
                gistit: Some(gistit),
            }) = bridge.recv().await?.expect_response()?
            {
                preview_or_save(&gistit, self.save, &config)?;
            } else {
                interruptln!();
                errorln!("gistit hash not found");
            }
        } else {
            let gistit: Gistit = (&config).try_into()?;

            let response = reqwest::Client::new()
                .post(SERVER_URL_GET.to_string())
                .header("content-type", "application/x-protobuf")
                .body(gistit.encode_to_vec())
                .send()
                .await?;
            updateln!("Fetched");

            match response.status() {
                StatusCode::OK => {
                    let gistit = Gistit::from_bytes(response.bytes().await?)?;
                    preview_or_save(&gistit, self.save, &config)?;
                }
                StatusCode::NOT_FOUND => {
                    return Err(Error::Server("gistit hash not found"));
                }
                _ => return Err(Error::Server("unexpected response")),
            }
        }

        Ok(())
    }
}

pub fn preview_or_save(gistit: &Gistit, save: bool, config: &Config) -> Result<()> {
    // NOTE: Currently we support one file
    let inner = gistit.inner.first().expect("to have at least one file");
    let mut file = File::from_data(&inner.data, &inner.name)?;
    let save_location = &config.data_path;

    if save {
        let file_path = save_location.join(file.name());
        file.save_as(&file_path)?;

        warnln!("gistit saved at: `{}`", file_path.to_string_lossy());
        finish!("💾  Saved");
    } else {
        finish!("👀  Preview");
        let mut header_string = style(&inner.name).green().to_string();
        header_string.push_str(&format!(" | {}", style(&gistit.author).blue().bold()));

        if let Some(ref description) = gistit.description {
            header_string.push_str(&format!(" | {}", style(description).italic()));
        }

        let input = bat::Input::from_reader(&*file)
            .name(&inner.name)
            .title(header_string);

        bat::PrettyPrinter::new()
            .header(true)
            .grid(true)
            .input(input)
            .line_numbers(true)
            .theme(config.colorscheme)
            .use_italics(true)
            .paging_mode(bat::PagingMode::QuitIfOneScreen)
            .print()?;
    }
    Ok(())
}
