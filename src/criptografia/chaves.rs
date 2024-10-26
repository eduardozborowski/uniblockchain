// src/criptografia/chaves.rs

use rsa::{RsaPrivateKey, RsaPublicKey};
use rand::rngs::OsRng;
use std::fs;
use rsa::pkcs1::{DecodeRsaPublicKey};

pub fn gerar_chave_privada_autoridade() -> RsaPrivateKey {
    // Aqui você pode carregar a chave privada de um arquivo seguro
    // Para fins de exemplo, vamos gerar uma nova chave a cada execução
    let bits = 2048;
    RsaPrivateKey::new(&mut OsRng, bits).expect("Falha ao gerar chave privada")
}

pub fn carregar_chave_publica_autoridade(id_autoridade: u32) -> Option<RsaPublicKey> {
    // Carrega a chave pública da autoridade a partir de um arquivo ou configuração
    // Exemplo simplificado:
    let chave_publica_pem = fs::read_to_string(format!("chaves_publicas/autoridade_{}.pem", id_autoridade)).ok()?;
    RsaPublicKey::from_pkcs1_pem(&chave_publica_pem).ok()
}
