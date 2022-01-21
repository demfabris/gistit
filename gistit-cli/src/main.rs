//
//   ________.__          __  .__  __
//  /  _____/|__| _______/  |_|__|/  |_
// /   \  ___|  |/  ___/\   __\  \   __\
// \    \_\  \  |\___ \  |  | |  ||  |
//  \______  /__/____  > |__| |__||__|
//         \/        \/
//
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![cfg_attr(
    test,
    allow(
        unused,
        clippy::all,
        clippy::pedantic,
        clippy::nursery,
        clippy::dbg_macro,
        clippy::unwrap_used,
        clippy::missing_docs_in_private_items,
    )
)]

mod args;
mod dispatch;
mod errors;
mod fetch;
mod host;
mod params;
mod send;
mod settings;

use std::io::{self, BufRead};
use std::process::exit;

use console::style;
use once_cell::sync::OnceCell;

pub use crate::errors::{Error, ErrorKind};
pub type Result<T> = std::result::Result<T, Error>;

use crate::args::app;
use crate::settings::Settings;

/// Stores the current command executed
pub static CURRENT_ACTION: OnceCell<String> = OnceCell::new();
/// Local config file
pub static LOCALFS_SETTINGS: OnceCell<Settings> = OnceCell::new();

async fn run() -> Result<()> {
    let matches = Box::leak(Box::new(app().get_matches()));
    LOCALFS_SETTINGS.set(Settings::default().merge_local()?)?;

    let cmd_args = if let Some((cmd, args)) = matches.subcommand() {
        CURRENT_ACTION.set(cmd.to_owned())?;
        (cmd, Some(args))
    } else {
        ("", None)
    };

    if matches.is_present("colorschemes") {
        list_bat_colorschemes();
        exit(0);
    }

    if matches.is_present("init-config") {
        Settings::save_new()?;
        prettyln!("Settings.yaml created!");
        exit(0);
    }

    match cmd_args {
        ("fetch", Some(args)) => {
            let action = Box::leak(Box::new(fetch::Action::from_args(args)?));
            let payload = action.prepare().await?;
            action.dispatch(payload).await?;
        }
        ("host", Some(args)) => {
            let action = Box::leak(Box::new(host::Action::from_args(args)?));
            let payload = action.prepare().await?;
            action.dispatch(payload).await?;
        }
        _ => {
            let default_action = Box::leak(Box::new(if matches.is_present("FILE") {
                send::Action::from_args(matches, None)?
            } else {
                let stdin = read_stdin();
                send::Action::from_args(matches, Some(stdin))?
            }));

            let payload = default_action.prepare().await?;
            default_action.dispatch(payload).await?;
        }
    };

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(err) = run().await {
        eprintln!(
            "{}: Something went wrong during {}{}: \n    {:?}",
            style("error").red().bold(),
            style("gistit-").green().bold(),
            style(CURRENT_ACTION.get().unwrap_or(&"action".to_string()))
                .green()
                .bold(),
            err
        )
    };

    Ok(())
}

#[doc(hidden)]
fn list_bat_colorschemes() {
    println!("{}", style("Supported colorschemes: \n").green().bold());
    crate::params::SUPPORTED_COLORSCHEMES.iter().for_each(|&c| {
        println!("    {}", style(c).yellow());
    });
    println!(
        r#"
This application uses '{}' to view gistits inside your terminal.
For more information please visit:
{}
        "#,
        style("bat").bold().blue(),
        style("https://github.com/sharkdp/bat").blue()
    );
}

const MAX_BYTES_FROM_STDIN: usize = 50_000;

fn read_stdin() -> String {
    let mut buf = String::new();
    let mut threshold = MAX_BYTES_FROM_STDIN;
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    while let Ok(read) = handle.read_line(&mut buf) {
        if threshold <= 0 || read == 0 {
            break;
        }
        threshold -= read;
    }

    buf
}

#[macro_export]
macro_rules! warnln {
    ($warn:expr) => {{
        use crate::CURRENT_ACTION;
        use console::style;

        eprintln!(
            "{}: in {}{}: \n    {}",
            style("warning").yellow().bold(),
            style("gistit-").green().bold(),
            style(CURRENT_ACTION.get().unwrap_or(&"any".to_owned()))
                .green()
                .bold(),
            $warn
        )
    }};
    ($msg:literal, $($rest:expr)*) => {{
        use crate::CURRENT_ACTION;
        use console::style;

        let msg = format!($msg, $($rest,)*);
        println!("{}: in {}{}: \n    {}",
            style("warning").yellow().bold(),
            style("gistit-").green().bold(),
            style(CURRENT_ACTION.get().unwrap_or("any"))
                .green()
                .bold(),
            msg
        );
    }};
}

#[macro_export]
macro_rules! prettyln {
    ($msg:expr) => {{
        println!(
            "{}{}",
            console::Emoji("\u{2734}  ", "> "),
            console::style($msg).bold(),
        );
    }};
    ($msg:literal, $($rest:expr)*) => {{
        let msg = format!($msg, $($rest,)*);
        println!("{}{}", console::Emoji("\u{2734}  ", "> "), console::style(msg).bold());
    }};
}
