#![allow(dead_code)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocumentStatus {
    Listo,
    Procesado,
    SinCodigo,
    FormatoInvalido,
    CodigoAmbiguo,
    DuplicadoLote,
    ConflictoDestino,
    PdfSinTexto,
    PdfInvalido,
    PaginaNoConstancia,
    MultipaginaAmbigua,
    ErrorCopia,
    Cancelado,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SourceKind {
    MasterPdf,
    PdfFile,
    ZipEntry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentRow {
    pub id: String,
    pub source_path: String,
    pub original_name: String,
    pub source_kind: SourceKind,
    /// Base 1; obligatorio en MASTER_PDF.
    pub source_page_number: Option<u32>,
    pub student_name: Option<String>,
    pub extracted_code: Option<String>,
    pub effective_code: Option<String>,
    pub proposed_name: Option<String>,
    pub status: DocumentStatus,
    pub observation: Option<String>,
    pub corrected_manually: bool,
    pub destination_path: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchSummary {
    pub total: u32,
    pub processed: u32,
    pub pending: u32,
    pub errors: u32,
    pub duplicates: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchView {
    pub master_path: String,
    pub destination: Option<String>,
    pub rows: Vec<DocumentRow>,
    pub summary: BatchSummary,
}

pub fn summarize(rows: &[DocumentRow]) -> BatchSummary {
    use DocumentStatus::*;
    let mut s = BatchSummary { total: rows.len() as u32, ..Default::default() };
    for r in rows {
        match r.status {
            Procesado => s.processed += 1,
            Listo | Cancelado => s.pending += 1,
            DuplicadoLote => s.duplicates += 1,
            _ => s.errors += 1,
        }
    }
    s
}
