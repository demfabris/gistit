//! Inter process comunication module
#![allow(clippy::missing_errors_doc)]

// use std::fs::remove_file;
// use std::ops::Drop;
use std::path::{Path, PathBuf};

use tokio::net::UnixDatagram;

use crate::Result;

const SOCKET_TX: &str = "__gistit_tx";
const SOCKET_RX: &str = "__gistit_rx";

#[derive(Debug)]
pub struct Bridge {
    pub tx: UnixDatagram,
    pub rx: UnixDatagram,
    __base: PathBuf,
}

impl Bridge {
    pub fn bounded(base: &Path) -> Result<Self> {
        println!("binding tx {:?}", &base.join(SOCKET_TX));
        Ok(Self {
            tx: UnixDatagram::bind(&base.join(SOCKET_TX))?,
            rx: UnixDatagram::bind(&base.join(SOCKET_RX))?,
            __base: base.to_path_buf(),
        })
    }

    pub fn connect(base: &Path) -> Result<Self> {
        let tx = UnixDatagram::unbound()?;
        let rx = UnixDatagram::unbound()?;
        println!("connecting tx {:?}", &base.join(SOCKET_TX));
        tx.connect(&base.join(SOCKET_TX))?;
        rx.connect(&base.join(SOCKET_RX))?;

        Ok(Self {
            tx,
            rx,
            __base: base.to_path_buf(),
        })
    }
}

// impl Drop for Bridge {
//     fn drop(&mut self) {
//         remove_file(self.__base.join(SOCKET_TX)).expect("To remove tx socket");
//         remove_file(self.__base.join(SOCKET_RX)).expect("To remove rx socket");
//     }
// }

pub enum Command {
    Init,
}
