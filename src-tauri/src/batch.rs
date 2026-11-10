//! Estado del lote en el backend (spec §6): la UI solo recibe vistas, nunca decide rutas.
use crate::errors::DocFlowError;
use crate::master_pdf;
use crate::models::{summarize, BatchView, DocumentRow, DocumentStatus};
use crate::parser::Parser;
use crate::pdf_split::extract_page;
use crate::pdf_text::PdfExtractBackend;
use crate::processor::process_master;
use crate::validator::{apply_correction, validate_batch};
use lopdf::Document;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

pub struct Batch {
    pub master: PathBuf,
    pub destination: Option<PathBuf>,
    pub rows: Vec<DocumentRow>,
    parser: Parser,
}

fn other(msg: impl Into<String>) -> DocFlowError { DocFlowError::Otro(msg.into()) }

impl Batch {
    pub fn analyze(source: &Path) -> Result<Self, DocFlowError> {
        let is_pdf = source.extension().is_some_and(|e| e.eq_ignore_ascii_case("pdf"));
        if !source.is_file() || !is_pdf {
            return Err(other("Selecciona un archivo PDF existente"));
        }
        let parser = Parser::default();
        let mut rows = master_pdf::analyze(source, &PdfExtractBackend, &parser)?;
        validate_batch(&mut rows, None);
        Ok(Self { master: source.to_path_buf(), destination: None, rows, parser })
    }

    pub fn set_destination(&mut self, dest: &Path) -> Result<(), DocFlowError> {
        if dest.is_file() {
            return Err(other("El destino debe ser una carpeta"));
        }
        if Some(dest) == self.master.parent() {
            return Err(other("La carpeta destino debe ser distinta de la del PDF maestro"));
        }
        self.destination = Some(dest.to_path_buf());
        validate_batch(&mut self.rows, Some(dest));
        Ok(())
    }

    pub fn correct(&mut self, row_id: &str, code: &str) -> Result<(), DocFlowError> {
        let row = self.rows.iter_mut().find(|r| r.id == row_id).ok_or_else(|| other("Fila no encontrada"))?;
        apply_correction(row, code, &self.parser).map_err(DocFlowError::Otro)?;
        validate_batch(&mut self.rows, self.destination.as_deref());
        Ok(())
    }

    /// Genera los PDFs de las filas pendientes. Reintenta las canceladas/fallidas; las procesadas se saltan.
    pub fn process(&mut self, cancel: &AtomicBool, on_progress: impl FnMut(usize, usize)) -> Result<(), DocFlowError> {
        let dest = self.destination.clone().ok_or_else(|| other("Primero elige la carpeta destino"))?;
        for r in self.rows.iter_mut().filter(|r| matches!(r.status, DocumentStatus::Cancelado | DocumentStatus::ErrorCopia)) {
            r.status = DocumentStatus::Listo;
            r.observation = None;
        }
        validate_batch(&mut self.rows, Some(&dest));
        process_master(&self.master, &mut self.rows, &dest, &PdfExtractBackend, cancel, on_progress)
    }

    /// PDF de una sola página (para vista previa local). No escribe nada.
    pub fn page_bytes(&self, row_id: &str) -> Result<Vec<u8>, DocFlowError> {
        let page = self.rows.iter().find(|r| r.id == row_id).and_then(|r| r.source_page_number)
            .ok_or_else(|| other("Fila no encontrada"))?;
        let master = Document::load(&self.master).map_err(|e| DocFlowError::PdfInvalido(e.to_string()))?;
        extract_page(&master, page)
    }

