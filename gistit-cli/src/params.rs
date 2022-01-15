use std::net::Ipv4Addr;
use std::ops::RangeInclusive;

use async_trait::async_trait;
use lazy_static::lazy_static;
use ngrammatic::{Corpus, CorpusBuilder, Pad};
use url::Url;

use crate::fetch::Action as FetchAction;
use crate::host::Action as HostAction;
use crate::send::Action as SendAction;
use crate::{ErrorKind, Result};

/// Allowed description length
const ALLOWED_DESCRIPTION_CHAR_LENGHT_RANGE: RangeInclusive<usize> = 10..=100;
/// Allowed author info length
const ALLOWED_AUTHOR_CHAR_LENGTH_RANGE: RangeInclusive<usize> = 3..=30;
/// Valid hash length
const GISTIT_HASH_CHAR_LENGTH: usize = 33;

pub const SUPPORTED_COLORSCHEMES: [&str; 24] = [
    "1337",
    "Coldark-Cold",
    "Coldark-Dark",
    "DarkNeon",
    "Dracula",
    "GitHub",
    "Monokai Extended",
    "Monokai Extended Bright",
    "Monokai Extended Light",
    "Monokai Extended Origin",
    "Nord",
    "OneHalfDark",
    "OneHalfLight",
    "Solarized (dark)",
    "Solarized (light)",
    "Sublime Snazzy",
    "TwoDark",
    "Visual Studio Dark+",
    "ansi",
    "base16",
    "base16-256",
    "gruvbox-dark",
    "gruvbox-light",
    "zenburn",
];

lazy_static! {
    static ref FUZZY_MATCH: Corpus = SUPPORTED_COLORSCHEMES.iter().fold(
        CorpusBuilder::new().arity(2).pad_full(Pad::Auto).finish(),
        |mut corpus, &t| {
            corpus.add_text(t);
            corpus
        },
    );
}

pub struct Params;

pub trait SendArgs {}
impl SendArgs for SendParams {}

pub trait FetchArgs {}
impl FetchArgs for FetchParams {}

pub trait HostArgs {}
impl HostArgs for HostParams {}

#[derive(Clone, Default, Debug)]
pub struct SendParams {
    pub author: &'static str,
    pub description: Option<&'static str>,
}

#[derive(Clone, Default, Debug)]
pub struct FetchParams {
    pub hash: Option<&'static str>,
    pub url: Option<&'static str>,
    pub colorscheme: Option<&'static str>,
}

#[derive(Clone, Default, Debug)]
pub struct HostParams {
    pub listen_addr: &'static str,
}

impl SendParams {
    pub fn check_consume(self) -> Result<Self> {
        <Self as Check>::description(&self)?;
        <Self as Check>::author(&self)?;
        Ok(self)
    }
}

impl FetchParams {
    pub fn check_consume(self) -> Result<Self> {
        <Self as Check>::colorscheme(&self)?;
        <Self as Check>::hash(&self)?;
        <Self as Check>::url(&self)?;
        Ok(self)
    }
}

impl HostParams {
    pub fn check_consume(self) -> Result<Self> {
        <Self as Check>::listen_addr(&self)?;
        Ok(self)
    }
}

impl Params {
    pub fn from_send(action: &SendAction) -> Result<SendParams> {
        Ok(SendParams {
            author: action.author,
            description: action.description,
        })
    }

    pub const fn from_fetch(action: &FetchAction) -> FetchParams {
        FetchParams {
            hash: action.hash,
            url: action.url,
            colorscheme: action.colorscheme,
        }
    }

    pub const fn from_host(action: &HostAction) -> HostParams {
        HostParams {
            listen_addr: action.listen,
        }
    }
}

#[async_trait]
trait Check {
    fn colorscheme(&self) -> Result<()>
    where
        Self: FetchArgs,
    {
        Ok(())
    }

    fn description(&self) -> Result<()>
    where
        Self: SendArgs,
    {
        Ok(())
    }

    fn author(&self) -> Result<()>
    where
        Self: SendArgs,
    {
        Ok(())
    }

    fn hash(&self) -> Result<()>
    where
        Self: FetchArgs,
    {
        Ok(())
    }

    fn url(&self) -> Result<()>
    where
        Self: FetchArgs,
    {
        Ok(())
    }

    fn listen_addr(&self) -> Result<()>
    where
        Self: HostArgs,
    {
        Ok(())
    }
}

#[async_trait]
impl Check for SendParams {
    fn description(&self) -> Result<()>
    where
        Self: SendArgs,
    {
        self.description.as_ref().map_or_else(
            || Ok(()),
            |value| {
                if ALLOWED_DESCRIPTION_CHAR_LENGHT_RANGE.contains(&value.len()) {
                    Ok(())
                } else {
                    Err(ErrorKind::InvalidParam(
                        "invalid description character length.",
                        "--description",
                    )
                    .into())
                }
            },
        )
    }

    fn author(&self) -> Result<()>
    where
        Self: SendArgs,
    {
        if ALLOWED_AUTHOR_CHAR_LENGTH_RANGE.contains(&self.author.len()) {
            Ok(())
        } else {
            Err(ErrorKind::InvalidParam("invalid author character length.", "--author").into())
        }
    }
}

#[async_trait]
impl Check for FetchParams {
    fn colorscheme(&self) -> Result<()> {
        self.colorscheme.map_or(Ok(()), |value| {
            if SUPPORTED_COLORSCHEMES.contains(&value) {
                Ok(())
            } else {
                let fuzzy_matches = FUZZY_MATCH.search(value, 0.25);
                let maybe_match = fuzzy_matches.first();

                maybe_match.map_or_else(
                    || Err(ErrorKind::Colorscheme(None).into()),
                    |top_match| Err(ErrorKind::Colorscheme(Some(top_match.text.clone())).into()),
                )
            }
        })
    }

    fn hash(&self) -> Result<()> {
        if let Some(hash) = &self.hash {
            validate_hash(hash)?;
        }
        Ok(())
    }

    fn url(&self) -> Result<()> {
        if let Some(url) = self.url {
            let url = Url::parse(url)?;
            let (_, hash) = url.path().split_at(1);
            validate_hash(hash)?;
            Ok(())
        } else {
            Ok(())
        }
    }
}

impl Check for HostParams {
    fn listen_addr(&self) -> Result<()> {
        let ipv4_err = || ErrorKind::InvalidParam("invalid ipv4 format.", "--listen");

        let (addr, port) = self.listen_addr.split_once(':').ok_or(ipv4_err())?;
        addr.parse::<Ipv4Addr>().map_err(|_| ipv4_err())?;

        port.parse::<u16>()
            .map_err(|_| ErrorKind::InvalidParam("invalid port.", "--port"))?;

        Ok(())
    }
}

pub fn validate_hash(hash: &str) -> Result<()> {
    let valid =
        (hash.starts_with('@') || hash.starts_with('#')) && hash.len() == GISTIT_HASH_CHAR_LENGTH;
    if !valid {
        return Err(ErrorKind::InvalidParam("invalid gistit hash format.", "--hash").into());
    }

    Ok(())
}
