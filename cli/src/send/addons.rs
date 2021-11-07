//! Addons module
use async_trait::async_trait;
use lazy_static::lazy_static;
use ngrammatic::{Corpus, CorpusBuilder, Pad};
use phf::{phf_set, Set};

use std::borrow::ToOwned;
use std::ops::RangeInclusive;

use super::Action;
use crate::errors::addons::AddonsError;
use crate::Result;

/// Allowed description length
const ALLOWED_DESCRIPTION_CHAR_LENGHT_RANGE: RangeInclusive<usize> = 10..=100;
/// Allowed author info length
const ALLOWED_AUTHOR_CHAR_LENGTH_RANGE: RangeInclusive<usize> = 3..=30;
/// Allowed lifespan range
const ALLOWED_LIFESPAN_VALUE_RANGE: RangeInclusive<u16> = 300..=3600;

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

/// The addons struct that holds data to be checked/dispatched
#[derive(Clone, Default, Debug)]
pub struct Addons {
    author: Option<String>,
    description: Option<String>,
    colorscheme: String,
    lifespan: u16,
}

impl Addons {
    /// Create a new [`Addons`] from the send [`Action`]
    ///
    /// # Errors
    ///
    /// Fails with [`InvalidAddons`] error
    pub fn from_action(action: &Action) -> Result<Self> {
        Ok(Self {
            author: action.author.map(ToOwned::to_owned),
            description: action.description.map(ToOwned::to_owned),
            colorscheme: action.theme.to_owned(),
            lifespan: action
                .lifespan
                .parse::<u16>()
                .map_err(|_| AddonsError::InvalidLifespan)?,
        })
    }

    /// Perform all the needed checks to the addons fields concurrently.
    ///
    /// # Errors
    ///
    /// Fails with [`InvalidAddons`] error
    pub async fn check_consume(self) -> Result<Self> {
        let _ = tokio::try_join! {
            <Self as Check>::lifespan(&self),
            <Self as Check>::colorscheme(&self),
            <Self as Check>::description(&self),
            <Self as Check>::author(&self),
        }?;
        Ok(self)
    }
}

#[async_trait]
trait Check {
    /// Check provided colorscheme name against supported ones
    ///
    /// # Errors
    ///
    /// Fails with [`InvalidAddons`] if colorscheme isn't named properly.
    /// Prompts the user with a suggestion if it fuzzy matches agains't a probability.
    async fn colorscheme(&self) -> Result<()>;

    /// Check provided lifetime limit range
    ///
    /// # Errors
    ///
    /// Fails with [`InvalidAddons`] if the provided number is outside allowed range.
    async fn lifespan(&self) -> Result<()>;

    /// Check description character length (if any)
    ///
    /// # Errors
    ///
    /// Fails with [`InvalidAddons`] if description is over allowed length
    async fn description(&self) -> Result<()>;

    /// Check author character length (if any)
    ///
    /// # Errors
    ///
    /// Fails with [`InvalidAddons`] if author name is over allowed length
    async fn author(&self) -> Result<()>;
}

#[async_trait]
impl Check for Addons {
    async fn colorscheme(&self) -> Result<()> {
        if SUPPORTED_COLORSCHEMES.contains(&self.colorscheme) {
            Ok(())
        } else {
            let fuzzy_matches = FUZZY_MATCH.search(&self.colorscheme, 0.25);
            let maybe_match = fuzzy_matches.first();

            if let Some(top_match) = maybe_match {
                Err(AddonsError::Colorscheme(Some(top_match.text.clone())).into())
            } else {
                Err(AddonsError::Colorscheme(None).into())
            }
        }
    }
    async fn lifespan(&self) -> Result<()> {
        if ALLOWED_LIFESPAN_VALUE_RANGE.contains(&self.lifespan) {
            Ok(())
        } else {
            Err(AddonsError::LifespanRange.into())
        }
    }
    async fn description(&self) -> Result<()> {
        self.description.as_ref().map_or_else(
            || Ok(()),
            |value| {
                if ALLOWED_DESCRIPTION_CHAR_LENGHT_RANGE.contains(&value.len()) {
                    Ok(())
                } else {
                    Err(AddonsError::DescriptionCharRange.into())
                }
            },
        )
    }
    async fn author(&self) -> Result<()> {
        self.author.as_ref().map_or_else(
            || Ok(()),
            |value| {
                if ALLOWED_AUTHOR_CHAR_LENGTH_RANGE.contains(&value.len()) {
                    Ok(())
                } else {
                    Err(AddonsError::AuthorCharRange.into())
                }
            },
        )
    }
}
