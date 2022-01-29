use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use clap::ArgMatches;
use console::style;
use gistit_ipc::{self, Instruction};

use libgistit::clipboard::Clipboard;
use libgistit::file::File;
use libgistit::hash::Hasheable;
use libgistit::project::runtime_dir;
use libgistit::server::{Gistit, Inner, IntoGistit, Response, SERVER_URL_LOAD};

use crate::dispatch::Dispatch;
use crate::param::check;
use crate::{prettyln, Error, Result};

#[derive(Debug, Clone)]
pub struct Action {
    pub file_path: Option<&'static OsStr>,
    pub maybe_stdin: Option<String>,
    pub description: Option<&'static str>,
    pub author: &'static str,
    pub clipboard: bool,
}

impl Action {
    pub fn from_args(
        args: &'static ArgMatches,
        maybe_stdin: Option<String>,
    ) -> Result<Box<dyn Dispatch<InnerData = Config> + Send + Sync + 'static>> {
        prettyln!("Preparing gistit...",);
        Ok(Box::new(Self {
            file_path: args.value_of_os("FILE"),
            maybe_stdin,
            description: args.value_of("description"),
            author: args
                .value_of("author")
                .ok_or(Error::Argument("missing argument", "--author"))?,
            clipboard: args.is_present("clipboard"),
        }))
    }
}

#[derive(Debug)]
pub struct Config {
    file: File,
    author: &'static str,
    description: Option<&'static str>,
    clipboard: bool,
}

impl Hasheable for Config {
    fn hash(&self) -> String {
        let to_digest = [
            self.file.data(),
            self.author.as_bytes(),
            self.description.unwrap_or("").as_bytes(),
        ];

        let mut md5 = md5::Context::new();
        for digest in to_digest {
            md5.consume(digest);
        }

        format!("{:x}", md5.compute())
    }
}

impl IntoGistit for Config {
    fn into_gistit(self) -> Result<Gistit> {
        Ok(Gistit {
            hash: self.hash(),
            author: self.author.to_owned(),
            description: self.description.map(ToOwned::to_owned),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Check your system time")
                .as_millis()
                .to_string(),
            inner: Inner {
                name: self.file.name(),
                lang: self.file.lang().to_owned(),
                size: self.file.size(),
                data: self.file.to_encoded_data(),
            },
        })
    }
}

#[async_trait]
impl Dispatch for Action {
    type InnerData = Config;

    async fn prepare(&self) -> Result<Self::InnerData> {
        let file = if let Some(file_ostr) = self.file_path {
            let path = Path::new(file_ostr);
            let attr = fs::metadata(&path)?;
            let maybe_extension = path.extension();

            check::metadata(&attr)?;
            check::extension(maybe_extension)?;

            File::from_path(path)?
        } else if let Some(ref stdin) = self.maybe_stdin {
            File::from_bytes(stdin.as_bytes().to_vec(), "stdin")?
        } else {
            return Err(Error::Argument("missing file input", "[FILE]/[STDIN]"));
        };

        let author = check::author(self.author)?;

        let description = if let Some(value) = self.description {
            Some(check::description(value)?)
        } else {
            None
        };

        Ok(Config {
            file,
            description,
            author,
            clipboard: self.clipboard,
        })
    }

    async fn dispatch(&self, config: Self::InnerData) -> Result<()> {
        let runtime_dir = runtime_dir()?;
        let hash = config.hash();
        let clipboard = config.clipboard;

        let mut bridge = gistit_ipc::client(&runtime_dir)?;
        if bridge.alive() {
            prettyln!("Hosting gistit...");
            bridge.connect_blocking()?;
            bridge.send(Instruction::Provide {
                hash: hash.clone(),
                // data: config.file.to_encoded_data(),
                data: Vec::new(),
            })?;
        } else {
            prettyln!("Uploading to server...");
            let response: Response = reqwest::Client::new()
                .post(SERVER_URL_LOAD)
                .json(&config.into_gistit()?)
                .send()
                .await?
                .json()
                .await?;
            response.into_gistit()?;

            if clipboard {
                Clipboard::new(hash.clone())
                    .try_into_selected()?
                    .into_provider()
                    .set_contents()?;
            }
        };

        println!(
            r#"
SUCCESS:
    hash: {} {}
    url: {}{}
            "#,
            style(&hash).bold().blue(),
            if self.clipboard {
                style("(copied to clipboard)").italic().to_string()
            } else {
                "".to_string()
            },
            "https://gistit.vercel.app/",
            style(&hash).bold().blue()
        );
        Ok(())
    }
}
