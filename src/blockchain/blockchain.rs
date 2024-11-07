use super::{Bloco, Transacao};
use std::collections::VecDeque;
use crate::utils::config::Config;
use crate::utils::erros::BlocoErro;
use rsa::RsaPrivateKey;
use chrono::{Utc, TimeZone};
use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{Write, Read};

#[derive(Debug, Serialize, Deserialize)]
pub struct Blockchain {
    pub cadeia: Vec<Bloco>,
    #[serde(skip)]
    pub transacoes_pendentes: VecDeque<Transacao>,
}

impl Blockchain {
    pub fn nova_blockchain() -> Self {
        let timestamp_genesis = Utc.timestamp_opt(0, 0).unwrap();
        let bloco_genesis = Bloco::novo_bloco(
            0,
            String::from("0"),
            Vec::new(),
            Some(timestamp_genesis),
        );
        let mut blockchain = Blockchain {
            cadeia: vec![bloco_genesis],
            transacoes_pendentes: VecDeque::new(),
        };
        blockchain.cadeia[0].hash_atual = blockchain.cadeia[0].calcular_hash();
        blockchain
    }

    pub fn adicionar_transacao(&mut self, transacao: Transacao) {
        self.transacoes_pendentes.push_back(transacao);
    }

    pub fn criar_e_adicionar_bloco(
        &mut self,
        chave_privada: &RsaPrivateKey,
        id_autoridade: u32,
    ) -> Bloco {
        let indice = self.cadeia.len() as u32;
        let hash_anterior = self.cadeia.last().unwrap().hash_atual.clone();
        let transacoes = self.transacoes_pendentes.drain(..).collect();

        let mut novo_bloco = Bloco {
            indice,
            hash_anterior,
            hash_atual: String::new(),
            timestamp: Utc::now(),
            transacoes,
            id_autoridade,
            assinatura_autoridade: String::new(),
        };

        novo_bloco.assinar_bloco(chave_privada, id_autoridade);
        self.cadeia.push(novo_bloco.clone());
        novo_bloco
    }

    pub fn adicionar_bloco_externo(&mut self, bloco: Bloco, config: &Config) -> Result<(), BlocoErro> {
        // Verifica se o hash anterior corresponde
        if bloco.hash_anterior != self.cadeia.last().unwrap().hash_atual {
            return Err(BlocoErro::HashAnteriorNaoCorresponde);
        }

        // Verifica a assinatura do bloco
        bloco.verificar_assinatura(config)?;

        // Adiciona o bloco à cadeia
        self.cadeia.push(bloco);
        Ok(())
    }

    pub fn salvar_em_disco(&self, caminho: &str) -> std::io::Result<()> {
        let dados = serde_json::to_string(&self).unwrap();
        let mut arquivo = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(caminho)?;
        arquivo.write_all(dados.as_bytes())?;
        Ok(())
    }

    pub fn carregar_do_disco(caminho: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut arquivo = File::open(caminho)?;
        let mut dados = String::new();
        arquivo.read_to_string(&mut dados)?;
        if dados.trim().is_empty() {
            Err("Arquivo de blockchain vazio".into())
        } else {
            let mut blockchain: Blockchain = serde_json::from_str(&dados)?;
            blockchain.transacoes_pendentes = VecDeque::new();
            Ok(blockchain)
        }
    }
}
