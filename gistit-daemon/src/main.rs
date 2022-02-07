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

mod behaviour;
mod config;
mod error;
mod event;
mod network;

pub type Error = crate::error::Error;
pub type Result<T> = std::result::Result<T, Error>;

use std::path::PathBuf;

use argh::FromArgs;

use gistit_reference::dir;

#[derive(FromArgs, PartialEq, Debug)]
/// Gistit p2p node
struct Args {
    #[argh(option, long = "runtime-path")]
    /// override runtime directory
    runtime_path: Option<PathBuf>,

    #[argh(option, long = "config-path")]
    /// override config directory
    config_path: Option<PathBuf>,
}

async fn run() -> Result<()> {
    let args: Args = argh::from_env();
    let default_runtime = dir::runtime()?;
    let default_config = dir::config()?;

    let runtime_path = args.runtime_path.unwrap_or(default_runtime);
    let config_path = args.config_path.unwrap_or(default_config);

    let config = config::Config::new(runtime_path, config_path);
    let node = network::Node::new(config).await?;

    node.run().await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .write_style(env_logger::WriteStyle::Always)
        .init();

    while let Err(err) = run().await {
        log::error!("{:?}", err);
    }
}
