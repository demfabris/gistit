//   ________.__          __  .__  __
//  /  _____/|__| _______/  |_|__|/  |_
// /   \  ___|  |/  ___/\   __\  \   __\
// \    \_\  \  |\___ \  |  | |  ||  |
//  \______  /__/____  > |__| |__||__|
//         \/        \/
//
// ------------------ Style police begin ------------------
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
// This would decrease readability
#![allow(clippy::module_name_repetitions)]
// Not my fault
#![allow(clippy::multiple_crate_versions)]
// Boring
#![allow(clippy::missing_panics_doc)]
// Test env should be chill
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
// ------------------ Style police end ------------------

pub mod cli;
pub mod dispatch;
pub mod params;
pub mod send;

#[cfg(feature = "fetch")]
pub mod fetch;

use std::sync::atomic::{AtomicBool, Ordering};

use console::style;
use lib_gistit::Result;
use once_cell::sync::OnceCell;

use cli::app;

/// Stores the current command executed
pub static CURRENT_ACTION: OnceCell<String> = OnceCell::new();
/// Stores wether or not to omit stdout
pub static OMIT_STDOUT: AtomicBool = AtomicBool::new(false);

async fn run() -> Result<()> {
    let matches = Box::leak(Box::new(app().get_matches()));
    CURRENT_ACTION
        .set(matches.subcommand().0.to_string())
        .expect("Internal error");
    match matches.subcommand() {
        ("send", Some(args)) => dispatch_from_args!(send, args),
        ("fetch", Some(args)) => dispatch_from_args!(fetch, args),
        ("", None) => {
            // Global commands
            if matches.is_present("colorschemes") {
                list_bat_colorschemes();
                std::process::exit(0);
            }
            if matches.is_present("silent") {
                OMIT_STDOUT.store(true, Ordering::Relaxed);
            }
            app().print_help().expect("Couldn't write to stdout");
        }
        _ => (),
    };
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Top level error output
    if let Err(err) = run().await {
        gistit_error!(err);
    };
    Ok(())
}

#[doc(hidden)]
fn list_bat_colorschemes() {
    let omit_stdout = crate::OMIT_STDOUT.load(::std::sync::atomic::Ordering::Relaxed);
    if omit_stdout {
        return;
    }
    println!("{}", style("Supported colorschemes: \n").green().bold());
    crate::params::SUPPORTED_BAT_COLORSCHEMES
        .iter()
        .for_each(|&c| {
            println!("    {}", style(c).yellow());
        });
    println!(
        r#"
This application uses '{}' to view gistits inside your terminal.
For more information please visit:
{}
        "#,
        style("bat").bold().blue(),
        style("https://github.com/sharkdp/bat").cyan()
    );
}

#[macro_export]
macro_rules! dispatch_from_args {
    ($mod:path, $args:expr) => {{
        use $mod as module;
        let action = module::Action::from_args($args)?;
        let payload = dispatch::Dispatch::prepare(&*action).await?;
        dispatch::Dispatch::dispatch(&*action, payload).await?;
    }};
}

#[macro_export]
macro_rules! gistit_error {
    ($err:expr) => {{
        use crate::CURRENT_ACTION;
        use console::style;
        let omit_stdout = crate::OMIT_STDOUT.load(::std::sync::atomic::Ordering::Relaxed);

        if !omit_stdout {
            eprintln!(
                "{}: Something went wrong during {}{}: \n    {:?}",
                style("error").red().bold(),
                style("gistit-").green().bold(),
                style(CURRENT_ACTION.get().expect("Internal error"))
                    .green()
                    .bold(),
                $err
            )
        }
    }};
}

#[macro_export]
macro_rules! gistit_warn {
    ($warn:expr) => {{
        use crate::CURRENT_ACTION;
        use console::style;
        let omit_stdout = crate::OMIT_STDOUT.load(::std::sync::atomic::Ordering::Relaxed);

        if !omit_stdout {
            eprintln!(
                "{}: Something went wrong during {}{}: \n    {}",
                style("warn").yellow().bold(),
                style("gistit-").green().bold(),
                style(CURRENT_ACTION.get().expect("Internal error"))
                    .green()
                    .bold(),
                $warn
            )
        }
    }};
}

#[macro_export]
macro_rules! gistit_line_out {
    ($msg:expr) => {{
        let omit_stdout = crate::OMIT_STDOUT.load(::std::sync::atomic::Ordering::Relaxed);

        if !omit_stdout {
            println!(
                "{}{}",
                console::Emoji("\u{2734}  ", "> "),
                console::style($msg).bold()
            );
        }
    }};
}
