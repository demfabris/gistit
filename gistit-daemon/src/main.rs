//
//   ________.__          __  .__  __
//  /  _____/|__| _______/  |_|__|/  |_
// /   \  ___|  |/  ___/\   __\  \   __\
// \    \_\  \  |\___ \  |  | |  ||  |
//  \______  /__/____  > |__| |__||__|
//         \/        \/
//
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

use std::convert::Infallible;
use std::ffi::OsStr;
use std::net::Ipv4Addr;
use std::path::Path;

use clap::ArgMatches;

use args::app;
use errors::ErrorKind;
use network::{ipv4_to_multiaddr, NetworkConfig};

use lib_gistit::ipc::Bridge;

mod args;
mod errors;
mod network;

pub type Error = crate::errors::Error;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
struct Config {
    seed: &'static str,
    runtime_dir: &'static OsStr,
    inbound_addr: Ipv4Addr,
    inbound_port: u16,
}

impl Config {
    fn from_args(args: &'static ArgMatches) -> Result<Self> {
        // SAFETY: They all have default values
        unsafe {
            Ok(Self {
                seed: args.value_of("seed").unwrap_unchecked(),
                runtime_dir: args.value_of_os("runtime-dir").unwrap_unchecked(),
                inbound_addr: args
                    .value_of("host")
                    .unwrap_unchecked()
                    .parse()
                    .map_err(|_| ErrorKind::InvalidArgs)?,
                inbound_port: args
                    .value_of("port")
                    .unwrap_unchecked()
                    .parse()
                    .map_err(|_| ErrorKind::InvalidArgs)?,
            })
        }
    }
}

async fn run() -> Result<()> {
    let args = Box::leak(Box::new(app().get_matches()));
    let Config {
        seed,
        inbound_addr,
        inbound_port,
        runtime_dir,
    } = Config::from_args(args)?;

    let multiaddr = ipv4_to_multiaddr(inbound_addr, inbound_port);
    let runtime_dir = Path::new(runtime_dir);
    println!(
        "{:?} {:?} {:?} {:?}",
        runtime_dir, seed, inbound_addr, inbound_port
    );

    let bridge = Bridge::bounded(runtime_dir)?;
    let node = NetworkConfig::new(seed, multiaddr, runtime_dir, bridge)?
        .apply()
        .await?;

    println!("{:?}", node.peer_id());

    node.run().await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("DAEMON ERROR: {:?}", err);
        std::process::exit(1);
    }
}
