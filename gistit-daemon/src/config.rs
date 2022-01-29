use std::path::PathBuf;

use libp2p::core::PeerId;
use libp2p::identity::{self, ed25519, Keypair};

pub struct Config {
    pub peer_id: PeerId,
    pub keypair: Keypair,
    pub runtime_dir: PathBuf,
    pub config_dir: PathBuf,
}

impl Config {
    pub fn new(runtime_dir: PathBuf, config_dir: PathBuf) -> Self {
        // TODO: generate and storing peer id happens here
        let keypair = identity::Keypair::Ed25519(ed25519::Keypair::generate());
        let peer_id = PeerId::from(keypair.public());

        Self {
            peer_id,
            keypair,
            runtime_dir,
            config_dir,
        }
    }
}
