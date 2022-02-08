use std::fmt::Debug;
use std::net::Ipv4Addr;
use std::path::PathBuf;

use libp2p::core::{Multiaddr, PeerId};
use libp2p::identity::{self, ed25519, Keypair};
use libp2p::multiaddr::multiaddr;

use gistit_reference::dir;

use crate::Result;

pub struct Config {
    pub peer_id: PeerId,
    pub keypair: Keypair,
    pub runtime_path: PathBuf,
    pub config_path: PathBuf,
    pub multiaddr: Multiaddr,
}

impl Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {:?} {:?} {:?}",
            self.peer_id, self.runtime_path, self.config_path, self.multiaddr,
        )
    }
}

impl Config {
    pub fn from_args(
        runtime_path: Option<PathBuf>,
        config_path: Option<PathBuf>,
        host: Option<Ipv4Addr>,
        port: Option<u16>,
    ) -> Result<Self> {
        dir::init()?;

        let host = host.unwrap_or(Ipv4Addr::new(0, 0, 0, 0));
        let port = port.unwrap_or(0_u16);

        let runtime_path = runtime_path.unwrap_or(dir::runtime()?);
        let config_path = config_path.unwrap_or(dir::config()?);

        let multiaddr = multiaddr!(Ip4(host), Tcp(port));

        let keypair = identity::Keypair::Ed25519(ed25519::Keypair::generate());
        let peer_id = PeerId::from(keypair.public());

        Ok(Self {
            peer_id,
            keypair,
            runtime_path,
            config_path,
            multiaddr,
        })
    }
}
