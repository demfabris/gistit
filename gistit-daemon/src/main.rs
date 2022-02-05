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

use clap::Parser;
use std::path::PathBuf;

mod behaviour;
mod config;
mod error;
mod network;

pub type Error = crate::error::Error;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Parser, Clone, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Directory where sockets and runtime artifacts will be placed
    #[clap(long)]
    runtime_dir: PathBuf,

    /// Directory to store configuration
    #[clap(long)]
    config_dir: PathBuf,
}

async fn run() -> Result<()> {
    let Args {
        runtime_dir,
        config_dir,
    } = Args::parse();

    let config = config::Config::new(runtime_dir, config_dir);
    let node = network::Node::new(config).await?;
    node.run().await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .write_style(env_logger::WriteStyle::Always)
        .init();

    while let Err(err) = run().await {
        log::error!("{:?}", err);
    }
}
