mod blockchain;
mod rede;
mod criptografia;
mod utils;

use blockchain::{Blockchain, Estudante, PeriodoLetivo, Transacao};
use rede::{iniciar_rede, P2PEvent};
use std::sync::Arc;
use tokio::sync::Mutex;
use criptografia::chaves::carregar_chave_privada;
use tokio::io::{self, AsyncBufReadExt};

#[tokio::main]
async fn main() {
    let config = utils::config::Config::carregar_configuracao();
    println!("Chaves públicas carregadas: {:?}", config.chaves_publicas.keys());

    // Determina se o nó é autoridade com base no argumento de linha de comando
    let is_autoridade = std::env::args().any(|arg| arg == "--autoridade");
    let id_autoridade = if is_autoridade { 1 } else { 0 };

    // Carrega a chave privada somente se for autoridade
    let chave_privada_autoridade = if is_autoridade {
        Some(carregar_chave_privada(id_autoridade))
    } else {
        None
    };

    let blockchain = Arc::new(Mutex::new(Blockchain::nova_blockchain()));
    let mut p2p_swarm = iniciar_rede().await;

    let stdin = io::BufReader::new(tokio::io::stdin());
    let mut stdin_lines = stdin.lines();

    println!("Digite o comando (ex: 'transacao' ou 'criar_bloco'):");

    loop {
        tokio::select! {
            event = p2p_swarm.next_event() => {
                if let Some(event) = event {
                    match event {
                        P2PEvent::NovoBloco(bloco_recebido) => {
                            println!("Bloco recebido: {:?}", bloco_recebido);
                            let mut bc = blockchain.lock().await;
                            if let Err(e) = bc.adicionar_bloco_externo(bloco_recebido, &config) {
                                println!("Erro ao adicionar bloco externo: {:?}", e);
                            }
                        }
                        P2PEvent::NovaTransacao(transacao_recebida) => {
                            println!("Transação recebida: {:?}", transacao_recebida);
                            let mut bc = blockchain.lock().await;
                            bc.adicionar_transacao(transacao_recebida);
                        }
                    }
                }
            }
            Ok(Some(line)) = stdin_lines.next_line() => {
                let command = line.trim().to_string();
                match command.as_str() {
                    "transacao" => {
                        // Leitura dos dados da transação
                        println!("Digite o ID da transação:");
                        let id_transacao = ler_u32_async(&mut stdin_lines).await;
                        println!("Digite o ID do estudante:");
                        let id_estudante = ler_u32_async(&mut stdin_lines).await;
                        println!("Digite o nome do estudante:");
                        let nome_estudante = ler_string_async(&mut stdin_lines).await;
                        println!("Digite o ano de nascimento do estudante:");
                        let ano_nascimento = ler_i32_async(&mut stdin_lines).await;
                        println!("Digite o mês de nascimento do estudante:");
                        let mes_nascimento = ler_u32_async(&mut stdin_lines).await;
                        println!("Digite o dia de nascimento do estudante:");
                        let dia_nascimento = ler_u32_async(&mut stdin_lines).await;
                        println!("Digite o ano do período letivo:");
                        let ano_periodo = ler_u32_async(&mut stdin_lines).await;
                        println!("Digite o semestre do período letivo:");
                        let semestre = ler_u8_async(&mut stdin_lines).await;

                        let estudante = Estudante::novo_estudante(
                            id_estudante,
                            &nome_estudante,
                            ano_nascimento,
                            mes_nascimento,
                            dia_nascimento,
                        );
                        let periodo_letivo = PeriodoLetivo::novo_periodo(
                            id_transacao,
                            ano_periodo,
                            semestre,
                        );
                        let transacao = Transacao::nova_transacao(
                            id_transacao,
                            estudante,
                            periodo_letivo,
                        );

                        {
                            let mut bc = blockchain.lock().await;
                            bc.adicionar_transacao(transacao.clone());
                        }

                        p2p_swarm.difundir_transacao(&transacao);
                        println!("Transação criada e difundida.\nDigite o próximo comando:");
                    }
                    "criar_bloco" => {
                        if let Some(chave_privada) = &chave_privada_autoridade {
                            println!("Criando bloco...");
                            let novo_bloco = {
                                let mut bc = blockchain.lock().await;
                                bc.criar_e_adicionar_bloco(
                                    chave_privada,
                                    id_autoridade,
                                )
                            };
                            p2p_swarm.difundir_bloco(&novo_bloco);
                            println!("Bloco criado e difundido.");
                        } else {
                            println!("Este nó não é autoridade e não pode criar blocos.");
                        }
                    }
                    _ => println!("Comando desconhecido. Tente 'transacao' ou 'criar_bloco'."),
                }
            }
            else => {
                // Caso nenhuma condição seja satisfeita, continua o loop
            }
        }
    }
}

async fn ler_u32_async(
    stdin_lines: &mut tokio::io::Lines<tokio::io::BufReader<tokio::io::Stdin>>,
) -> u32 {
    loop {
        if let Ok(Some(line)) = stdin_lines.next_line().await {
            if let Ok(value) = line.trim().parse::<u32>() {
                return value;
            } else {
                println!("Valor inválido, por favor digite um número válido:");
            }
        } else {
            println!("Erro ao ler entrada.");
        }
    }
}

async fn ler_i32_async(
    stdin_lines: &mut tokio::io::Lines<tokio::io::BufReader<tokio::io::Stdin>>,
) -> i32 {
    loop {
        if let Ok(Some(line)) = stdin_lines.next_line().await {
            if let Ok(value) = line.trim().parse::<i32>() {
                return value;
            } else {
                println!("Valor inválido, por favor digite um número válido:");
            }
        } else {
            println!("Erro ao ler entrada.");
        }
    }
}

async fn ler_u8_async(
    stdin_lines: &mut tokio::io::Lines<tokio::io::BufReader<tokio::io::Stdin>>,
) -> u8 {
    loop {
        if let Ok(Some(line)) = stdin_lines.next_line().await {
            if let Ok(value) = line.trim().parse::<u8>() {
                return value;
            } else {
                println!("Valor inválido, por favor digite um número válido:");
            }
        } else {
            println!("Erro ao ler entrada.");
        }
    }
}

async fn ler_string_async(
    stdin_lines: &mut tokio::io::Lines<tokio::io::BufReader<tokio::io::Stdin>>,
) -> String {
    loop {
        if let Ok(Some(line)) = stdin_lines.next_line().await {
            return line.trim().to_string();
        } else {
            println!("Erro ao ler entrada.");
        }
    }
}
