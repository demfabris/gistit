//! Inter process comunication module
#![allow(clippy::missing_errors_doc)]

use std::env::temp_dir;
use std::path::PathBuf;

use tokio::net::UnixDatagram;

use crate::Result;

#[derive(Debug, Default)]
pub struct BridgeUnbinded {
    tx_path: PathBuf,
    rx_path: PathBuf,
}

impl BridgeUnbinded {
    #[must_use]
    pub const fn new(tx_path: PathBuf, rx_path: PathBuf) -> Self {
        Self { tx_path, rx_path }
    }

    #[must_use]
    pub fn new_rng() -> Self {
        let tmp = temp_dir();

        let tx_path = tmp.join("__gistit_tx");
        let rx_path = tmp.join("__gistit_rx");

        Self::new(tx_path, rx_path)
    }

    pub fn into_binded(self) -> Result<BridgeBinded> {
        let tx = UnixDatagram::bind(&self.tx_path)?;
        let rx = UnixDatagram::bind(&self.rx_path)?;

        Ok(BridgeBinded { tx, rx })
    }
}

#[derive(Debug)]
pub struct BridgeBinded {
    tx: UnixDatagram,
    rx: UnixDatagram,
}

impl BridgeBinded {
    pub fn foo() {
        todo!()
    }
}

pub enum Command {
    Init,
}
