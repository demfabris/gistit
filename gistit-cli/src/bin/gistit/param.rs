use lazy_static::lazy_static;
use ngrammatic::{Corpus, CorpusBuilder, Pad};

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

pub mod check {
    use super::{FUZZY_MATCH, SUPPORTED_COLORSCHEMES};

    use std::ffi::OsStr;
    use std::fs;
    use std::ops::RangeInclusive;

    use libgistit::file::EXTENSION_TO_LANG_MAPPING;
    use libgistit::{Error, Result};

    const ALLOWED_FILE_SIZE_RANGE: RangeInclusive<u64> = 20..=50_000;

    const ALLOWED_DESCRIPTION_CHAR_LENGHT_RANGE: RangeInclusive<usize> = 10..=100;

    const ALLOWED_AUTHOR_CHAR_LENGTH_RANGE: RangeInclusive<usize> = 3..=30;

    const GISTIT_HASH_CHAR_LENGTH: usize = 64;

    pub fn description(description: &str) -> Result<&str> {
        if ALLOWED_DESCRIPTION_CHAR_LENGHT_RANGE.contains(&description.len()) {
            Ok(description)
        } else {
            Err(Error::Argument(
                "invalid description character length.",
                "--description",
            ))
        }
    }

    pub fn author(author: &str) -> Result<&str> {
        if ALLOWED_AUTHOR_CHAR_LENGTH_RANGE.contains(&author.len()) {
            Ok(author)
        } else {
            Err(Error::Argument(
                "invalid author character length.",
                "--author",
            ))
        }
    }

    pub fn metadata(attr: &fs::Metadata) -> Result<()> {
        let size_allowed = ALLOWED_FILE_SIZE_RANGE.contains(&attr.len());

        if size_allowed {
            Ok(())
        } else {
            Err(Error::Argument("file size not allowed", "[FILE]"))
        }
    }

    pub fn extension(ext: Option<&OsStr>) -> Result<()> {
        let ext = ext
            .and_then(OsStr::to_str)
            .ok_or(Error::Argument("file doesn't have an extension", "[FILE]"))?;

        if EXTENSION_TO_LANG_MAPPING.contains_key(ext) {
            Ok(())
        } else {
            Err(Error::Argument("file extension not supported", "[FILE]"))
        }
    }

    pub fn colorscheme(colorscheme: &str) -> Result<&str> {
        if SUPPORTED_COLORSCHEMES.contains(&colorscheme) {
            Ok(colorscheme)
        } else {
            let fuzzy_matches = FUZZY_MATCH.search(colorscheme, 0.25);
            let maybe_match = fuzzy_matches.first();

            maybe_match.map_or_else(
                || Err(Error::Argument("invalid colorscheme", "--colorscheme")),
                |top_match| Err(Error::Colorscheme(top_match.text.clone())),
            )
        }
    }

    pub const fn hash(hash: &str) -> Result<&str> {
        if hash.len() == GISTIT_HASH_CHAR_LENGTH {
            Ok(hash)
        } else {
            Err(Error::Argument("invalid gistit hash format.", "--hash"))
        }
    }
}
