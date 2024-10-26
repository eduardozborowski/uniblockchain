use std::collections::HashMap;
use rsa::RsaPublicKey;
use rsa::pkcs8::DecodePublicKey; // Import necessário para from_public_key_pem
use std::fs;
use toml::Value;

#[derive(Debug)]
pub struct Config {
    pub chaves_publicas: HashMap<u32, RsaPublicKey>,
}

impl Config {
    pub fn carregar_configuracao() -> Self {
        let conteudo = fs::read_to_string("config.toml").expect("Não foi possível ler o arquivo config.toml");
        let value = conteudo.parse::<Value>().expect("Erro ao parsear o arquivo config.toml");

        let mut chaves_publicas = HashMap::new();

        if let Some(autoridades) = value.get("autoridades").and_then(|v| v.as_table()) {
            for (id_str, chave_pem) in autoridades {
                let id_autoridade: u32 = id_str.parse().expect("ID da autoridade inválido");
                let chave_pem = chave_pem.as_str().expect("Chave PEM inválida");

                let chave_publica = RsaPublicKey::from_public_key_pem(chave_pem)
                    .expect("Erro ao carregar chave pública da autoridade");

                chaves_publicas.insert(id_autoridade, chave_publica);
            }
        }

        Config { chaves_publicas }
    }

    pub fn obter_chave_publica(&self, id_autoridade: u32) -> Option<&RsaPublicKey> {
        self.chaves_publicas.get(&id_autoridade)
    }
}
