//! Addons module
use ngrammatic::{Corpus, CorpusBuilder, Pad};
use phf::{phf_set, Set};

use crate::{Error, Result};

/// Allowed description character length
const ALLOWED_DESCRIPTION_LEN: usize = 100;
/// Allowed author info character length
const ALLOWED_AUTHOR_LEN: usize = 50;
/// Allowed lifetime range
const ALLOWED_LIFETIME_RANGE: std::ops::RangeInclusive<u16> = 300..=3600;

/// A [`phf::Set`] with all the supported colorschemes
static SUPPORTED_COLORSCHEMES: Set<&'static str> = phf_set![
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
lazy_static::lazy_static! {
    /// [`ngrammatic::Corpus`] constructed against [`SUPPORTED_COLORSCHEMES`] set
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
pub struct Addons<'a> {
    author: Option<&'a str>,
    description: Option<&'a str>,
    colorscheme: &'a str,
    lifetime: u16,
}
impl<'a> From<&'a super::Action> for Addons<'a> {
    fn from(action: &'a super::Action) -> Self {
        Self {
            author: action.author.as_deref(),
            description: action.description.as_deref(),
            colorscheme: action.theme.as_ref(),
            lifetime: action.lifetime,
        }
    }
}
#[async_trait::async_trait]
pub trait Check {
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
    async fn lifetime(&self) -> Result<()>;
}
#[async_trait::async_trait]
impl Check for Addons<'_> {
    async fn description(&self) -> Result<()> {
        let length = self.description.unwrap_or("").len();
        if ALLOWED_DESCRIPTION_LEN < length {
            Err(Error::InvalidAddons {
                message: INVALID_DESCRIPTION_CHAR_LENGTH.to_owned(),
            })
        } else {
            Ok(())
        }
    }
    async fn author(&self) -> Result<()> {
        let length = self.author.unwrap_or("").len();
        if ALLOWED_AUTHOR_LEN < length {
            Err(Error::InvalidAddons {
                message: INVALID_AUTHOR_CHAR_LENGTH.to_owned(),
            })
        } else {
            Ok(())
        }
    }
    async fn colorscheme(&self) -> Result<()> {
        let contains = SUPPORTED_COLORSCHEMES.contains(self.colorscheme);
        if contains {
            Ok(())
        } else {
            let fuzzy_matches = FUZZY_MATCH.search(self.colorscheme, 0.25);
            let maybe_match = fuzzy_matches.first();
            if let Some(top_match) = maybe_match {
                Err(Error::InvalidAddons {
                    message: format!("invalid colorscheme... did you mean {}?", top_match.text),
                })
            } else {
                Err(Error::InvalidAddons {
                    message: INVALID_COLORSCHEME_NAME.to_owned(),
                })
            }
        }
    }
    async fn lifetime(&self) -> Result<()> {
        if ALLOWED_LIFETIME_RANGE.contains(&self.lifetime) {
            Ok(())
        } else {
            Err(Error::InvalidAddons {
                message: INVALID_LIFETIME_RANGE.to_owned(),
            })
        }
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
const INVALID_LIFETIME_RANGE: &str = "invalid lifetime parameter. MIN = 60s MAX = 3600s";
