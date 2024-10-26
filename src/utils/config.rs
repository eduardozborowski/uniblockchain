// src/utils/config.rs

use serde::{Serialize, Deserialize};
use std::fs;
use rsa::RsaPublicKey;
use rsa::pkcs1::DecodeRsaPublicKey;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Autoridade {
    pub id: u32,
    pub nome: String,
    pub chave_publica_pem: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub autoridades: Vec<Autoridade>,
}

impl Config {
    pub fn carregar_configuracao() -> Self {
        // Carrega a configuração de um arquivo JSON ou inicializa manualmente
        // Exemplo simplificado:
        let autoridades = vec![
            Autoridade {
                id: 1,
                nome: "Autoridade1".to_string(),
                chave_publica_pem: fs::read_to_string("chaves_publicas/autoridade_1.pem").expect("Falha ao carregar chave pública"),
            },
            // Adicione outras autoridades conforme necessário
        ];

        Config { autoridades }
    }

    pub fn obter_chave_publica(&self, id: u32) -> Option<RsaPublicKey> {
        self.autoridades
            .iter()
            .find(|auth| auth.id == id)
            .and_then(|auth| RsaPublicKey::from_pkcs1_pem(&auth.chave_publica_pem).ok())
    }
}
