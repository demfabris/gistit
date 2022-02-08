use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use clap::ArgMatches;
use console::style;
use reqwest::StatusCode;

use gistit_ipc::{self, Instruction, ServerResponse};
use gistit_reference::dir;
use gistit_reference::{Gistit, Inner};

use libgistit::clipboard::Clipboard;
use libgistit::file::File;
use libgistit::github::{self, CreateResponse, GITHUB_GISTS_API_URL};
use libgistit::hash::Hasheable;
use libgistit::server::{IntoGistit, Response, SERVER_URL_LOAD};

use crate::dispatch::Dispatch;
use crate::param::check;
use crate::{errorln, finish, interruptln, progress, updateln, warnln, Error, Result};

#[derive(Debug, Clone)]
pub struct Action {
    pub file_path: Option<&'static OsStr>,
    pub maybe_stdin: Option<String>,
    pub description: Option<&'static str>,
    pub author: &'static str,
    pub clipboard: bool,
    pub github: bool,
}

impl Action {
    pub fn from_args(
        args: &'static ArgMatches,
        maybe_stdin: Option<String>,
    ) -> Result<Box<dyn Dispatch<InnerData = Config> + Send + Sync + 'static>> {
        Ok(Box::new(Self {
            file_path: args.value_of_os("FILE"),
            maybe_stdin,
            description: args.value_of("description"),
            author: args
                .value_of("author")
                .ok_or(Error::Argument("missing argument", "--author"))?,
            clipboard: args.is_present("clipboard"),
            github: args.is_present("github"),
        }))
    }
}

#[derive(Debug)]
pub struct Config {
    file: File,
    author: &'static str,
    description: Option<&'static str>,
    clipboard: bool,
    github_token: Option<github::Token>,
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
        progress!("Preparing");
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
        updateln!("Prepared");

        let github_token = if self.github {
            progress!("Authorizing");
            let mut oauth = github::Oauth::new()?;

            if oauth.token().is_none() {
                if let Err(url) = oauth.authorize() {
                    warnln!(
                        "failed to open your web browser. \n\nAuthorize manually: '{}'",
                        style(url).cyan()
                    );
                }
                oauth.poll_token().await?;
                warnln!(
                    "storing github token at: '{}'",
                    dir::config()?.to_string_lossy()
                );
            }
            updateln!("Authorized");
            oauth.token
        } else {
            None
        };

        Ok(Config {
            file,
            description,
            author,
            clipboard: self.clipboard,
            github_token,
        })
    }

    #[allow(clippy::too_many_lines)]
    async fn dispatch(&self, config: Self::InnerData) -> Result<()> {
        let hash = config.hash();

        let runtime_dir = dir::runtime()?;
        let clipboard = config.clipboard;

        let mut bridge = gistit_ipc::client(&runtime_dir)?;
        if bridge.alive() {
            progress!("Hosting");

            bridge.connect_blocking()?;
            bridge
                .send(Instruction::Provide {
                    hash: hash.clone(),
                    data: config.into_gistit()?,
                })
                .await?;

            if let Instruction::Response(ServerResponse::Provide(Some(hash))) =
                bridge.recv().await?
            {
                updateln!("Hosted");
                finish!(format!("\n    hash: '{}'\n\n", style(hash).bold()));
            } else {
                interruptln!();
                errorln!("failed to provide gistit, check gistit-daemon logs");
            }
        } else {
            progress!("Sending");
            let maybe_github_token = config.github_token.as_ref().map(Clone::clone);

            let maybe_gist = if let Some(token) = maybe_github_token {
                let name = config.file.name();
                let description = config.description.unwrap_or("");
                let data = str::from_utf8(config.file.data())?;

                let response = reqwest::Client::new()
                    .post(GITHUB_GISTS_API_URL)
                    .header("User-Agent", "gistit")
                    .header("Authorization", format!("token {}", token.access_token))
                    .header("Accept", "application/vnd.github.v3+json")
                    .json(&serde_json::json!({
                        "description": description,
                        "public": true,
                        "files": {
                            name: {
                                "content": data
                            }
                        }
                    }))
                    .send()
                    .await?;

                match response.status() {
                    StatusCode::CREATED => {
                        let data: CreateResponse = response.json().await?;
                        Some(data.url)
                    }
                    StatusCode::FORBIDDEN | StatusCode::UNPROCESSABLE_ENTITY => {
                        warnln!(
                            "your github token is expired, nothing was posted. status {}",
                            response.status()
                        );
                        None
                    }
                    _ => {
                        warnln!("got a invalid response from github, nothing was posted");
                        None
                    }
                }
            } else {
                None
            };

            let gistit = config.into_gistit()?;
            let response: Response = reqwest::Client::new()
                .post(SERVER_URL_LOAD.to_string())
                .json(&gistit)
                .send()
                .await?
                .json()
                .await?;
            let server_hash = response.into_gistit()?.hash;

            if clipboard {
                Clipboard::new(server_hash)
                    .try_into_selected()?
                    .into_provider()
                    .set_contents()?;
            }
            updateln!("Sent");

            let clipboard_msg = if self.clipboard {
                style("(copied to clipboard)").italic().dim().to_string()
            } else {
                "".to_string()
            };

            let gist = maybe_gist.map_or_else(
                || "".to_string(),
                |gist_url| format!("github gist: '{}'\n", gist_url),
            );

            finish!(format!(
                r#"
    hash: '{}' {}
    url: 'https://gistit.vercel.app/h/{}'
    {}      "#,
                style(&hash).bold(),
                clipboard_msg,
                style(&hash).bold(),
                gist
            ));
        };

        Ok(())
    }
}
