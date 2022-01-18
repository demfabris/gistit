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
use lib_gistit::file::{name_from_path, File};

use crate::dispatch::{Dispatch, GistitInner, GistitPayload, Hasheable};
use crate::params::Params;
use crate::params::SendParams;
use crate::settings::{get_runtime_settings, GistitSend, Mergeable};
use crate::{prettyln, warnln, ErrorKind, Result};

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
    pub file: &'static OsStr,
    pub description: Option<&'static str>,
    pub author: &'static str,
    pub clipboard: bool,
    pub dry_run: bool,
}

impl Action {
    pub fn from_args(
        args: &'static ArgMatches,
    ) -> Result<Box<dyn Dispatch<InnerData = Config> + Send + Sync + 'static>> {
        let file = args.value_of_os("file").ok_or(ErrorKind::Argument)?;
        prettyln!(
            "Preparing gistit: {}",
            style(name_from_path(Path::new(file))).green()
        );

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
            file,
            description: args.value_of("description"),
            author: Box::leak(Box::new(author)),
            clipboard,
            dry_run: args.is_present("dry-run"),
        }))
    }
}

pub struct Config {
    pub file: File,
    pub params: SendParams,
}

impl Hasheable for Config {
    fn hash(&self) -> String {
        let to_digest = [
            self.file.data(),
            self.params.author.as_bytes(),
            self.params.description.unwrap_or("").as_bytes(),
        ];

        let mut md5 = md5::Context::new();
        to_digest.iter().map(|data| {
            md5.consume(data);
        });

        format!("{}{:x}", SERVER_IDENTIFIER_CHAR, md5.compute())
    }
}

impl Config {
    #[must_use]
    fn new(file: File, params: SendParams) -> Self {
        Self { file, params }
    }

    async fn into_payload(self) -> Result<GistitPayload> {
        Ok(GistitPayload {
            hash: self.hash(),
            author: self.params.author.to_owned(),
            description: self.params.description.map(ToOwned::to_owned),
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
        })
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
        let params = Params::from_send(self)?.check_consume()?;

        let path = Path::new(self.file);
        let file = File::from_path(path)?;

        let config = Config::new(file, params);
        Ok(config)
    }

    async fn dispatch(&'static self, config: Self::InnerData) -> Result<()> {
        if self.dry_run {
            warnln!("Dry-run mode, exiting...");
            return Ok(());
        }

        prettyln!("Uploading to server...");
        let payload = config.into_payload().await?;
        let response: Response = reqwest::Client::new()
            .post(GISTIT_SERVER_LOAD_URL.to_string())
            .json(&payload)
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