    pub fn view(&self) -> BatchView {
        BatchView {
            master_path: self.master.to_string_lossy().into_owned(),
            destination: self.destination.as_ref().map(|d| d.to_string_lossy().into_owned()),
            rows: self.rows.clone(),
            summary: summarize(&self.rows),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::synthetic_pdf;

    fn c(code: &str) -> String {
        format!("CONSTANCIA DE ADMISION\nHACE CONSTAR\nQue, Don X Y ha sido\nadmitido, con codigo de alumno de posgrado:\n{code}, fin")
    }

    fn batch(dir: &Path, codes: &[&str]) -> Batch {
        let pages: Vec<String> = codes.iter().map(|k| c(k)).collect();
        let refs: Vec<&str> = pages.iter().map(String::as_str).collect();
        let p = dir.join("m.pdf");
        std::fs::write(&p, synthetic_pdf(&refs)).unwrap();
        Batch::analyze(&p).unwrap()
    }

    #[test]
    fn rechaza_no_pdf() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("a.txt");
        std::fs::write(&p, "x").unwrap();
        assert!(Batch::analyze(&p).is_err());
        assert!(Batch::analyze(&d.path().join("no.pdf")).is_err());
    }

    #[test]
    fn resumen_y_duplicados() {
        let d = tempfile::tempdir().unwrap();
        let b = batch(d.path(), &["20260001A", "20260002B", "20260001A"]);
        let v = b.view();
        assert_eq!((v.summary.total, v.summary.duplicates, v.summary.pending), (3, 2, 1));
    }

    #[test]
    fn destino_igual_al_origen_se_rechaza() {
        let d = tempfile::tempdir().unwrap();
        let mut b = batch(d.path(), &["20260001A"]);
        assert!(b.set_destination(d.path()).is_err());
        assert!(b.set_destination(&d.path().join("salida")).is_ok());
    }

    #[test]
    fn correccion_revalida_y_actualiza_resumen() {
        let d = tempfile::tempdir().unwrap();
        let mut b = batch(d.path(), &["20260001A", "20260001A"]);
        b.correct("p2", "20260009z").unwrap();
        let v = b.view();
        assert_eq!(v.summary.duplicates, 0);
        assert_eq!(v.rows[1].effective_code.as_deref(), Some("20260009Z"));
        assert!(v.rows[1].corrected_manually);
        assert!(b.correct("p2", "mal").is_err());
        assert!(b.correct("zzz", "20260009Z").is_err());
        // corregir a un código ya usado crea duplicado y bloquea
        b.correct("p2", "20260001A").unwrap();
        assert_eq!(b.view().rows[0].status, DocumentStatus::DuplicadoLote);
    }

    #[test]
    fn genera_cancela_y_reintenta() {
        let d = tempfile::tempdir().unwrap();
        let mut b = batch(d.path(), &["20260001A", "20260002B", "20260003C"]);
        assert!(b.process(&AtomicBool::new(false), |_, _| {}).is_err(), "sin destino");
        let out = d.path().join("salida");
        b.set_destination(&out).unwrap();
        let cancel = AtomicBool::new(true);
        b.process(&cancel, |_, _| {}).unwrap();
        assert_eq!(b.view().summary.pending, 3);
        assert_eq!(std::fs::read_dir(&out).unwrap().count(), 0);
        let mut seen = vec![];
        b.process(&AtomicBool::new(false), |h, t| seen.push((h, t))).unwrap();
        assert_eq!(seen.last(), Some(&(3, 3)));
        assert_eq!(b.view().summary.processed, 3);
        assert_eq!(std::fs::read_dir(&out).unwrap().count(), 3);
        // segunda corrida: nada pendiente, no toca lo ya generado
        b.process(&AtomicBool::new(false), |_, _| {}).unwrap();
        assert_eq!(b.view().summary.processed, 3);
    }

    #[test]
    fn pagina_previa_es_un_pdf_de_una_pagina() {
        let d = tempfile::tempdir().unwrap();
        let b = batch(d.path(), &["20260001A", "20260002B"]);
        let bytes = b.page_bytes("p2").unwrap();
        assert_eq!(Document::load_mem(&bytes).unwrap().get_pages().len(), 1);
    }
}
