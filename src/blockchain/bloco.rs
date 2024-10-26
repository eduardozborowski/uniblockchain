// src\blockchain\bloco.rs
use base64::{decode, encode};
use chrono::{DateTime, Utc};
use rsa::pkcs1v15::{SigningKey, VerifyingKey, Signature as RsaSignature};
use rsa::RsaPrivateKey;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use signature::{Signer, Verifier};

use crate::utils::config::Config;
use crate::utils::erros::BlocoErro;
use super::Transacao;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bloco {
    pub indice: u32,
    pub hash_anterior: String,
    pub hash_atual: String,
    pub timestamp: DateTime<Utc>,
    pub transacoes: Vec<Transacao>,
    pub id_autoridade: u32,
    pub assinatura_autoridade: String,
}

impl Bloco {
    pub fn novo_bloco(
        indice: u32,
        hash_anterior: String,
        transacoes: Vec<Transacao>,
        timestamp: Option<DateTime<Utc>>,
    ) -> Self {
        Bloco {
            indice,
            hash_anterior,
            hash_atual: String::new(),
            timestamp: timestamp.unwrap_or_else(|| Utc::now()),
            transacoes,
            id_autoridade: 0,
            assinatura_autoridade: String::new(),
        }
    }

    pub fn calcular_hash(&self) -> String {
        let mut bloco_clone = self.clone();
        bloco_clone.hash_atual = String::new(); // Evitar loop infinito
        let bloco_serializado = serde_json::to_string(&bloco_clone).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(bloco_serializado.as_bytes());
        let resultado = hasher.finalize();
        format!("{:x}", resultado)
    }

    pub fn assinar_bloco(&mut self, chave_privada: &RsaPrivateKey, id_autoridade: u32) {
        self.id_autoridade = id_autoridade;
        self.hash_atual = self.calcular_hash();

        let dados_para_assinar = format!(
            "{}{}{}{}{:?}",
            self.indice,
            self.hash_anterior,
            self.hash_atual,
            self.timestamp,
            self.transacoes
        );

        let signing_key = SigningKey::<Sha256>::new(chave_privada.clone());

        let assinatura = signing_key.sign(dados_para_assinar.as_bytes());
        self.assinatura_autoridade = encode(assinatura.as_ref());
    }

    pub fn verificar_assinatura(
        &self,
        config: &Config,
    ) -> Result<(), BlocoErro> {
        let chave_publica = config
            .obter_chave_publica(self.id_autoridade)
            .ok_or(BlocoErro::AutoridadeDesconhecida)?;

        let dados_assinados = format!(
            "{}{}{}{}{:?}",
            self.indice,
            self.hash_anterior,
            self.hash_atual,
            self.timestamp,
            self.transacoes
        );

        let assinatura_bytes = decode(&self.assinatura_autoridade).map_err(|_| BlocoErro::AssinaturaInvalida)?;
        let assinatura = RsaSignature::from(assinatura_bytes.into_boxed_slice());

        let verifying_key = VerifyingKey::<Sha256>::new(chave_publica);

        verifying_key
            .verify(dados_assinados.as_bytes(), &assinatura)
            .map_err(|_| BlocoErro::AssinaturaInvalida)
    }
}
