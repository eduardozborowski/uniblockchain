// src\blockchain\transacao.rs
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use super::{Estudante, PeriodoLetivo};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transacao {
    pub id_transacao: u32,
    pub estudante: Estudante,
    pub periodo_letivo: PeriodoLetivo,
    pub timestamp: DateTime<Utc>,
}

impl Transacao {
    pub fn nova_transacao(
        id_transacao: u32,
        estudante: Estudante,
        periodo_letivo: PeriodoLetivo,
    ) -> Self {
        let timestamp = Utc::now();

        Transacao {
            id_transacao,
            estudante,
            periodo_letivo,
            timestamp,
        }
    }
}
