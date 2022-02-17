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

use clap::Parser;

use config::Config;
use node::Node;

/// Gistit p2p node
#[derive(Parser, PartialEq, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    /// Override runtime directory
    runtime_path: Option<PathBuf>,

    #[clap(long)]
    /// Override config directory
    config_path: Option<PathBuf>,

    #[clap(long)]
    /// IPFS config file to extract key material
    config_file: Option<PathBuf>,

    #[clap(long)]
    /// Address to listen for connections
    host: Option<Ipv4Addr>,

    #[clap(long)]
    /// Port to listen for connections
    port: Option<u16>,

    #[clap(long)]
    /// Dial these addresses on start
    dial: Vec<String>,

    #[clap(long)]
    /// Listen to these addresses, useful for relays
    listen: Vec<String>,

    #[clap(long)]
    /// Bootstrap this node
    bootstrap: bool,
}

async fn run() -> Result<()> {
    let Args {
        runtime_path,
        config_path,
        config_file,
        host,
        port,
        bootstrap,
        dial,
        listen,
    } = Args::parse();

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

    for addr in listen {
        node.listen_on_init(&addr)?;
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
