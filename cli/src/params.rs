//! Params module
use async_trait::async_trait;
use lazy_static::lazy_static;
use ngrammatic::{Corpus, CorpusBuilder, Pad};
use phf::{phf_set, Set};
use url::Url;

use std::borrow::ToOwned;
use std::ops::RangeInclusive;

use crate::errors::params::ParamsError;
use crate::fetch::Action as FetchAction;
use crate::send::Action as SendAction;
use crate::Result;

/// Allowed description length
const ALLOWED_DESCRIPTION_CHAR_LENGHT_RANGE: RangeInclusive<usize> = 10..=100;
/// Allowed author info length
const ALLOWED_AUTHOR_CHAR_LENGTH_RANGE: RangeInclusive<usize> = 3..=30;
/// Allowed lifespan range
const ALLOWED_LIFESPAN_VALUE_RANGE: RangeInclusive<u16> = 300..=3600;
/// Valid hash length
const GISTIT_HASH_CHAR_LENGTH: usize = 33;

/// A [`phf::Set`] with all the supported colorschemes
const SUPPORTED_COLORSCHEMES: Set<&'static str> = phf_set![
    "coy",
    "dark",
    "funky",
    "okaidia",
    "solarizedlight",
    "tomorrow",
    "twilight",
    "prism",
    "a11yDark",
    "atomDark",
    "base16AteliersulphurpoolLight",
    "cb",
    "coldarkCold",
    "coldarkDark",
    "coyWithoutShadows",
    "darcula",
    "dracula",
    "duotoneDark",
    "duotoneEarth",
    "duotoneForest",
    "duotoneLight",
    "duotoneSea",
    "duotoneSpace",
    "ghcolors",
    "hopscotch",
    "materialDark",
    "materialLight",
    "materialOceanic",
    "nord",
    "pojoaque",
    "shadesOfPurple",
    "synthwave84",
    "vs",
    "vscDarkPlus",
    "xonokai",
];

lazy_static! {
    /// A [`ngrammatic::Corpus`] constructed against [`SUPPORTED_COLORSCHEMES`] set
    /// to fuzzy match user colorscheme incorrect attempts.
    static ref FUZZY_MATCH: Corpus = SUPPORTED_COLORSCHEMES.iter().fold(
        CorpusBuilder::new().arity(2).pad_full(Pad::Auto).finish(),
        |mut corpus, &t| {
            corpus.add_text(t);
            corpus
        },
    );
}

/// Common function to match agains the avaiable colorschemes
///
/// # Errors
///
/// Fails with [`InvalidParams`] if colorscheme isn't named properly.
/// Prompts the user with a suggestion if it fuzzy matches agains't a probability.
pub fn try_match_colorscheme(value: &(impl AsRef<str> + Send)) -> Result<()> {
    if SUPPORTED_COLORSCHEMES.contains(value.as_ref()) {
        Ok(())
    } else {
        let fuzzy_matches = FUZZY_MATCH.search(value.as_ref(), 0.25);
        let maybe_match = fuzzy_matches.first();

        maybe_match.map_or_else(
            || Err(ParamsError::Colorscheme(None).into()),
            |top_match| Err(ParamsError::Colorscheme(Some(top_match.text.clone())).into()),
        )
    }
}

/// Main params struct, used to further check parameters based on the action
pub struct Params;

pub trait SendArgs {}
impl SendArgs for SendParams {}
/// The data structure that holds data to be checked/dispatched during a [`SendAction`]
#[derive(Clone, Default, Debug)]
pub struct SendParams {
    pub author: Option<String>,
    pub description: Option<String>,
    pub colorscheme: String,
    pub lifespan: u16,
}

/// Marker trait for fetch action
pub trait FetchArgs {}
impl FetchArgs for FetchParams {}
/// The data structure that holds data to be checked/dispatched during a [`FetchAction`]
#[derive(Clone, Default, Debug)]
pub struct FetchParams {
    pub hash: Option<String>,
    pub url: Option<String>,
    pub colorscheme: String,
}

impl SendParams {
    /// Perform all the needed checks to the params fields concurrently.
    ///
    /// # Errors
    ///
    /// Fails with [`InvalidParams`] error
    pub fn check_consume(self) -> Result<Self> {
        <Self as Check>::lifespan(&self)?;
        <Self as Check>::colorscheme(&self)?;
        <Self as Check>::description(&self)?;
        <Self as Check>::author(&self)?;
        Ok(self)
    }
}

