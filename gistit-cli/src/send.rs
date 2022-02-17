use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use clap::ArgMatches;
use console::style;
use reqwest::StatusCode;

use gistit_proto::payload::{hash, Gistit};
use gistit_proto::prost::Message;
use gistit_proto::{ipc, Instruction};

use gistit_project::path;

use crate::clipboard::Clipboard;
use crate::file::File;
use crate::github::{self, CreateResponse, GITHUB_GISTS_API_URL};
use crate::server::SERVER_URL_LOAD;
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
    runtime_path: PathBuf,
}

impl TryFrom<Config> for Gistit {
    type Error = Error;

    #[allow(clippy::cast_possible_truncation)]
    fn try_from(value: Config) -> std::result::Result<Self, Self::Error> {
        let data = value.file.read()?;
        let hash = hash(value.author, value.description, &data);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Check your system time")
            .as_millis()
            .to_string();

        let inner = Self::new_inner(
            value.file.name(),
            value.file.lang().to_owned(),
            value.file.size() as u32,
            data,
        );

        let gistit = Self::new(
            hash,
            value.author.to_owned(),
            value.description.map(ToOwned::to_owned),
            now,
            vec![inner],
        );

        Ok(gistit)
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
            File::from_data(stdin, "stdin")?
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
                    gistit_project::path::config()?.to_string_lossy()
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
            runtime_path: path::runtime()?,
        })
    }

    #[allow(clippy::too_many_lines)]
    async fn dispatch(&self, config: Self::InnerData) -> Result<()> {
        let clipboard = config.clipboard;

        let mut bridge = gistit_ipc::client(&config.runtime_path)?;
        if bridge.alive() {
            // Daemon is running, hosting with p2p
            progress!("Hosting");
            let gistit: Gistit = config.try_into()?;

            bridge.connect_blocking()?;
            bridge.send(Instruction::request_provide(gistit)).await?;

            if let ipc::instruction::Kind::ProvideResponse(ipc::instruction::ProvideResponse {
                hash: Some(hash),
            }) = bridge.recv().await?.expect_response()?
            {
                if clipboard {
                    Clipboard::new(&hash)
                        .try_into_selected()?
                        .into_provider()
                        .set_contents()?;
                }

                let clipboard_msg = if self.clipboard {
                    style("(copied to clipboard)").italic().dim().to_string()
                } else {
                    "".to_string()
                };

                updateln!("Hosted");
                finish!(format!(
                    "\n    hash: '{}' {}\n\n",
                    style(hash).bold(),
                    style(clipboard_msg).italic().dim()
                ));
            } else {
                interruptln!();
                errorln!("failed to provide gistit, check gistit-daemon logs");
            }
        } else {
            progress!("Sending");
            let maybe_github_token = config.github_token.as_ref().map(Clone::clone);
            let gistit: Gistit = config.try_into()?;

            let maybe_gist = if let Some(token) = maybe_github_token {
                // Github flag was provided, sending to Github Gists
                // NOTE: Currently we only support one file
                let inner = gistit.inner.first().expect("to have at least one file");
                let name = &inner.name;
                let description = gistit.description.as_deref().unwrap_or("");

                let response = reqwest::Client::new()
                    .post(GITHUB_GISTS_API_URL)
                    .header("user-agent", "gistit")
                    .header("authorization", format!("token {}", token.access_token))
                    .header("accept", "application/vnd.github.v3+json")
                    .json(&serde_json::json!({
                        "description": description,
                        "public": true,
                        "files": {
                            name: {
                                "content": inner.data
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

            let response = reqwest::Client::new()
                .post(SERVER_URL_LOAD.to_string())
                .header("content-type", "application/x-protobuf")
                .body(gistit.encode_to_vec())
                .send()
                .await?;

            match response.status() {
                StatusCode::OK => {
                    let server_hash = Gistit::from_bytes(response.bytes().await?)?.hash;

                    if clipboard {
                        Clipboard::new(&server_hash)
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
                        "\n    hash: '{}' {} \n    url: 'https://gistit.vercel.app/h/{}' {}\n\n",
                        style(&server_hash).bold(),
                        clipboard_msg,
                        style(&server_hash).bold(),
                        gist
                    ));
                }
                StatusCode::UNPROCESSABLE_ENTITY | StatusCode::BAD_REQUEST => {
                    return Err(Error::Server("invalid gistit payload"));
                }
                _ => return Err(Error::Server("invalid server response")),
            }
        };
        Ok(())
    }
}
