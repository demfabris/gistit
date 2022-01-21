use std::ffi::OsStr;
use std::path::Path;
use std::str;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use clap::ArgMatches;
use console::style;
use lazy_static::lazy_static;
use serde::Deserialize;
use url::Url;

use lib_gistit::clipboard::Clipboard;
use lib_gistit::file::File;

use crate::dispatch::{Dispatch, GistitInner, GistitPayload, Hasheable};
use crate::params::Check;
use crate::settings::{get_runtime_settings, GistitSend, Mergeable};
use crate::{prettyln, ErrorKind, Result};

const SERVER_IDENTIFIER_CHAR: char = '#';

lazy_static! {
    static ref GISTIT_SERVER_LOAD_URL: Url = Url::parse(
        option_env!("GISTIT_SERVER_URL")
            .unwrap_or("https://us-central1-gistit-base.cloudfunctions.net")
    )
    .expect("GISTIT_SERVER_URL env variable is not valid URL")
    .join("load")
    .expect("to join 'load' function URL");
}

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

        let rhs_settings = get_runtime_settings()?.gistit_send.clone();

        let lhs_settings = Box::new(GistitSend {
            author: args.value_of("author").map(ToOwned::to_owned),
            clipboard: Some(args.is_present("clipboard")),
        });

        let merged = lhs_settings.merge(rhs_settings);
        let (author, clipboard) = (
            merged.author.ok_or(ErrorKind::Argument)?,
            merged.clipboard.ok_or(ErrorKind::Argument)?,
        );

        Ok(Box::new(Self {
            file_path: args.value_of_os("FILE"),
            maybe_stdin,
            description: args.value_of("description"),
            author: Box::leak(Box::new(author)),
            clipboard,
        }))
    }
}

#[derive(Debug)]
pub struct Config {
    file: File,
    description: Option<&'static str>,
    author: &'static str,
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

        format!("{}{:x}", SERVER_IDENTIFIER_CHAR, md5.compute())
    }
}

impl Config {
    fn into_json(self) -> Result<serde_json::Value> {
        let payload = GistitPayload {
            hash: self.hash(),
            author: self.author.to_owned(),
            description: self.description.map(ToOwned::to_owned),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Check your system time")
                .as_millis()
                .to_string(),
            gistit: GistitInner {
                name: self.file.name(),
                lang: self.file.lang().to_owned(),
                size: self.file.size(),
                data: self.file.to_encoded_data(),
            },
        };
        Ok(serde_json::to_value(payload)?)
    }
}

#[derive(Deserialize, Debug)]
struct Response {
    success: Option<String>,
    error: Option<String>,
}

impl Response {
    fn into_inner(self) -> Result<String> {
        match self {
            Self {
                success: Some(hash),
                ..
            } => Ok(hash),
            Self { error: Some(_), .. } => Err(ErrorKind::Unknown.into()),
            _ => unreachable!("Gistit server is unreachable"),
        }
    }
}

#[async_trait]
impl Dispatch for Action {
    type InnerData = Config;

    async fn prepare(&'static self) -> Result<Self::InnerData> {
        <Self as Check>::check(self)?;

        let file = if let Some(file) = self.file_path {
            File::from_path(Path::new(file))?
        } else if let Some(ref stdin) = self.maybe_stdin {
            File::from_bytes(stdin.as_bytes().to_vec(), "stdin")?
        } else {
            return Err(ErrorKind::Argument.into());
        };

        let config = Config {
            file,
            description: self.description,
            author: self.author,
        };
        Ok(config)
    }

    async fn dispatch(&'static self, config: Self::InnerData) -> Result<()> {
        prettyln!("Uploading to server...");
        let response: Response = reqwest::Client::new()
            .post(GISTIT_SERVER_LOAD_URL.to_string())
            .json(&config.into_json()?)
            .send()
            .await?
            .json()
            .await?;

        let server_hash = response.into_inner()?;
        if self.clipboard {
            Clipboard::new(server_hash.clone())
                .try_into_selected()?
                .into_provider()
                .set_contents()?;
        }

        println!(
            r#"
SUCCESS:
    hash: {} {}
    url: {}{}
            "#,
            style(&server_hash).bold().blue(),
            if self.clipboard {
                style("(copied to clipboard)").italic().to_string()
            } else {
                "".to_string()
            },
            "https://gistit.vercel.app/",
            style(&server_hash).bold().blue()
        );
        Ok(())
    }
}
