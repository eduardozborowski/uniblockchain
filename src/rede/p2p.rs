use libp2p::{
    development_transport,
    gossipsub::{Gossipsub, GossipsubEvent, IdentTopic as Topic, MessageAuthenticity},
    identity, swarm::{SwarmEvent}, PeerId, Swarm, Multiaddr, NetworkBehaviour
};
use tokio::sync::{mpsc, Mutex};
use futures::StreamExt;
use serde_json::Value;
use crate::blockchain::{Bloco, Transacao};
use std::sync::Arc;

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

impl MyBehaviour {
    pub async fn new(id_keys: identity::Keypair) -> Self {
        let gossipsub_config = libp2p::gossipsub::GossipsubConfig::default();
        let gossipsub: Gossipsub = Gossipsub::new(
            MessageAuthenticity::Signed(id_keys.clone()),
            gossipsub_config,
        ).unwrap();

        MyBehaviour { gossipsub }
    }
}

pub enum P2PEvent {
    NovoBloco(Bloco),
    NovaTransacao(Transacao),
}

pub struct P2PSwarm {
    pub swarm: Swarm<MyBehaviour>,
    topic: Topic,
    peer_id: PeerId,
    sender: mpsc::Sender<P2PEvent>,
}

impl P2PSwarm {
    pub async fn new(
        blockchain: Arc<Mutex<crate::blockchain::Blockchain>>
    ) -> (Self, mpsc::Receiver<P2PEvent>) {
        let local_key = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(local_key.public());

        println!("Peer ID local: {}", peer_id);

        let transport = development_transport(local_key.clone()).await.unwrap();
        let mut behaviour = MyBehaviour::new(local_key.clone()).await;

        let topic = Topic::new("blockchain");
        behaviour.gossipsub.subscribe(&topic).unwrap();

        let mut swarm = Swarm::new(transport, behaviour, peer_id.clone());

        // Configuração de porta
        let porta = if std::env::args().any(|arg| arg == "--autoridade") {
            "4001"
        } else {
            "4002"
        };
        swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", porta).parse().unwrap()).unwrap();

        // Configuração de conexão com peer conhecido
        let peer_address = if porta == "4001" {
            "/ip4/127.0.0.1/tcp/4002"
        } else {
            "/ip4/127.0.0.1/tcp/4001"
        };
        swarm.dial(peer_address.parse::<Multiaddr>().unwrap()).expect("Falha ao conectar com peer");

        let (sender, receiver) = mpsc::channel(100);

        (Self { swarm, topic, peer_id, sender }, receiver)
    }

    pub async fn processar_eventos(&mut self) {
        while let Some(event) = self.swarm.next().await {
            match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Escutando em {:?}", address);
                }
                SwarmEvent::Behaviour(event) => match event {
                    MyBehaviourEvent::Gossipsub(GossipsubEvent::Message { message, .. }) => {
                        let data_str = String::from_utf8_lossy(&message.data);
                        println!("Mensagem recebida: {}", data_str);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    pub fn difundir_transacao(&mut self, transacao: &Transacao) {
        let data = serde_json::to_string(transacao).unwrap();
        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.topic.clone(), data.as_bytes())
            .unwrap();
    }

    pub fn difundir_bloco(&mut self, bloco: &Bloco) {
        let data = serde_json::to_string(bloco).unwrap();
        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.topic.clone(), data.as_bytes())
            .unwrap();
    }
}

pub async fn iniciar_rede(
    blockchain: Arc<tokio::sync::Mutex<crate::blockchain::Blockchain>>
) -> (P2PSwarm, mpsc::Receiver<P2PEvent>) {
    P2PSwarm::new(blockchain).await
}
