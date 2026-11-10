//! Abstracción de extracción de texto para no acoplar la app a una sola librería (spec §6).
use crate::errors::DocFlowError;

pub trait PdfTextExtractor {
    /// Texto de cada página, en orden; el índice 0 es la página 1.
    fn pages(&self, pdf_bytes: &[u8]) -> Result<Vec<String>, DocFlowError>;
}

/// Implementación en Rust puro (`pdf-extract`), sin binarios externos.
pub struct PdfExtractBackend;

impl PdfTextExtractor for PdfExtractBackend {
    fn pages(&self, pdf_bytes: &[u8]) -> Result<Vec<String>, DocFlowError> {
        pdf_extract::extract_text_from_mem_by_pages(pdf_bytes)
            .map_err(|e| DocFlowError::PdfInvalido(e.to_string()))
    }
}
