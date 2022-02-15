use std::fmt::Debug;
use std::fs;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use libp2p::core::{Multiaddr, PeerId};
use libp2p::identity::{self, ed25519, Keypair};
use libp2p::multiaddr::multiaddr;

use log::{debug, info};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

use gistit_reference::project;

use crate::{Error, Result};

pub struct Config {
    pub peer_id: PeerId,
    pub keypair: Keypair,
    pub runtime_path: PathBuf,
    pub config_path: PathBuf,
    pub multiaddr: Multiaddr,
    pub bootstrap: bool,
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
        config_file: Option<PathBuf>,
        host: Option<Ipv4Addr>,
        port: Option<u16>,
        bootstrap: bool,
    ) -> Result<Self> {
        project::path::init()?;

        let host = host.unwrap_or(Ipv4Addr::new(0, 0, 0, 0));
        let port = port.unwrap_or(0_u16);
        let multiaddr = multiaddr!(Ip4(host), Tcp(port));

        let runtime_path = runtime_path.unwrap_or(project::path::runtime()?);
        let config_path = config_path.unwrap_or(project::path::config()?);
        let node_config = config_file.unwrap_or_else(|| config_path.join("node-config"));

        let (peer_id, keypair) = if fs::metadata(&node_config).is_ok() {
            debug!("Using existing node config file");
            let config = Zeroizing::new(NodeKey::from_file(&node_config)?);

            let keypair = identity::Keypair::from_protobuf_encoding(&Zeroizing::new(
                base64::decode(config.identity.priv_key.as_bytes())?,
            ))?;

            let peer_id = keypair.public().into();
            assert_eq!(
                    PeerId::from_str(&config.identity.peer_id)
                    .map_err(|_| Error::Parse("failed to parse config peer id"))?,
                    peer_id,
                    "Expect peer id derived from private key and peer id retrieved from config to match."
                );
            (peer_id, keypair)
        } else {
            debug!("Generating new node key material");
            let keypair = identity::Keypair::Ed25519(ed25519::Keypair::generate());
            let peer_id = PeerId::from(keypair.public());

            // Storing generated key material
            let key_material = NodeKey::from_key_material(peer_id, &keypair)?;
            fs::write(node_config, serde_json::to_string(&key_material)?)?;

            (peer_id, keypair)
        };
        info!("{:?}", peer_id);

        Ok(Self {
            peer_id,
            keypair,
            runtime_path,
            config_path,
            multiaddr,
            bootstrap,
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NodeKey {
    pub identity: Identity,
}

impl NodeKey {
    pub fn from_key_material(peer_id: PeerId, keypair: &Keypair) -> Result<Self> {
        let priv_key = base64::encode(keypair.to_protobuf_encoding()?);
        let peer_id = peer_id.to_base58();
        Ok(Self {
            identity: Identity { peer_id, priv_key },
        })
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Identity {
    #[serde(rename = "PeerID")]
    pub peer_id: String,
    pub priv_key: String,
}

impl Zeroize for NodeKey {
    fn zeroize(&mut self) {
        self.identity.peer_id.zeroize();
        self.identity.priv_key.zeroize();
    }
}
