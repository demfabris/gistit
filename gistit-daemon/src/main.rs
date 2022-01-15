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
use unchecked_unwrap::UncheckedUnwrap;

use ::clap::ArgMatches;

use crate::args::app;
use crate::errors::ErrorKind;
use crate::network::{ipv4_to_multiaddr, NetworkConfig};

mod args;
mod errors;
mod network;

pub type Error = crate::errors::Error;
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(not(feature = "host"))]
fn main() {}

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
                seed: args.value_of("seed").unchecked_unwrap(),
                runtime_dir: args.value_of_os("runtime-dir").unchecked_unwrap(),
                inbound_addr: args
                    .value_of("host")
                    .unchecked_unwrap()
                    .parse()
                    .map_err(|_| ErrorKind::InvalidArgs)?,
                inbound_port: args
                    .value_of("port")
                    .unchecked_unwrap()
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

    let node = NetworkConfig::new(seed, multiaddr, runtime_dir)?
        .apply()
        .await?;

    println!("{:?}", node.peer_id());

    node.run().await?;

    Ok(())
}

#[cfg(feature = "host")]
#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        println!("{:?}", err);
        std::process::exit(1);
    }
}