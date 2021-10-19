//! Addons module
use async_trait::async_trait;
use lazy_static::lazy_static;
use ngrammatic::{Corpus, CorpusBuilder, Pad};
use phf::{phf_set, Set};

use std::ops::RangeInclusive;

use crate::{Error, Result};

/// Allowed description length
const ALLOWED_DESCRIPTION_CHAR_LENGHT_RANGE: RangeInclusive<usize> = 0..=100;
/// Allowed author info length
const ALLOWED_AUTHOR_CHAR_LENGTH_RANGE: RangeInclusive<usize> = 0..=50;
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
    /// Create a new [`Addons`] instance with colorscheme and lifespan specified.
    /// These fields are expected since they derive default values in the arguments parsing stage.
    #[must_use]
    pub fn new(colorscheme: &str, lifespan: u16) -> Self {
        Self {
            colorscheme: colorscheme.to_owned(),
            lifespan,
            ..Self::default()
        }
    }

    /// Append optional description and author information.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // False positive
    pub fn with_optional(self, description: Option<String>, author: Option<String>) -> Self {
        Self {
            author,
            description,
            ..self
        }
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
                Err(Error::InvalidAddons(format!(
                    "invalid colorscheme... did you mean {}?",
                    top_match.text
                )))
            } else {
                Err(Error::InvalidAddons(INVALID_COLORSCHEME_NAME.to_owned()))
            }
        }
    }
    async fn lifespan(&self) -> Result<()> {
        if ALLOWED_LIFESPAN_VALUE_RANGE.contains(&self.lifespan) {
            Ok(())
        } else {
            Err(Error::InvalidAddons(INVALID_LIFESPAN_RANGE.to_owned()))
        }
    }
    async fn description(&self) -> Result<()> {
        self.description.as_ref().map_or_else(
            || Ok(()),
            |value| {
                if ALLOWED_DESCRIPTION_CHAR_LENGHT_RANGE.contains(&value.len()) {
                    Ok(())
                } else {
                    Err(Error::InvalidAddons(
                        INVALID_DESCRIPTION_CHAR_LENGTH.to_owned(),
                    ))
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
                    Err(Error::InvalidAddons(INVALID_AUTHOR_CHAR_LENGTH.to_owned()))
                }
            },
        )
    }
}

#[doc(hidden)]
const INVALID_DESCRIPTION_CHAR_LENGTH: &str =
    "invalid description character length. MAX = 100 chars";
#[doc(hidden)]
const INVALID_AUTHOR_CHAR_LENGTH: &str = "invalid author character length. MAX = 50 chars";
#[doc(hidden)]
const INVALID_COLORSCHEME_NAME: &str =
    "invalid colorscheme. 'gistit --colors' to see avaiable ones.";
#[doc(hidden)]
const INVALID_LIFESPAN_RANGE: &str = "invalid lifespan parameter. MIN = 60s MAX = 3600s";
