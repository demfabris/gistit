use std::ffi::OsStr;
use std::fs;
use std::net::Ipv4Addr;
use std::ops::RangeInclusive;
use std::path::Path;

use lazy_static::lazy_static;
use ngrammatic::{Corpus, CorpusBuilder, Pad};

use lib_gistit::file::EXTENSION_TO_LANG_MAPPING;

use crate::fetch::Action as FetchAction;
use crate::node::Action as NodeAction;
use crate::send::Action as SendAction;
use crate::{ErrorKind, Result};

/// Allowed file size range in bytes
const ALLOWED_FILE_SIZE_RANGE: RangeInclusive<u64> = 20..=50_000;
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

fn description(description: &str) -> Result<()> {
    if ALLOWED_DESCRIPTION_CHAR_LENGHT_RANGE.contains(&description.len()) {
        Ok(())
    } else {
        Err(
            ErrorKind::InvalidParam("invalid description character length.", "--description")
                .into(),
        )
    }
}

fn author(author: &str) -> Result<()> {
    if ALLOWED_AUTHOR_CHAR_LENGTH_RANGE.contains(&author.len()) {
        Ok(())
    } else {
        Err(ErrorKind::InvalidParam("invalid author character length.", "--author").into())
    }
}

fn colorscheme(colorscheme: &str) -> Result<()> {
    if SUPPORTED_COLORSCHEMES.contains(&colorscheme) {
        Ok(())
    } else {
        let fuzzy_matches = FUZZY_MATCH.search(colorscheme, 0.25);
        let maybe_match = fuzzy_matches.first();

        maybe_match.map_or_else(
            || Err(ErrorKind::Colorscheme(None).into()),
            |top_match| Err(ErrorKind::Colorscheme(Some(top_match.text.clone())).into()),
        )
    }
}

fn hash(hash: &str) -> Result<()> {
    let valid =
        (hash.starts_with('@') || hash.starts_with('#')) && hash.len() == GISTIT_HASH_CHAR_LENGTH;
    if !valid {
        return Err(ErrorKind::InvalidParam("invalid gistit hash format.", "--hash").into());
    }

    Ok(())
}

fn host(host: &str) -> Result<()> {
    host.parse::<Ipv4Addr>()
        .map_err(|_| ErrorKind::InvalidParam("invalid ipv4 format.", "--host"))?;
    Ok(())
}

fn port(port: &str) -> Result<()> {
    port.parse::<u16>()
        .map_err(|_| ErrorKind::InvalidParam("invalid port.", "--port"))?;
    Ok(())
}

fn metadata(file_path: &Path) -> Result<()> {
    let handler = fs::File::open(file_path)?;
    let attr = handler.metadata()?;
    let size_allowed = ALLOWED_FILE_SIZE_RANGE.contains(&attr.len());

    if !size_allowed {
        return Err(ErrorKind::FileSize.into());
    }
    Ok(())
}

fn extension(file_path: &Path) -> Result<()> {
    let ext = file_path
        .extension()
        .and_then(OsStr::to_str)
        .ok_or(ErrorKind::FileExtension)?;

    if EXTENSION_TO_LANG_MAPPING.contains_key(ext) {
        Ok(())
    } else {
        Err(ErrorKind::FileExtension.into())
    }
}

pub trait Check {
    fn check(&self) -> Result<()>;
}

impl Check for SendAction {
    fn check(&self) -> Result<()> {
        if let Some(value) = self.description {
            description(value)?;
        }
        if let Some(file_path_osstr) = self.file_path {
            let file_path = Path::new(file_path_osstr);
            metadata(&file_path)?;
            extension(&file_path)?;
        }
        author(self.author)?;
        Ok(())
    }
}

impl Check for FetchAction {
    fn check(&self) -> Result<()> {
        hash(self.hash)?;
        if let Some(value) = self.colorscheme {
            colorscheme(&value)?;
        }
        Ok(())
    }
}

impl Check for NodeAction {
    fn check(&self) -> Result<()> {
        if let Some(value) = self.file {
            let file_path = Path::new(&value);
            metadata(&file_path)?;
            extension(&file_path)?;
        }
        host(&self.host)?;
        port(&self.port)?;
        Ok(())
    }
}
