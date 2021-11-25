//! The Fetch module
use async_trait::async_trait;
use clap::ArgMatches;

use crate::dispatch::Dispatch;
use crate::encrypt::Secret;
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

#[derive(Default)]
pub struct Config {
    pub params: Option<FetchParams>,
}

impl Config {
    /// Trivially initialize config structure
    #[must_use]
    pub fn with_none() -> Self {
        Self::default()
    }

    /// Append a checked instance of [`Params`].
    pub fn with_params(&mut self, params: FetchParams) -> &mut Self {
        self.params = Some(params);
        self
    }
}

#[async_trait]
impl Dispatch for Action<'_> {
    type InnerData = Config;

    async fn prepare(&self) -> Result<Self::InnerData> {
        let mut config = Config::with_none();
        let params = Params::from_fetch(self)?.check_consume()?;
        if let Some(secret_str) = self.secret {
            Secret::new(secret_str).check_consume()?;
        }
        config.with_params(params);
        Ok(config)
    }

    async fn dispatch(&self, _config: Self::InnerData) -> Result<()> {
        Ok(())
    }
}
