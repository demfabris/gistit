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
    use std::net::Ipv4Addr;
    use std::ops::RangeInclusive;

    use crate::{ErrorKind, Result};

    use lib_gistit::file::EXTENSION_TO_LANG_MAPPING;

    const ALLOWED_FILE_SIZE_RANGE: RangeInclusive<u64> = 20..=50_000;

    const ALLOWED_DESCRIPTION_CHAR_LENGHT_RANGE: RangeInclusive<usize> = 10..=100;

    const ALLOWED_AUTHOR_CHAR_LENGTH_RANGE: RangeInclusive<usize> = 3..=30;

    const GISTIT_HASH_CHAR_LENGTH: usize = 32;

    pub fn description(description: &str) -> Result<&str> {
        if ALLOWED_DESCRIPTION_CHAR_LENGHT_RANGE.contains(&description.len()) {
            Ok(description)
        } else {
            Err(
                ErrorKind::InvalidParam("invalid description character length.", "--description")
                    .into(),
            )
        }
    }

    pub fn author(author: &str) -> Result<&str> {
        if ALLOWED_AUTHOR_CHAR_LENGTH_RANGE.contains(&author.len()) {
            Ok(author)
        } else {
            Err(ErrorKind::InvalidParam("invalid author character length.", "--author").into())
        }
    }

    pub fn metadata(attr: fs::Metadata) -> Result<()> {
        let size_allowed = ALLOWED_FILE_SIZE_RANGE.contains(&attr.len());

        if size_allowed {
            Ok(())
        } else {
            Err(ErrorKind::FileSize.into())
        }
    }

    pub fn extension(ext: Option<&OsStr>) -> Result<()> {
        let ext = ext.and_then(OsStr::to_str).ok_or(ErrorKind::FileExtension)?;

        if EXTENSION_TO_LANG_MAPPING.contains_key(ext) {
            Ok(())
        } else {
            Err(ErrorKind::FileExtension.into())
        }
    }

    pub fn colorscheme(colorscheme: &str) -> Result<&str> {
        if SUPPORTED_COLORSCHEMES.contains(&colorscheme) {
            Ok(colorscheme)
        } else {
            let fuzzy_matches = FUZZY_MATCH.search(colorscheme, 0.25);
            let maybe_match = fuzzy_matches.first();

            maybe_match.map_or_else(
                || Err(ErrorKind::Colorscheme(None).into()),
                |top_match| Err(ErrorKind::Colorscheme(Some(top_match.text.clone())).into()),
            )
        }
    }

    pub fn hash(hash: &str) -> Result<&str> {
        if hash.len() == GISTIT_HASH_CHAR_LENGTH {
            Ok(hash)
        } else {
            Err(ErrorKind::InvalidParam("invalid gistit hash format.", "--hash").into())
        }
    }

    pub fn host(host: &str) -> Result<Ipv4Addr> {
        Ok(host.parse::<Ipv4Addr>()
            .map_err(|_| ErrorKind::InvalidParam("invalid ipv4 format.", "--host"))?)
    }

    pub fn port(port: &str) -> Result<u16> {
        Ok(port.parse::<u16>()
            .map_err(|_| ErrorKind::InvalidParam("invalid port.", "--port"))?)
    }
}
