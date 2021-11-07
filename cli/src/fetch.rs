//! The Fetch module

use clap::ArgMatches;
use std::convert::TryFrom;

use crate::{Error, Result};

pub struct Action<'a> {
    hash: Option<&'a str>,
    url: Option<&'a str>,
    secret: Option<&'a str>,
    colorscheme: &'a str,
    no_syntax_highlighting: bool,
}

impl<'act, 'args> TryFrom<&'act ArgMatches<'args>> for Action<'act> {
    type Error = Error;

    /// Parse [`ArgMatches`] into the Fetch action or error out
    fn try_from(args: &'act ArgMatches<'args>) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            hash: args.value_of("hash"),
            url: args.value_of("url"),
            secret: args.value_of("secret"),
            colorscheme: args.value_of("theme").ok_or(Self::Error::Argument)?,
            no_syntax_highlighting: args.is_present("no-syntax-highlighting"),
        })
    }
}
