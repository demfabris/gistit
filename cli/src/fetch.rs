//! The Fetch module
use async_trait::async_trait;
use clap::ArgMatches;
use serde_json::json;
use url::Url;

use crate::dispatch::Dispatch;
use crate::encrypt::Secret;
use crate::errors::params::ParamsError;
use crate::params::{FetchParams, Params};
use crate::{Error, Result};

pub struct Action<'a> {
    pub hash: Option<&'a str>,
    pub url: Option<&'a str>,
    pub secret: Option<&'a str>,
    pub colorscheme: &'a str,
    pub no_syntax_highlighting: bool,
}

impl<'act, 'args> Action<'act> {
    /// Parse [`ArgMatches`] into the dispatchable Fetch action
    ///
    /// # Errors
    ///
    /// Fails with argument errors
    pub fn from_args(
        args: &'act ArgMatches<'args>,
    ) -> Result<Box<dyn Dispatch<InnerData = Config> + 'act>> {
        Ok(Box::new(Self {
            hash: args.value_of("hash"),
            url: args.value_of("url"),
            secret: args.value_of("secret"),
            colorscheme: args.value_of("theme").ok_or(Error::Argument)?,
            no_syntax_highlighting: args.is_present("no-syntax-highlighting"),
        }))
    }
}

pub struct Config {
    pub params: FetchParams,
    pub maybe_secret: Option<String>,
}

impl Config {
    /// Trivially initialize config structure
    #[must_use]
    const fn new(params: FetchParams, maybe_secret: Option<String>) -> Self {
        Self {
            params,
            maybe_secret,
        }
    }

    /// Converts `gistit-fetch` [`Config`] into json.
    /// If input is a URL it extracts the hash and it's safe to grab it
    /// directly from `url.path()` because it was previously checked to be valid.
    ///
    /// # Errors
    ///
    /// Fails with [`InvalidUrl`] error
    fn into_json(self) -> Result<serde_json::Value> {
        let final_hash = match &self.params {
            FetchParams {
                hash: Some(hash), ..
            } => hash.clone(),
            FetchParams {
                url: Some(url),
                hash: None,
                ..
            } => Url::parse(url)
                .map_err(|err| ParamsError::InvalidUrl(err.to_string()))?
                .path()
                // Removing `/` prefix from URL parsing
                .split_at(1)
                .1
                .to_owned(),
            _ => unreachable!(),
        };
        Ok(json!({
            "hash": final_hash,
            "secret": self.maybe_secret,
        }))
    }
}

#[async_trait]
impl Dispatch for Action<'_> {
    type InnerData = Config;

    async fn prepare(&self) -> Result<Self::InnerData> {
        let params = Params::from_fetch(self)?.check_consume()?;
        if let Some(secret_str) = self.secret {
            Secret::new(secret_str).check_consume()?;
        }
        let config = Config::new(params, self.secret.map(ToOwned::to_owned));
        Ok(config)
    }

    async fn dispatch(&self, config: Self::InnerData) -> Result<()> {
        let json = config.into_json()?;
        dbg!(json);
        Ok(())
    }
}
