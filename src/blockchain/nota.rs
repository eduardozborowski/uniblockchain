
use serde::{Serialize, Deserialize};
use chrono::NaiveDate;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Nota {
    pub id_nota: u32,
    pub valor: f32,
    pub tipo: String,
    pub data: NaiveDate,
}

impl Nota {
    pub fn nova_nota(id: u32, valor: f32, tipo: &str, ano: i32, mes: u32, dia: u32) -> Self {
        Nota {
            id_nota: id,
            valor,
            tipo: tipo.to_string(),
            data: NaiveDate::from_ymd_opt(ano, mes, dia).expect("Data inválida"),
        }
    }
}
