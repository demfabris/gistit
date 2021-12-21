//! The network module
#![allow(unused_variables)]
#![allow(dead_code)]

use libp2p::core::PeerId;
use libp2p::futures::StreamExt;
use libp2p::futures::{channel::mpsc, Stream};
use libp2p::ping;
use libp2p::swarm::{SwarmBuilder, SwarmEvent};
use libp2p::{development_transport, Swarm};
use libp2p::{identity, Multiaddr};

use crate::Result;

pub struct Network {
    pub client: Client,
    pub event_recv: Box<dyn Stream<Item = Event> + Send + Sync>,
    pub swarm: Swarm<ping::Behaviour>,
}

impl Network {
    /// # Errors
    ///
    /// asd
    pub async fn new(secret: &str) -> Result<Self> {
        let ed25519_keypair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(ed25519_keypair.public());

        let (command_sender, command_receiver) = mpsc::channel(0);
        let (event_sender, event_receiver) = mpsc::channel(0);

        let client = Client {
            sender: command_sender,
        };
        let event_recv = Box::new(event_receiver);

        let mut swarm = SwarmBuilder::new(
            development_transport(ed25519_keypair).await.unwrap(),
            ping::Behaviour::new(ping::PingConfig::new().with_keep_alive(true)),
            peer_id,
        )
        .build();

        swarm
            .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
            .unwrap();

        Ok(Self {
            client,
            event_recv,
            swarm,
        })
    }

    pub fn dial(mut self, addr: &str) -> Self {
        self.swarm.dial(addr.parse::<Multiaddr>().unwrap()).unwrap();
        self
    }

    pub async fn run(mut self) {
        loop {
            match self.swarm.select_next_some().await {
                other => println!("{:?}", other),
            };
        }
    }
}

#[derive(Clone)]
pub struct Client {
    sender: mpsc::Sender<Command>,
}

#[derive(Clone)]
pub enum Command {}

#[derive(Clone)]
pub enum Event {}
