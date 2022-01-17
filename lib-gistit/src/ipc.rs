//! Inter process comunication module
#![allow(clippy::missing_errors_doc)]

use std::fs::{metadata, remove_file};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::net::UnixDatagram;

use crate::Result;

const NAMED_SOCKET: &str = "gistit-0";

#[derive(Debug)]
pub struct Bridge {
    pub sock: UnixDatagram,
    keep_alive: bool,
    __base: PathBuf,
}

impl Bridge {
    pub fn bounded(base: &Path) -> Result<Self> {
        println!("binding tx {:?}", &base.join(NAMED_SOCKET));
        let sock_path = &base.join(NAMED_SOCKET);
        if metadata(sock_path).is_ok() {
            remove_file(sock_path)?;
        }

        Ok(Self {
            sock: UnixDatagram::bind(&base.join(NAMED_SOCKET))?,
            keep_alive: false,
            __base: base.to_path_buf(),
        })
    }

    pub fn connect(base: &Path) -> Result<Self> {
        let sock = UnixDatagram::unbound()?;
        println!("connecting tx {:?}", &base.join(NAMED_SOCKET));
        sock.connect(&base.join(NAMED_SOCKET))?;

        Ok(Self {
            sock,
            keep_alive: true,
            __base: base.to_path_buf(),
        })
    }

    pub fn check_alive(base: &Path) -> bool {
        if Bridge::connect(base).is_err() {
            false
        } else {
            true
        }
    }
}

impl Drop for Bridge {
    fn drop(&mut self) {
        if !self.keep_alive {
            remove_file(self.__base.join(NAMED_SOCKET)).expect("to remove named socket");
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum Command {
    Init,
}
