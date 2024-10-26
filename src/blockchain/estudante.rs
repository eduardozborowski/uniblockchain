// src/blockchain/estudante.rs

use serde::{Serialize, Deserialize};
use chrono::NaiveDate;

use super::PeriodoLetivo;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Estudante {
    pub id_estudante: u32,
    pub nome: String,
    pub data_nascimento: NaiveDate,
    pub periodos_letivos: Vec<PeriodoLetivo>,
}

impl Estudante {
    pub fn novo_estudante(id: u32, nome: &str, ano: i32, mes: u32, dia: u32) -> Self {
        Estudante {
            id_estudante: id,
            nome: nome.to_string(),
            data_nascimento: NaiveDate::from_ymd_opt(ano, mes, dia).expect("Data inválida"),
            periodos_letivos: Vec::new(),
        }
    }
}
