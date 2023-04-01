use crate::turtle::msg;
use libp2p::core::PublicKey;
use libp2p::identify;
use libp2p::kad::store::MemoryStore;
use libp2p::kad::Kademlia;
use libp2p::mdns;
use libp2p::ping;
use libp2p::request_response::{ProtocolSupport, RequestResponse};
use libp2p::swarm::NetworkBehaviour;
use std::iter;

#[derive(NetworkBehaviour)]
pub(crate) struct Behavior {
    pub mdns: mdns::tokio::Behaviour,
    pub ping: ping::Behaviour,
    pub identify: identify::Behaviour,
    pub kad: Kademlia<MemoryStore>,
    pub request_response: RequestResponse<msg::Codec>,
}

impl Behavior {
    pub fn new(pubkey: &PublicKey) -> Self {
        let mdns = mdns::tokio::Behaviour::new(Default::default())
            .expect("construct ping behavior failed");
        let ping = ping::Behaviour::default();
        let identify = identify::Behaviour::new(identify::Config::new(
            String::from("/send-msg-id/1.0.0"),
            pubkey.clone(),
        ));
        let kad = Kademlia::new(pubkey.to_peer_id(), MemoryStore::new(pubkey.to_peer_id()));
        let request_response = RequestResponse::new(
            msg::Codec,
            iter::once((msg::MsgProto, ProtocolSupport::Full)),
            Default::default(),
        );

        Self {
            mdns,
            ping,
            identify,
            kad,
            request_response,
        }
    }
}
