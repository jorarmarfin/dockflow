use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum DocFlowError {
    #[error("PDF inválido: {0}")]
    PdfInvalido(String),
    #[error("Error de E/S: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Otro(String),
}

// Los errores cruzan al frontend como texto, sin datos del documento.
impl Serialize for DocFlowError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
