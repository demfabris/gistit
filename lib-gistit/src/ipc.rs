//! Inter process comunication module
#![allow(clippy::missing_errors_doc)]

use std::fs::{metadata, remove_file};
use std::path::{Path, PathBuf};

use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use tokio::net::UnixDatagram;

use crate::file::EncodedFileData;
use crate::Result;

const NAMED_SOCKET: &str = "gistit-0";
const READBUF_SIZE: usize = 60_000;

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

    pub fn alive(base: &Path) -> bool {
        if Bridge::connect(base).is_err() {
            false
        } else {
            true
        }
    }

    pub async fn send(&self, instruction: Instruction) -> Result<()> {
        let encoded = serialize(&instruction).unwrap();
        self.sock.send(&encoded).await?;
        Ok(())
    }

    pub async fn recv(&self) -> Result<Instruction> {
        let mut buf = vec![0u8; READBUF_SIZE];
        self.sock.recv(&mut buf).await?;
        let target: Instruction = deserialize(&buf).unwrap();
        Ok(target)
    }
}

impl Drop for Bridge {
    fn drop(&mut self) {
        if !self.keep_alive {
            remove_file(self.__base.join(NAMED_SOCKET)).expect("to remove named socket");
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Instruction {
    Shutdown,
    File(EncodedFileData),
}
