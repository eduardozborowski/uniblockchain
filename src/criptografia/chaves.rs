// src/criptografia/chaves.rs

use rsa::RsaPrivateKey;
use rsa::pkcs1::DecodeRsaPrivateKey;
use std::fs;

pub fn carregar_chave_privada(id_autoridade: u32) -> RsaPrivateKey {
    let caminho = format!("chaves_privadas/autoridade_{}.pem", id_autoridade);
    let chave_privada_pem = fs::read_to_string(&caminho)
        .expect("Falha ao ler a chave privada da autoridade");

    RsaPrivateKey::from_pkcs1_pem(&chave_privada_pem)
        .expect("Falha ao parsear a chave privada da autoridade")
}
