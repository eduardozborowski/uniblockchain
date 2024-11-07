use libp2p::{
    development_transport,
    gossipsub::{
        Gossipsub, GossipsubEvent, IdentTopic as Topic,
        MessageAuthenticity, GossipsubConfig,
    },
    identity, PeerId, Swarm, Multiaddr, NetworkBehaviour,
    request_response::{
        RequestResponse, RequestResponseCodec, RequestResponseEvent,
        RequestResponseMessage, ProtocolName, ProtocolSupport,
        RequestResponseConfig, ResponseChannel,
    },
    swarm::SwarmEvent,
};
use futures::prelude::*;
use serde_json;
use crate::blockchain::{Bloco, Transacao};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::io;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use std::io::{Error as IoError};

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "MyBehaviourEvent")]
pub struct MyBehaviour {
    pub gossipsub: Gossipsub,
    pub request_response: RequestResponse<BlockchainExchangeCodec>,
}

#[derive(Debug)]
pub enum MyBehaviourEvent {
    Gossipsub(GossipsubEvent),
    RequestResponse(RequestResponseEvent<BlockchainRequest, BlockchainResponse>),
}

impl From<GossipsubEvent> for MyBehaviourEvent {
    fn from(event: GossipsubEvent) -> Self {
        MyBehaviourEvent::Gossipsub(event)
    }
}

impl From<RequestResponseEvent<BlockchainRequest, BlockchainResponse>> for MyBehaviourEvent {
    fn from(event: RequestResponseEvent<BlockchainRequest, BlockchainResponse>) -> Self {
        MyBehaviourEvent::RequestResponse(event)
    }
}

pub enum P2PEvent {
    NovoBloco(Bloco),
    NovaTransacao(Transacao),
    BlockchainSolicitada {
        peer: PeerId,
        channel: ResponseChannel<BlockchainResponse>,
    },
    BlockchainRecebida(Vec<Bloco>),
}

pub struct P2PSwarm {
    pub swarm: Swarm<MyBehaviour>,
    topic: Topic,
}

impl P2PSwarm {
    pub async fn new(_blockchain: Arc<Mutex<crate::blockchain::Blockchain>>) -> Self {
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

        // Configuração do RequestResponse
        let protocols = std::iter::once((
            BlockchainExchangeProtocol(),
            ProtocolSupport::Full,
        ));
        let cfg = RequestResponseConfig::default();
        let request_response = RequestResponse::new(BlockchainExchangeCodec(), protocols, cfg);

        let mut swarm = Swarm::new(
            transport,
            MyBehaviour { gossipsub, request_response },
            peer_id,
        );

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
                SwarmEvent::Behaviour(event) => match event {
                    MyBehaviourEvent::Gossipsub(GossipsubEvent::Message { message, .. }) => {
                        let data_str = String::from_utf8_lossy(&message.data);
                        if let Ok(bloco) = serde_json::from_str::<Bloco>(&data_str) {
                            return Some(P2PEvent::NovoBloco(bloco));
                        } else if let Ok(transacao) =
                            serde_json::from_str::<Transacao>(&data_str)
                        {
                            return Some(P2PEvent::NovaTransacao(transacao));
                        }
                    }
                    MyBehaviourEvent::RequestResponse(event) => {
                        match event {
                            RequestResponseEvent::Message { peer, message } => {
                                match message {
                                    RequestResponseMessage::Request { request, channel, .. } => {
                                        match request {
                                            BlockchainRequest::SolicitacaoBlockchain => {
                                                // Envia a blockchain em resposta
                                                return Some(P2PEvent::BlockchainSolicitada {
                                                    peer,
                                                    channel,
                                                });
                                            }
                                        }
                                    }
                                    RequestResponseMessage::Response { response, .. } => {
                                        match response {
                                            BlockchainResponse::Blockchain(cadeia) => {
                                                return Some(P2PEvent::BlockchainRecebida(cadeia));
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    pub fn solicitar_blockchain(&mut self) {
        let peers: Vec<_> = self.swarm.behaviour().gossipsub.all_mesh_peers().cloned().collect();
        for peer in peers {
            self.swarm.behaviour_mut().request_response.send_request(
                &peer,
                BlockchainRequest::SolicitacaoBlockchain,
            );
        }
    }

    pub fn enviar_blockchain(&mut self, cadeia: &Vec<Bloco>, channel: ResponseChannel<BlockchainResponse>) {
        let response = BlockchainResponse::Blockchain(cadeia.clone());
        if let Err(e) = self.swarm.behaviour_mut().request_response.send_response(
            channel,
            response,
        ) {
            println!("Erro ao enviar blockchain: {:?}", e);
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

// Implementação dos tipos e codecs para RequestResponse

#[derive(Clone)]
pub struct BlockchainExchangeProtocol();

impl ProtocolName for BlockchainExchangeProtocol {
    fn protocol_name(&self) -> &[u8] {
        b"/blockchain-exchange/1.0.0"
    }
}

#[derive(Clone)]
pub struct BlockchainExchangeCodec();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockchainRequest {
    SolicitacaoBlockchain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockchainResponse {
    Blockchain(Vec<Bloco>),
}

#[async_trait]
impl RequestResponseCodec for BlockchainExchangeCodec {
    type Protocol = BlockchainExchangeProtocol;
    type Request = BlockchainRequest;
    type Response = BlockchainResponse;

    async fn read_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> Result<Self::Request, IoError>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        serde_json::from_slice(&buf).map_err(|e| IoError::new(io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> Result<Self::Response, IoError>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        serde_json::from_slice(&buf).map_err(|e| IoError::new(io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        request: Self::Request,
    ) -> Result<(), IoError>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let buf = serde_json::to_vec(&request).map_err(|e| IoError::new(io::ErrorKind::InvalidData, e))?;
        io.write_all(&buf).await?;
        io.close().await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        response: Self::Response,
    ) -> Result<(), IoError>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let buf = serde_json::to_vec(&response).map_err(|e| IoError::new(io::ErrorKind::InvalidData, e))?;
        io.write_all(&buf).await?;
        io.close().await?;
        Ok(())
    }
}

pub async fn iniciar_rede(blockchain: Arc<Mutex<crate::blockchain::Blockchain>>) -> P2PSwarm {
    P2PSwarm::new(blockchain).await
}
