// src/utils/erros.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlocoErro {
    #[error("Autoridade desconhecida")]
    AutoridadeDesconhecida,
    #[error("Assinatura inválida")]
    AssinaturaInvalida,
    #[error("Hash anterior não corresponde")]
    HashAnteriorNaoCorresponde,
}
