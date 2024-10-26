// src\blockchain\periodo_letivo.rs
use serde::{Serialize, Deserialize};

use super::Disciplina;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeriodoLetivo {
    pub id_periodo: u32,
    pub ano: u32,
    pub semestre: u8,
    pub disciplinas: Vec<Disciplina>,
}

impl PeriodoLetivo {
    pub fn novo_periodo(id: u32, ano: u32, semestre: u8) -> Self {
        PeriodoLetivo {
            id_periodo: id,
            ano,
            semestre,
            disciplinas: Vec::new(),
        }
    }
}
