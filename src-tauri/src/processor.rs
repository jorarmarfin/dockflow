//! Generación segura: temporal privado → verificación → publicación sin reemplazo.
use crate::errors::DocFlowError;
use crate::models::{DocumentRow, DocumentStatus};
use crate::pdf_split::extract_page;
use crate::pdf_text::PdfTextExtractor;
use lopdf::Document;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

/// Verifica la salida: PDF legible, 1 página y, si el código no fue corregido a mano, que el texto lo contenga.
pub fn verify_output(bytes: &[u8], row: &DocumentRow, extractor: &dyn PdfTextExtractor) -> Result<(), String> {
    let doc = Document::load_mem(bytes).map_err(|e| format!("salida ilegible: {e}"))?;
    let n = doc.get_pages().len();
    if n != 1 {
        return Err(format!("la salida tiene {n} páginas, se esperaba 1"));
    }
    if !row.corrected_manually {
        let code = row.effective_code.as_deref().unwrap_or_default();
        let pages = extractor.pages(bytes).map_err(|e| e.to_string())?;
        let text = crate::parser::normalize_ws(pages.first().map(String::as_str).unwrap_or(""));
        if !text.to_uppercase().contains(code) {
            return Err(format!("el código {code} no aparece en el PDF generado"));
        }
    }
    Ok(())
}

/// Publica sin reemplazar nunca un archivo existente.
fn publish(tmp: &Path, dest: &Path) -> std::io::Result<()> {
    match fs::hard_link(tmp, dest) {
        Ok(()) => fs::remove_file(tmp),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Err(e),
        Err(_) => {
            // sistemas sin enlaces duros (FAT/exFAT): create_new también impide sobrescribir
            let mut out = OpenOptions::new().write(true).create_new(true).open(dest)?;
            out.write_all(&fs::read(tmp)?)?;
            out.sync_all()?;
            fs::remove_file(tmp)
        }
    }
}

/// Procesa filas LISTO del maestro. `on_progress(hechas, total)`; `cancel` detiene sin dejar parciales.
pub fn process_master(
    master_path: &Path,
    rows: &mut [DocumentRow],
    destination: &Path,
    extractor: &dyn PdfTextExtractor,
    cancel: &AtomicBool,
    mut on_progress: impl FnMut(usize, usize),
) -> Result<(), DocFlowError> {
    fs::create_dir_all(destination)?;
    let master = Document::load_mem(&fs::read(master_path)?).map_err(|e| DocFlowError::PdfInvalido(e.to_string()))?;
    let total = rows.iter().filter(|r| r.status == DocumentStatus::Listo).count();
    let mut done = 0;
    for row in rows.iter_mut() {
        if row.status != DocumentStatus::Listo {
            continue;
        }
        if cancel.load(Ordering::Relaxed) {
            row.status = DocumentStatus::Cancelado;
            continue;
        }
        let (Some(page), Some(code)) = (row.source_page_number, row.effective_code.clone()) else { continue };
        let final_path = destination.join(format!("{code}.pdf"));
        let result = (|| -> Result<(), String> {
            let bytes = extract_page(&master, page).map_err(|e| e.to_string())?;
            verify_output(&bytes, row, extractor)?;
            let tmp = destination.join(format!(".docflow-{code}.part"));
            fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
            publish(&tmp, &final_path).map_err(|e| {
                let _ = fs::remove_file(&tmp);
                if e.kind() == std::io::ErrorKind::AlreadyExists { "el destino ya existe".to_string() } else { e.to_string() }
            })
        })();
        match result {
            Ok(()) => {
                row.status = DocumentStatus::Procesado;
                row.destination_path = Some(final_path.to_string_lossy().into_owned());
            }
            Err(msg) => {
                row.status = DocumentStatus::ErrorCopia;
                row.observation = Some(msg);
            }
        }
        done += 1;
        on_progress(done, total);
    }
    Ok(())
}
