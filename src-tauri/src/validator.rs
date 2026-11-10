//! Reglas sobre el lote completo: duplicados y conflictos de destino.
use crate::models::{DocumentRow, DocumentStatus};
use std::collections::HashMap;
use std::path::Path;

/// Revalida todo el lote. Idempotente: restablece primero los estados derivados.
pub fn validate_batch(rows: &mut [DocumentRow], destination: Option<&Path>) {
    for r in rows.iter_mut() {
        if matches!(r.status, DocumentStatus::DuplicadoLote | DocumentStatus::ConflictoDestino) {
            r.status = DocumentStatus::Listo;
            r.observation = None;
        }
    }
    let mut count: HashMap<String, usize> = HashMap::new();
    // clave en minúsculas: sistemas de archivos que no distinguen mayúsculas
    for r in rows.iter().filter(|r| r.status == DocumentStatus::Listo) {
        if let Some(c) = &r.effective_code {
            *count.entry(c.to_lowercase()).or_default() += 1;
        }
    }
    for r in rows.iter_mut().filter(|r| r.status == DocumentStatus::Listo) {
        let Some(code) = r.effective_code.clone() else { continue };
        if count.get(&code.to_lowercase()).copied().unwrap_or(0) > 1 {
            r.status = DocumentStatus::DuplicadoLote;
            r.observation = Some(format!("El código {code} aparece en más de una fila"));
        } else if let Some(dest) = destination {
            if dest.join(format!("{code}.pdf")).exists() {
                r.status = DocumentStatus::ConflictoDestino;
                r.observation = Some(format!("{code}.pdf ya existe en el destino"));
            }
        }
    }
}

/// Corrección manual: solo cambia el código efectivo (nombre de salida), no el PDF.
pub fn apply_correction(row: &mut DocumentRow, code: &str, parser: &crate::parser::Parser) -> Result<(), String> {
    let code = parser.normalize_code(code);
    if !parser.is_valid_code(&code) {
        return Err(format!("«{code}» no cumple el formato configurado"));
    }
    row.effective_code = Some(code.clone());
    row.proposed_name = Some(format!("{code}.pdf"));
    row.corrected_manually = true;
    row.status = DocumentStatus::Listo;
    row.observation = Some("Código corregido manualmente".into());
    Ok(())
}
