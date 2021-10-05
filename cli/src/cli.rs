//! Gistit command line interface
//!
use std::ffi::OsString;

/// Share or get a gistit.
#[derive(argh::FromArgs, PartialEq, Debug)]
pub struct MainArgs {
    /// dry run mode. check for platform requirements
    #[argh(switch, short = 'r')]
    pub dry_run: bool,
    /// list avaiable colorschemes
    #[argh(switch, short = 't')]
    pub colorschemes: bool,
    /// action
    #[argh(subcommand)]
    pub action: Command,
}
/// Subcommands variations
#[non_exhaustive]
#[derive(argh::FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum Command {
    Send(SendArgs),
    Fetch(FetchArgs),
}
/// Upload a gistit to the cloud
#[derive(argh::FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "send")]
pub struct SendArgs {
    /// the file. currently one supported
    #[argh(option, short = 'f')]
    pub file: OsString,
    /// with a description
    #[argh(option, short = 'd')]
    pub description: Option<String>,
    /// with author details
    #[argh(option, short = 'a')]
    pub author: Option<String>,
    /// with password encryption
    #[argh(option, short = 's')]
    pub secret: Option<String>,
    /// blank
    #[argh(
        option,
        short = 't',
        description = "choose a colorscheme. 'gistit -t' for available colorschemes",
        default = "String::from(\"dracula\")"
    )]
    pub theme: String,
    /// store output hash in clipboard. (on successful upload)
    #[argh(switch, short = 'c')]
    pub clipboard: bool,
    /// custom lifetime in seconds. DEFAULT = 3600, MAX = 3600
    #[argh(option, short = 'l')]
    pub lifetime: Option<u16>,
}
/// Fetch a gistit
#[derive(argh::FromArgs, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "fetch",
    note = "The default filesystem save location is based on you platform..."
)]
pub struct FetchArgs {
    /// provide the secret to decrypt (if any)
    #[argh(option, short = 's')]
    pub secret: Option<String>,
    /// no syntax highlighting
    #[argh(switch, short = 'o')]
    pub no_syntax_highlighting: bool,
    /// save a copy on local filesystem
    #[argh(switch, short = 'v')]
    pub save: bool,
}
