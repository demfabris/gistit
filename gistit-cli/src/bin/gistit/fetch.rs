use async_trait::async_trait;
use clap::ArgMatches;
use console::style;
use reqwest::StatusCode;
use serde::Serialize;

use gistit_proto::gistit::Gistit;
use gistit_proto::ipc::Instruction;
use gistit_proto::server::Response;
use gistit_reference::project;

use libgistit::file::File;
use libgistit::server::{IntoGistit, SERVER_URL_GET};

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
        let runtime_dir = project::path::runtime()?;
        let mut bridge = gistit_ipc::client(&runtime_dir)?;

        if bridge.alive() {
            warnln!("gistit-daemon running, looking in the DHT");
            bridge.connect_blocking()?;
            bridge
                .send(Instruction::fetch(self.hash.to_owned()))
                .await?;

            let _a = bridge.recv().await?;
            warnln!("{:?}", _a);
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
                    let mut file = File::from_data(gistit.data(), gistit.name())?;

                    if self.save {
                        let save_location = project::path::data()?;
                        let file_path = save_location.join(file.name());
                        file.save_as(&file_path)?;

                        warnln!("gistit saved at: `{}`", file_path.to_string_lossy());
                        finish!("💾  Saved");
                    } else {
                        finish!("👀  Preview");
                        let mut header_string = style(&gistit.inner.name).green().to_string();
                        header_string
                            .push_str(&format!(" | {}", style(&gistit.author).blue().bold()));

                        if let Some(ref description) = gistit.description {
                            header_string.push_str(&format!(" | {}", style(description).italic()));
                        }

                        let input = bat::Input::from_reader(&*file)
                            .name(&gistit.inner.name)
                            .title(header_string);

                        bat::PrettyPrinter::new()
                            .header(true)
                            .grid(true)
                            .input(input)
                            .line_numbers(true)
                            .theme(self.colorscheme)
                            .use_italics(true)
                            .paging_mode(bat::PagingMode::QuitIfOneScreen)
                            .print()?;
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
