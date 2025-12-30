//! libp2p网络行为模块
//! 
//! 定义libp2p网络行为和事件类型

use libp2p::{
    autonat,
    dcutr,
    gossipsub::{self, Behaviour as GossipsubBehaviour, Event as GossipsubEvent, IdentTopic as Topic},
    kad::{store::MemoryStore, Kademlia, KademliaEvent},
    mdns::{self, tokio::Behaviour as Mdns, Event as MdnsEvent},
    relay,
    swarm::NetworkBehaviour,
    PeerId,
};

/// libp2p网络行为组合
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "OutEvent")]
pub struct Behaviour {
    pub gossipsub: GossipsubBehaviour,
    pub relay: relay::Behaviour,
    pub autonat: autonat::Behaviour,
    pub dcutr: dcutr::Behaviour,
    pub mdns: Mdns,
    pub kademlia: Kademlia<MemoryStore>,
}

/// 网络行为输出事件
#[derive(Debug)]
pub enum OutEvent {
    Gossipsub(GossipsubEvent),
    Relay(relay::Event),
    Autonat(autonat::Event),
    Dcutr(dcutr::Event),
    Mdns(MdnsEvent),
    Kademlia(KademliaEvent),
}

impl From<GossipsubEvent> for OutEvent {
    fn from(v: GossipsubEvent) -> Self {
        OutEvent::Gossipsub(v)
    }
}

impl From<relay::Event> for OutEvent {
    fn from(v: relay::Event) -> Self {
        OutEvent::Relay(v)
    }
}

impl From<autonat::Event> for OutEvent {
    fn from(v: autonat::Event) -> Self {
        OutEvent::Autonat(v)
    }
}

impl From<dcutr::Event> for OutEvent {
    fn from(v: dcutr::Event) -> Self {
        OutEvent::Dcutr(v)
    }
}

impl From<MdnsEvent> for OutEvent {
    fn from(v: MdnsEvent) -> Self {
        OutEvent::Mdns(v)
    }
}

impl From<KademliaEvent> for OutEvent {
    fn from(v: KademliaEvent) -> Self {
        OutEvent::Kademlia(v)
    }
}
