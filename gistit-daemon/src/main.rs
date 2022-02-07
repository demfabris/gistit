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

use gistit_reference::dir::{config_dir, runtime_dir};

async fn run() -> Result<()> {
    let config = config::Config::new(runtime_dir()?, config_dir()?);
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
