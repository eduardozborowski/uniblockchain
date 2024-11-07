mod blockchain;
mod rede;
mod criptografia;
mod utils;

use blockchain::{Blockchain, Estudante, PeriodoLetivo, Transacao, Disciplina, Nota};
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

    // Carrega ou inicializa a blockchain
    let blockchain = match Blockchain::carregar_do_disco("blockchain.json") {
        Ok(bc) => Arc::new(Mutex::new(bc)),
        Err(e) => {
            println!("Erro ao carregar a blockchain local: {:?}.", e);
            println!("Inicializando uma blockchain vazia e solicitando atualização da rede...");
            Arc::new(Mutex::new(Blockchain::nova_blockchain()))
        }
    };

    let mut p2p_swarm = iniciar_rede(blockchain.clone()).await;

    // Se a blockchain foi inicializada vazia, solicita atualização da rede
    {
        let bc = blockchain.lock().await;
        if bc.cadeia.len() <= 1 {
            p2p_swarm.solicitar_blockchain();
        }
    }

    let stdin = io::BufReader::new(tokio::io::stdin());
    let mut stdin_lines = stdin.lines();

    println!("Digite o comando (ex: 'transacao', 'criar_bloco' ou 'exibir_blockchain'):");

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
                            } else {
                                // Salva a blockchain após adicionar o bloco
                                if let Err(e) = bc.salvar_em_disco("blockchain.json") {
                                    println!("Erro ao salvar a blockchain: {:?}", e);
                                }
                            }
                        }
                        P2PEvent::NovaTransacao(transacao_recebida) => {
                            println!("Transação recebida: {:?}", transacao_recebida);
                            let mut bc = blockchain.lock().await;
                            bc.adicionar_transacao(transacao_recebida);
                        }
                        P2PEvent::BlockchainSolicitada { peer, channel } => {
                            println!("Nó {} solicitou a blockchain.", peer);
                            let bc = blockchain.lock().await;
                            p2p_swarm.enviar_blockchain(&bc.cadeia, channel);
                        }
                        P2PEvent::BlockchainRecebida(cadeia_recebida) => {
                            println!("Blockchain recebida da rede.");
                            let mut bc = blockchain.lock().await;
                            if bc.cadeia.len() < cadeia_recebida.len() {
                                bc.cadeia = cadeia_recebida;
                                // Salva a blockchain após receber
                                if let Err(e) = bc.salvar_em_disco("blockchain.json") {
                                    println!("Erro ao salvar a blockchain: {:?}", e);
                                }
                            } else {
                                println!("A blockchain local já está atualizada.");
                            }
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

                        let mut estudante = Estudante::novo_estudante(
                            id_estudante,
                            &nome_estudante,
                            ano_nascimento,
                            mes_nascimento,
                            dia_nascimento,
                        );
                        let mut periodo_letivo = PeriodoLetivo::novo_periodo(
                            id_transacao,
                            ano_periodo,
                            semestre,
                        );

                        // Pergunta se o usuário deseja adicionar disciplinas
                        println!("Deseja adicionar disciplinas ao período letivo? (s/n):");
                        let adicionar_disciplinas = ler_string_async(&mut stdin_lines).await;

                        if adicionar_disciplinas.eq_ignore_ascii_case("s") {
                            loop {
                                println!("Digite o ID da disciplina:");
                                let id_disciplina = ler_u32_async(&mut stdin_lines).await;

                                println!("Digite o nome da disciplina:");
                                let nome_disciplina = ler_string_async(&mut stdin_lines).await;

                                println!("Digite o código da disciplina:");
                                let codigo_disciplina = ler_string_async(&mut stdin_lines).await;

                                let mut disciplina = Disciplina::nova_disciplina(
                                    id_disciplina,
                                    &nome_disciplina,
                                    &codigo_disciplina,
                                );

                                // Pergunta se o usuário deseja adicionar notas à disciplina
                                println!("Deseja adicionar notas à disciplina? (s/n):");
                                let adicionar_notas = ler_string_async(&mut stdin_lines).await;

                                if adicionar_notas.eq_ignore_ascii_case("s") {
                                    loop {
                                        println!("Digite o ID da nota:");
                                        let id_nota = ler_u32_async(&mut stdin_lines).await;

                                        println!("Digite o valor da nota:");
                                        let valor_nota = ler_f32_async(&mut stdin_lines).await;

                                        println!("Digite o tipo da nota (ex: Prova, Trabalho):");
                                        let tipo_nota = ler_string_async(&mut stdin_lines).await;

                                        println!("Digite o ano da nota:");
                                        let ano_nota = ler_i32_async(&mut stdin_lines).await;

                                        println!("Digite o mês da nota:");
                                        let mes_nota = ler_u32_async(&mut stdin_lines).await;

                                        println!("Digite o dia da nota:");
                                        let dia_nota = ler_u32_async(&mut stdin_lines).await;

                                        // Cria uma nova nota
                                        let nota = Nota::nova_nota(
                                            id_nota,
                                            valor_nota,
                                            &tipo_nota,
                                            ano_nota,
                                            mes_nota,
                                            dia_nota,
                                        );

                                        // Adiciona a nota à disciplina
                                        disciplina.adicionar_nota(nota);

                                        println!("Deseja adicionar outra nota? (s/n):");
                                        let continuar_notas = ler_string_async(&mut stdin_lines).await;
                                        if continuar_notas.eq_ignore_ascii_case("n") {
                                            break;
                                        }
                                    }
                                }

                                // Adiciona a disciplina ao período letivo
                                periodo_letivo.adicionar_disciplina(disciplina);

                                println!("Deseja adicionar outra disciplina? (s/n):");
                                let continuar_disciplinas = ler_string_async(&mut stdin_lines).await;
                                if continuar_disciplinas.eq_ignore_ascii_case("n") {
                                    break;
                                }
                            }
                        }

                        // Adiciona o período letivo ao estudante
                        estudante.adicionar_periodo_letivo(periodo_letivo.clone());

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
                                let bloco = bc.criar_e_adicionar_bloco(
                                    chave_privada,
                                    id_autoridade,
                                );

                                // Salva a blockchain após criar o bloco
                                if let Err(e) = bc.salvar_em_disco("blockchain.json") {
                                    println!("Erro ao salvar a blockchain: {:?}", e);
                                }
                                bloco
                            };
                            p2p_swarm.difundir_bloco(&novo_bloco);
                            println!("Bloco criado e difundido.");
                        } else {
                            println!("Este nó não é autoridade e não pode criar blocos.");
                        }
                    }
                    "exibir_blockchain" => {
                        let bc = blockchain.lock().await;
                        println!("Blockchain atual:");
                        for bloco in &bc.cadeia {
                            println!("{:#?}", bloco);
                        }
                    }
                    _ => println!("Comando desconhecido. Tente 'transacao', 'criar_bloco' ou 'exibir_blockchain'."),
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

async fn ler_f32_async(
    stdin_lines: &mut tokio::io::Lines<tokio::io::BufReader<tokio::io::Stdin>>,
) -> f32 {
    loop {
        if let Ok(Some(line)) = stdin_lines.next_line().await {
            if let Ok(value) = line.trim().parse::<f32>() {
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
