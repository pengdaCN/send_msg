use libp2p::core::identity::Keypair;
use libp2p::core::muxing::StreamMuxerBox;
use libp2p::core::transport::Boxed;
use libp2p::core::Multiaddr;
use libp2p::core::{upgrade, PublicKey};
use libp2p::futures::FutureExt;
use libp2p::futures::{select, StreamExt};
use libp2p::kad::store::MemoryStore;
use libp2p::kad::Kademlia;
use libp2p::mdns;
use libp2p::noise::NoiseAuthenticated;
use libp2p::ping;
use libp2p::request_response::{ProtocolSupport, RequestResponse};
use libp2p::swarm::SwarmEvent;
use libp2p::swarm::{ConnectionHandler, IntoConnectionHandler, NetworkBehaviour};
use libp2p::tcp::tokio::Transport as TcpTransport;
use libp2p::Swarm;
use libp2p::{identify, yamux, PeerId, Transport};
use std::iter;
use tokio::io;
use tokio::io::AsyncBufReadExt;
use tracing::{error, info, warn};

pub mod msg;

#[tokio::main]
async fn main() {
    let id_keys = Keypair::generate_ed25519();

    let transport = build_transport(&id_keys);
    let pubkey = id_keys.public();
    let behavior = Behavior::new(&pubkey);

    let mut swarm = Swarm::with_tokio_executor(transport, behavior, pubkey.to_peer_id());

    let mut lines = io::BufReader::new(io::stdin()).lines();
    loop {
        select! {
            line = lines.next_line().fuse() => {
                if let Some(x) = line.ok().flatten() {
                    handle_command(&x, &mut swarm);
                }
            }

            event = swarm.select_next_some() => {
                handle_swarm_event(event, &mut swarm);
            }

        }
    }
}

fn handle_command(line: &str, swarm: &mut Swarm<Behavior>) {
    let mut args = line.split_whitespace();

    let Some(command) = args.next().map(|x| x.trim()) else {
        return;
    };

    match command {
        "DIAL" => {
            handle_command_dial(&mut args, swarm);
        }
        "LISTEN" => {
            handle_command_listen(&mut args, swarm);
        }
        "SEND" => {
            handle_command_send(&mut args, swarm);
        }
        "SHOW" => {
            handle_command_show(&mut args, swarm);
        }
        _ => {}
    };
}

fn handle_command_dial<'a, T>(args: &'a mut T, swarm: &mut Swarm<Behavior>)
where
    T: Iterator<Item = &'a str>,
{
}

fn handle_command_listen<'a, T>(args: &'a mut T, swarm: &mut Swarm<Behavior>)
where
    T: Iterator<Item = &'a str>,
{
    let Some(addr) = args.next().map(|x| x.trim())
        .and_then(|x| x.parse::<Multiaddr>().ok()) else {
        warn!("地址不能解析Multiaddr");
        return;
    };

    if let Err(e) = swarm.listen_on(addr) {
        warn!("建立监听失败 {e:?}");
    }
}

fn handle_command_show<'a, T>(args: &'a mut T, swarm: &mut Swarm<Behavior>)
where
    T: Iterator<Item = &'a str>,
{
    let Some(name) = args.next().map(|x| x.trim()) else {
        error!("必须要一个name");
        return;
    };

    match name {
        "PEER" => {
            println!("PEER: {}", swarm.local_peer_id().to_string());
        }
        _ => {}
    }
}

fn handle_command_send<'a, T>(args: &'a mut T, swarm: &mut Swarm<Behavior>)
where
    T: Iterator<Item = &'a str>,
{
}

fn handle_swarm_event(
    event: SwarmEvent<
        <Behavior as NetworkBehaviour>::OutEvent,
        <<<
        Behavior as NetworkBehaviour>::ConnectionHandler
        as IntoConnectionHandler>::Handler
        as ConnectionHandler>::Error
    >,
    swarm: &mut Swarm<Behavior>,
) {
    match event {
        SwarmEvent::Behaviour(behavior_event) => {
            handle_behavior_event(behavior_event, swarm);
        }
        e => {
            info!("swarm事件 {e:?}");
        }
    }
}

fn handle_behavior_event(event: BehaviorEvent, swarm: &mut Swarm<Behavior>) {}

#[derive(NetworkBehaviour)]
struct Behavior {
    mdns: mdns::tokio::Behaviour,
    ping: ping::Behaviour,
    identify: identify::Behaviour,
    kad: Kademlia<MemoryStore>,
    request_response: RequestResponse<msg::Codec>,
}

impl Behavior {
    fn new(pubkey: &PublicKey) -> Self {
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

fn build_transport(k: &Keypair) -> Boxed<(PeerId, StreamMuxerBox)> {
    TcpTransport::default()
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseAuthenticated::xx(k).expect("build transport failed because of noise"))
        .multiplex(yamux::YamuxConfig::default())
        .boxed()
}
