use crate::turtle::event::Event;
use libp2p::identity::Keypair;
use libp2p::Swarm;
use tokio::select;
use tokio::sync::mpsc::{self, Receiver, Sender};

pub mod behavior;
pub mod event;
pub mod msg;
pub mod transport;

pub struct Turtle {
    event_rx: Receiver<Event>,
    event_tx: Sender<Event>,
    key: Keypair,
    swarm: Option<Swarm<behavior::Behavior>>,
}

impl Default for Turtle {
    fn default() -> Self {
        let (event_tx, event_rx) = mpsc::channel(1);
        let key = Keypair::generate_ed25519();

        Self {
            event_rx,
            event_tx,
            key,
            swarm: None,
        }
    }
}

impl Turtle {
    pub async fn run(&self) {
        loop {}
    }

    pub async fn stop(&self) {
        self.send_event_not_blocking(Event::Stop).await
    }

    async fn send_event_not_blocking(&self, evt: Event) {
        select! {
            _ = self.event_tx.send(evt) => (),
            else => (),
        }
    }
}
