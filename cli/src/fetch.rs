//! The Fetch module
use async_trait::async_trait;
use clap::ArgMatches;

use crate::dispatch::Dispatch;
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
    ) -> Result<Box<dyn Dispatch<InnerData = Payload> + 'act>> {
        Ok(Box::new(Self {
            hash: args.value_of("hash"),
            url: args.value_of("url"),
            secret: args.value_of("secret"),
            colorscheme: args.value_of("theme").ok_or(Error::Argument)?,
            no_syntax_highlighting: args.is_present("no-syntax-highlighting"),
        }))
    }
}

pub struct Payload;

#[async_trait]
impl Dispatch for Action<'_> {
    type InnerData = Payload;

    async fn prepare(&self) -> Result<Self::InnerData> {
        todo!()
    }

    async fn dispatch(&self, _payload: Self::InnerData) -> Result<()> {
        todo!()
    }
}