impl FetchParams {
    /// Perform all the needed checks to the params fields concurrently.
    ///
    /// # Errors
    ///
    /// Fails with [`InvalidParams`] error
    pub fn check_consume(self) -> Result<Self> {
        <Self as Check>::colorscheme(&self)?;
        <Self as Check>::hash(&self)?;
        <Self as Check>::url(&self)?;
        Ok(self)
    }
}

impl Params {
    /// Create a new [`SendParams`] from [`SendAction`]
    ///
    /// # Errors
    ///
    /// Fails with [`InvalidParams`] error
    pub fn from_send(action: &SendAction) -> Result<SendParams> {
        Ok(SendParams {
            author: action.author.map(ToOwned::to_owned),
            description: action.description.map(ToOwned::to_owned),
            colorscheme: action.theme.to_owned(),
            lifespan: action
                .lifespan
                .parse::<u16>()
                .map_err(|_| ParamsError::InvalidLifespan)?,
        })
    }

    /// Create a new [`FetchParams`] from [`FetchAction`]
    ///
    /// # Errors
    ///
    /// Fails with [`InvalidParams`] error
    pub fn from_fetch(action: &FetchAction) -> Result<FetchParams> {
        Ok(FetchParams {
            hash: action.hash.map(ToOwned::to_owned),
            url: action.url.map(ToOwned::to_owned),
            colorscheme: action.colorscheme.to_owned(),
        })
    }
}

#[async_trait]
trait Check {
    /// Check provided colorscheme name against supported ones
    ///
    /// # Errors
    ///
    /// Fails with [`InvalidParams`] if colorscheme isn't named properly.
    /// Prompts the user with a suggestion if it fuzzy matches agains't a probability.
    fn colorscheme(&self) -> Result<()>;

    /// Check provided lifetime limit range
    ///
    /// # Errors
    ///
    /// Fails with [`InvalidParams`] if the provided number is outside allowed range.
    fn lifespan(&self) -> Result<()>
    where
        Self: SendArgs,
    {
        Ok(())
    }

    /// Check description character length (if any)
    ///
    /// # Errors
    ///
    /// Fails with [`InvalidParams`] if description is over allowed length
    fn description(&self) -> Result<()>
    where
        Self: SendArgs,
    {
        Ok(())
    }

    /// Check author character length (if any)
    ///
    /// # Errors
    ///
    /// Fails with [`InvalidParams`] if author name is over allowed length
    fn author(&self) -> Result<()>
    where
        Self: SendArgs,
    {
        Ok(())
    }

    /// Check wthe gistit hash (if any)
    ///
    /// # Errors
    ///
    /// Fails with [`InvalidParams`] if hash is an invalid format
    fn hash(&self) -> Result<()>
    where
        Self: FetchArgs,
    {
        Ok(())
    }

    /// Check the gistit url (if any)
    ///
    /// # Errors
    ///
    /// Fails with [`InvalidParams`] if url is invalid
    fn url(&self) -> Result<()>
    where
        Self: FetchArgs,
    {
        Ok(())
    }
}

#[async_trait]
impl Check for SendParams {
    fn colorscheme(&self) -> Result<()> {
        try_match_colorscheme(&self.colorscheme)
    }
    fn lifespan(&self) -> Result<()>
    where
        Self: SendArgs,
    {
        if ALLOWED_LIFESPAN_VALUE_RANGE.contains(&self.lifespan) {
            Ok(())
        } else {
            Err(ParamsError::LifespanRange.into())
        }
    }
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
                    Err(ParamsError::DescriptionCharRange.into())
                }
            },
        )
    }
    fn author(&self) -> Result<()>
    where
        Self: SendArgs,
    {
        self.author.as_ref().map_or_else(
            || Ok(()),
            |value| {
                if ALLOWED_AUTHOR_CHAR_LENGTH_RANGE.contains(&value.len()) {
                    Ok(())
                } else {
                    Err(ParamsError::AuthorCharRange.into())
                }
            },
        )
    }
}

#[async_trait]
impl Check for FetchParams {
    fn colorscheme(&self) -> Result<()> {
        try_match_colorscheme(&self.colorscheme)
    }
    fn hash(&self) -> Result<()> {
        if let Some(hash) = &self.hash {
            let valid = (hash.starts_with('@') || hash.starts_with('$'))
                && hash.len() == GISTIT_HASH_CHAR_LENGTH;
            if !valid {
                return Err(ParamsError::InvalidHash(hash.clone()).into());
            }
        }
        Ok(())
    }
    fn url(&self) -> Result<()> {
        if let Some(ref url) = self.url {
            let _url = Url::parse(url).map_err(|err| ParamsError::InvalidUrl(err.to_string()))?;
            Ok(())
        } else {
            Ok(())
        }
    }
}
