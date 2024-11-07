
use rsa::RsaPrivateKey;
use rsa::pkcs8::DecodePrivateKey;
use std::fs;

pub fn carregar_chave_privada(id_autoridade: u32) -> RsaPrivateKey {
    let caminho = format!("chaves_privadas/autoridade_{}.pem", id_autoridade);
    let chave_privada_pem = fs::read_to_string(&caminho)
        .unwrap_or_else(|err| panic!("Falha ao ler a chave privada da autoridade no caminho '{}': {}", caminho, err));

    RsaPrivateKey::from_pkcs8_pem(&chave_privada_pem)
        .expect("Falha ao parsear a chave privada da autoridade")
}
