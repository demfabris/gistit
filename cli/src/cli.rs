//! Gistit command line interface
//!
/// Share or get a gistit.
#[derive(argh::FromArgs, PartialEq, Debug)]
pub struct MainArgs {
    /// dry run mode. check for platform requirements
    #[argh(switch, short = 'r')]
    dry_run: bool,
    /// list avaiable colorschemes
    #[argh(switch, short = 't')]
    colorschemes: bool,
    /// action
    #[argh(subcommand)]
    action: SubCommands,
}
/// Subcommands variations
#[derive(argh::FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum SubCommands {
    Send(SendArgs),
    Fetch(FetchArgs),
}
/// Upload a gistit to the cloud
#[derive(argh::FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "send")]
struct SendArgs {
    /// with a description
    #[argh(option, short = 'd')]
    description: Option<String>,
    /// with author details
    #[argh(option, short = 'a')]
    author: Option<String>,
    /// with password encryption
    #[argh(option, short = 's')]
    secret: Option<String>,
    /// blank
    #[argh(
        option,
        short = 't',
        description = "choose a colorscheme. 'gistit -t' for available colorschemes"
    )]
    theme: Option<String>,
    /// store output hash in clipboard. (on successful upload)
    #[argh(switch, short = 'c')]
    clipboard: bool,
    /// custom lifetime in seconds. DEFAULT = 3600, MAX = 3600
    #[argh(option, short = 'l')]
    lifetime: Option<i16>,
}
/// Fetch a gistit
#[derive(argh::FromArgs, PartialEq, Debug)]
#[argh(
    subcommand,
    name = "fetch",
    note = "The default filesystem save location is based on you platform..."
)]
struct FetchArgs {
    /// provide the secret to decrypt (if any)
    #[argh(option, short = 's')]
    secret: Option<String>,
    /// no syntax highlighting
    #[argh(switch, short = 'o')]
    no_syntax_highlighting: bool,
    /// save a copy on local filesystem
    #[argh(switch, short = 'v')]
    save: bool,
}
