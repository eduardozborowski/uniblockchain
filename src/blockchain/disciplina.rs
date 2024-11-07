use serde::{Serialize, Deserialize};

use super::Nota;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Disciplina {
    pub id_disciplina: u32,
    pub nome: String,
    pub codigo: String,
    pub notas: Vec<Nota>,
    pub media: f32,
    pub frequencia: f32,
}

impl Disciplina {
    pub fn nova_disciplina(id: u32, nome: &str, codigo: &str) -> Self {
        Disciplina {
            id_disciplina: id,
            nome: nome.to_string(),
            codigo: codigo.to_string(),
            notas: Vec::new(),
            media: 0.0,
            frequencia: 0.0,
        }
    }

    pub fn adicionar_nota(&mut self, nota: Nota) {
        self.notas.push(nota);
    }
}
