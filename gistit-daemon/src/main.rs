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

use std::ffi::OsStr;
use std::path::PathBuf;

use clap::ArgMatches;

use args::app;
use network::NetworkConfig;

mod args;
mod errors;
mod network;

pub type Error = crate::errors::Error;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
struct Config {
    runtime_dir: &'static OsStr,
    config_dir: &'static OsStr,
}

impl Config {
    fn from_args(args: &'static ArgMatches) -> Result<Self> {
        // SAFETY: Args have default values
        unsafe {
            Ok(Self {
                runtime_dir: args.value_of_os("runtime-dir").unwrap_unchecked(),
                config_dir: args.value_of_os("config-dir").unwrap_unchecked(),
            })
        }
    }
}

async fn run() -> Result<()> {
    let args = Box::leak(Box::new(app().get_matches()));
    let Config {
        runtime_dir,
        config_dir,
    } = Config::from_args(args)?;

    let runtime_dir = PathBuf::new().join(runtime_dir);
    let _config_dir = PathBuf::new().join(config_dir);

    let node = NetworkConfig::new(runtime_dir)?.apply().await?;
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
