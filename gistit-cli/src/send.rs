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
use lib_gistit::encrypt::{digest_md5_multi, HashedSecret, Secret};
use lib_gistit::file::{name_from_path, File, FileReady};

use crate::dispatch::{Dispatch, GistitInner, GistitPayload, Hasheable};
use crate::params::Params;
use crate::params::SendParams;
use crate::settings::{get_runtime_settings, GistitSend, Mergeable};
use crate::{gistit_line_out, ErrorKind, Result};

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
    pub secret: Option<&'static str>,
    pub clipboard: bool,
    pub dry_run: bool,
}

impl Action {
    pub fn from_args(
        args: &'static ArgMatches,
    ) -> Result<Box<dyn Dispatch<InnerData = Config> + Send + Sync + 'static>> {
        let file = args.value_of_os("file").ok_or(ErrorKind::Argument)?;
        gistit_line_out!(format!(
            "{} {}",
            style("Preparing gistit:").bold(),
            style(name_from_path(Path::new(file))).green()
        ));

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
            secret: args.value_of("secret"),
            clipboard,
            dry_run: args.is_present("dry-run"),
        }))
    }
}

pub struct Config {
    pub file: Box<dyn FileReady + Send + Sync>,
    pub params: SendParams,
    pub maybe_secret: Option<HashedSecret>,
}

#[async_trait]
impl Hasheable for Config {
    fn hash(&self) -> String {
        let file_data = self.file.data();
        let maybe_secret_bytes = self
            .maybe_secret
            .as_ref()
            .map_or("", HashedSecret::to_str)
            .as_bytes();

        let hash = digest_md5_multi(&[file_data, maybe_secret_bytes]);
        format!("{}{}", SERVER_IDENTIFIER_CHAR, hash)
    }
}

impl Config {
    #[must_use]
    fn new(
        file: Box<dyn FileReady + Send + Sync>,
        params: SendParams,
        maybe_secret: Option<HashedSecret>,
    ) -> Self {
        Self {
            file,
            params,
            maybe_secret,
        }
    }

    async fn into_payload(self) -> Result<GistitPayload> {
        let hash = self.hash();
        let params = self.params;
        let data = self.file.to_encoded_data();
        let file_ref = self.file.inner().await.expect("The file to be opened");

        Ok(GistitPayload {
            hash,
            author: params.author.to_owned(),
            description: params.description.map(ToOwned::to_owned),
            secret: self.maybe_secret.map(|t| t.to_str().to_owned()),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Check your system time")
                .as_millis()
                .to_string(),
            gistit: GistitInner {
                name: file_ref.name().clone(),
                lang: file_ref.lang().to_owned(),
                size: file_ref.size().await,
                data,
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

        let (file, maybe_hashed_secret): (Box<dyn FileReady + Send + Sync>, Option<HashedSecret>) = {
            let path = Path::new(self.file);
            let file = File::from_path(path).await?.check_consume().await?;

            if let Some(secret_str) = self.secret {
                let hashed_secret = Secret::new(secret_str).check_consume()?.into_hashed()?;
                gistit_line_out!("Encrypting...");

                let encrypted_file = file.into_encrypted(secret_str).await?;
                (Box::new(encrypted_file), Some(hashed_secret))
            } else {
                (Box::new(file), None)
            }
        };
        let config = Config::new(file, params, maybe_hashed_secret);
        Ok(config)
    }
    async fn dispatch(&'static self, config: Self::InnerData) -> Result<()> {
        if self.dry_run {
            return Ok(());
        }

        gistit_line_out!("Uploading to server...");
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
{}:
    hash: {} {}
    url: {}{}
            "#,
            "SUCCESS",
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
