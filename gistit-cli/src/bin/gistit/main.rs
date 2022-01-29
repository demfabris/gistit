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

mod arg;
mod dispatch;
mod fetch;
mod fmt;
mod node;
mod param;
mod send;
mod stdin;

pub use libgistit::error::Error;
pub type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(err) = run().await {
        errorln!(err);
    };

    Ok(())
}

async fn run() -> Result<()> {
    let matches = Box::leak(Box::new(arg::app().get_matches()));

    let (cmd, args) = if let Some((cmd, args)) = matches.subcommand() {
        fmt::set_action(cmd)?;
        (cmd, Some(args))
    } else {
        ("", None)
    };

    if matches.is_present("list-colorschemes") {
        list_bat_colorschemes();
    }

    match (cmd, args) {
        ("fetch", Some(args)) => {
            let action = fetch::Action::from_args(args)?;
            let payload = action.prepare().await?;
            action.dispatch(payload).await?;
        }
        ("node", Some(args)) => {
            let action = node::Action::from_args(args)?;
            let payload = action.prepare().await?;
            action.dispatch(payload).await?;
        }
        _ => {
            let default_action = if matches.is_present("FILE") {
                send::Action::from_args(matches, None)?
            } else {
                let stdin = stdin::read_to_end();
                send::Action::from_args(matches, Some(stdin))?
            };

            let payload = default_action.prepare().await?;
            default_action.dispatch(payload).await?;
        }
    };

    Ok(())
}

fn list_bat_colorschemes() {
    println!(
        "{}",
        console::style("Supported colorschemes: \n").green().bold()
    );
    for c in param::SUPPORTED_COLORSCHEMES {
        println!("    {}", c);
    }
    println!(
        r#"
This application uses '{}' to view gistits inside your terminal.
For more information please visit:
{}
        "#,
        console::style("bat").bold().blue(),
        "https://github.com/sharkdp/bat"
    );
}
