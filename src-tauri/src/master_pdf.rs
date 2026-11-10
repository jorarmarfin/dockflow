//! Análisis del PDF maestro: una fila por página, sin escribir nada.
use crate::errors::DocFlowError;
use crate::models::{DocumentRow, DocumentStatus, SourceKind};
use crate::parser::{CodeResult, Parser};
use crate::pdf_text::PdfTextExtractor;
use std::path::Path;

pub fn analyze(path: &Path, extractor: &dyn PdfTextExtractor, parser: &Parser) -> Result<Vec<DocumentRow>, DocFlowError> {
    let bytes = std::fs::read(path)?; // solo lectura
    let pages = extractor.pages(&bytes)?;
    let original_name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    Ok(pages
        .iter()
        .enumerate()
        .map(|(i, text)| analyze_page(path, &original_name, i as u32 + 1, text, parser))
        .collect())
}

pub fn analyze_page(path: &Path, original_name: &str, page: u32, text: &str, parser: &Parser) -> DocumentRow {
    let mut row = DocumentRow {
        id: format!("p{page}"),
        source_path: path.to_string_lossy().into_owned(),
        original_name: original_name.to_string(),
        source_kind: SourceKind::MasterPdf,
        source_page_number: Some(page),
        student_name: parser.extract_name(text),
        extracted_code: None,
        effective_code: None,
        proposed_name: None,
        status: DocumentStatus::Listo,
        observation: None,
        corrected_manually: false,
        destination_path: None,
    };
    if text.trim().is_empty() {
        row.status = DocumentStatus::PdfSinTexto;
        row.observation = Some("Página sin texto extraíble (posible escaneo)".into());
        return row;
    }
    if !parser.looks_like_constancia(text) {
        row.status = DocumentStatus::PaginaNoConstancia;
        row.observation = Some("La página no tiene la estructura de una constancia".into());
        return row;
    }
    match parser.extract_code(text) {
        CodeResult::None => {
            row.status = DocumentStatus::SinCodigo;
            row.observation = Some("No se encontró «código de alumno/a de posgrado»".into());
        }
        CodeResult::Ambiguous(v) => {
            row.status = DocumentStatus::CodigoAmbiguo;
            row.observation = Some(format!("Varios códigos en la página: {}", v.join(", ")));
        }
        CodeResult::One(code) => {
            row.extracted_code = Some(code.clone());
            if parser.is_valid_code(&code) {
                row.proposed_name = Some(format!("{code}.pdf"));
                row.effective_code = Some(code);
            } else {
                row.status = DocumentStatus::FormatoInvalido;
                row.observation = Some(format!("«{code}» no cumple el formato configurado"));
            }
        }
    }
    row
}
