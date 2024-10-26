mod blockchain;
mod rede;
mod criptografia;
mod utils;

use blockchain::{Blockchain, Estudante, PeriodoLetivo, Transacao};
use rede::{iniciar_rede, P2PEvent};
use std::io::{self, BufRead};
use std::sync::Arc;
use tokio::sync::Mutex;
use rsa::RsaPrivateKey;

#[tokio::main]
async fn main() {
    let config = utils::config::Config::carregar_configuracao();
    let chave_privada_autoridade = criptografia::chaves::gerar_chave_privada_autoridade();
    let id_autoridade = 1;

    let blockchain = Arc::new(Mutex::new(Blockchain::nova_blockchain()));
    let (p2p_swarm, mut p2p_receiver) = iniciar_rede(blockchain.clone()).await;
    let p2p_swarm = Arc::new(Mutex::new(p2p_swarm));

    {
        let p2p_swarm = Arc::clone(&p2p_swarm);
        tokio::spawn(async move {
            p2p_swarm.lock().await.processar_eventos().await;
        });
    }

    println!("Digite o comando (ex: 'transacao' ou 'criar_bloco'):");

    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Erro ao ler entrada");
        let command = input.trim().to_string();

        match command.as_str() {
            "transacao" => {
                println!("Digite o ID da transação:");
                let mut id_transacao = String::new();
                io::stdin().read_line(&mut id_transacao).expect("Erro ao ler ID");
                let id_transacao: u32 = id_transacao.trim().parse().expect("ID inválido");

                println!("Digite o ID do estudante:");
                let mut id_estudante = String::new();
                io::stdin().read_line(&mut id_estudante).expect("Erro ao ler ID do estudante");
                let id_estudante: u32 = id_estudante.trim().parse().expect("ID inválido");

                println!("Digite o nome do estudante:");
                let mut nome_estudante = String::new();
                io::stdin().read_line(&mut nome_estudante).expect("Erro ao ler nome do estudante");

                println!("Digite o ano do período letivo:");
                let mut ano = String::new();
                io::stdin().read_line(&mut ano).expect("Erro ao ler ano");
                let ano: u32 = ano.trim().parse().expect("Ano inválido");

                println!("Digite o semestre do período letivo:");
                let mut semestre = String::new();
                io::stdin().read_line(&mut semestre).expect("Erro ao ler semestre");
                let semestre: u8 = semestre.trim().parse().expect("Semestre inválido");

                println!("Digite o mês de nascimento do estudante:");
                let mut mes = String::new();
                io::stdin().read_line(&mut mes).expect("Erro ao ler mês de nascimento");
                let mes: u32 = mes.trim().parse().expect("Mês inválido");

                println!("Digite o dia de nascimento do estudante:");
                let mut dia = String::new();
                io::stdin().read_line(&mut dia).expect("Erro ao ler dia de nascimento");
                let dia: u32 = dia.trim().parse().expect("Dia inválido");

                let estudante = Estudante::novo_estudante(
                    id_estudante,
                    &nome_estudante,
                    ano.try_into().unwrap(),
                    mes,
                    dia,
                );
                let periodo_letivo = PeriodoLetivo::novo_periodo(id_transacao, ano, semestre);
                let transacao = Transacao::nova_transacao(id_transacao, estudante, periodo_letivo);

                {
                    let mut bc = blockchain.lock().await;
                    bc.adicionar_transacao(transacao.clone());
                }

                p2p_swarm.lock().await.difundir_transacao(&transacao);
                println!("Transação criada e difundida.\nDigite o próximo comando:");
            }
            "criar_bloco" => {
                println!("Criando bloco...");
                let novo_bloco = {
                    let mut bc = blockchain.lock().await;
                    bc.criar_e_adicionar_bloco(&chave_privada_autoridade, id_autoridade)
                };
                p2p_swarm.lock().await.difundir_bloco(&novo_bloco);
                println!("Bloco criado e difundido.");
            }
            _ => println!("Comando desconhecido. Tente 'transacao' ou 'criar_bloco'."),
        }

        while let Ok(event) = p2p_receiver.try_recv() {
            match event {
                P2PEvent::NovoBloco(bloco_recebido) => {
                    println!("Bloco recebido: {:?}", bloco_recebido);
                    let mut bc = blockchain.lock().await;
                    bc.adicionar_bloco_externo(bloco_recebido, &config).expect("Erro ao adicionar bloco externo");
                }
                P2PEvent::NovaTransacao(transacao_recebida) => {
                    println!("Transação recebida: {:?}", transacao_recebida);
                    let mut bc = blockchain.lock().await;
                    bc.adicionar_transacao(transacao_recebida);
                }
                _ => {}
            }
        }
    }
}
