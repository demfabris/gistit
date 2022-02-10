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
mod node;

pub type Error = crate::error::Error;
pub type Result<T> = std::result::Result<T, Error>;

use std::net::Ipv4Addr;
use std::path::PathBuf;

use argh::FromArgs;

use config::Config;
use node::Node;

#[derive(FromArgs, PartialEq, Debug)]
/// Gistit p2p node
struct Args {
    #[argh(option, long = "runtime-path")]
    /// override runtime directory
    runtime_path: Option<PathBuf>,

    #[argh(option, long = "config-path")]
    /// override config directory
    config_path: Option<PathBuf>,

    #[argh(option, long = "config-file")]
    /// IPFS config file to extract key material
    config_file: Option<PathBuf>,

    #[argh(option)]
    /// address to listen for connections
    host: Option<Ipv4Addr>,

    #[argh(option)]
    /// port to listen for connections
    port: Option<u16>,

    #[argh(option)]
    /// dial these addresses on start
    dial: Vec<String>,

    #[argh(switch)]
    /// bootstrap this node
    bootstrap: bool,
}

async fn run() -> Result<()> {
    let Args {
        runtime_path,
        config_path,
        config_file,
        host,
        port,
        dial,
        bootstrap,
    } = argh::from_env();

    let config = Config::from_args(
        runtime_path,
        config_path,
        config_file,
        host,
        port,
        bootstrap,
    )?;
    log::debug!("Running config: {:?}", config);

    let mut node = Node::new(config).await?;
    for addr in dial {
        node.dial_on_init(&addr)?;
    }

    node.run().await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        // .filter_level(log::LevelFilter::Info)
        .write_style(env_logger::WriteStyle::Always)
        .init();

    while let Err(err) = run().await {
        log::error!("{:?}", err);
    }
}
