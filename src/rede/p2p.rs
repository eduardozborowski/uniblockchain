use libp2p::{
    development_transport,
    gossipsub::{
        Gossipsub, GossipsubEvent, IdentTopic as Topic,
        MessageAuthenticity, GossipsubConfig,
    },
    identity, PeerId, Swarm, Multiaddr, NetworkBehaviour,
};
use futures::prelude::*;
use serde_json;
use crate::blockchain::{Bloco, Transacao};

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "MyBehaviourEvent")]
pub struct MyBehaviour {
    pub gossipsub: Gossipsub,
}

#[derive(Debug)]
pub enum MyBehaviourEvent {
    Gossipsub(GossipsubEvent),
}

impl From<GossipsubEvent> for MyBehaviourEvent {
    fn from(event: GossipsubEvent) -> Self {
        MyBehaviourEvent::Gossipsub(event)
    }
}

pub enum P2PEvent {
    NovoBloco(Bloco),
    NovaTransacao(Transacao),
}

pub struct P2PSwarm {
    pub swarm: Swarm<MyBehaviour>,
    topic: Topic,
}

impl P2PSwarm {
    pub async fn new() -> Self {
        let local_key = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(local_key.public());

        println!("Peer ID local: {}", peer_id);

        let transport = development_transport(local_key.clone()).await.unwrap();
        let gossipsub_config = GossipsubConfig::default();
        let mut gossipsub = Gossipsub::new(
            MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )
            .unwrap();

        let topic = Topic::new("blockchain");
        gossipsub.subscribe(&topic).unwrap();

        let mut swarm = Swarm::new(transport, MyBehaviour { gossipsub }, peer_id);

        // Configuração de porta
        let porta = if std::env::args().any(|arg| arg == "--autoridade") {
            "4001"
        } else {
            "4002"
        };
        swarm
            .listen_on(format!("/ip4/0.0.0.0/tcp/{}", porta).parse().unwrap())
            .unwrap();

        // Conecta ao peer conhecido
        let outro_peer = if porta == "4001" {
            "/ip4/127.0.0.1/tcp/4002"
        } else {
            "/ip4/127.0.0.1/tcp/4001"
        };
        swarm
            .dial(outro_peer.parse::<Multiaddr>().unwrap())
            .unwrap_or_else(|e| println!("Falha ao conectar com peer: {:?}", e));

        P2PSwarm { swarm, topic }
    }

    pub async fn next_event(&mut self) -> Option<P2PEvent> {
        loop {
            match self.swarm.select_next_some().await {
                libp2p::swarm::SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(
                                                         GossipsubEvent::Message { message, .. },
                                                     )) => {
                    let data_str = String::from_utf8_lossy(&message.data);
                    if let Ok(bloco) = serde_json::from_str::<Bloco>(&data_str) {
                        return Some(P2PEvent::NovoBloco(bloco));
                    } else if let Ok(transacao) =
                        serde_json::from_str::<Transacao>(&data_str)
                    {
                        return Some(P2PEvent::NovaTransacao(transacao));
                    }
                }
                _ => {}
            }
        }
    }

    pub fn difundir_transacao(&mut self, transacao: &Transacao) {
        let data = serde_json::to_string(transacao).unwrap();
        if let Err(e) = self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.topic.clone(), data.as_bytes()) {
            println!("Erro ao difundir transação: {:?}", e);
        }
    }

    pub fn difundir_bloco(&mut self, bloco: &Bloco) {
        let data = serde_json::to_string(bloco).unwrap();
        if let Err(e) = self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.topic.clone(), data.as_bytes()) {
            println!("Erro ao difundir bloco: {:?}", e);
        }
    }
}

pub async fn iniciar_rede() -> P2PSwarm {
    P2PSwarm::new().await
}
