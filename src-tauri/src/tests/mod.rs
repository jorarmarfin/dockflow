//! Pruebas de integración del motor. Fixture sintético en memoria (sin datos reales).
//! Prueba con el maestro real (opt-in): DOCFLOW_REAL_MASTER=/ruta/maestro.pdf cargo test real -- --nocapture
use crate::master_pdf::analyze;
use crate::models::DocumentStatus;
use crate::parser::Parser;
use crate::pdf_text::PdfExtractBackend;
use crate::processor::process_master;
use crate::validator::{apply_correction, validate_batch};
use lopdf::{dictionary, Document, Object, Stream};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

/// Construye un PDF sintético con una página de texto por entrada.
pub fn synthetic_pdf(pages: &[&str]) -> Vec<u8> {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font = doc.add_object(dictionary! { "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica" });
    let res = doc.add_object(dictionary! { "Font" => dictionary! { "F1" => font } });
    let mut kids = vec![];
    for text in pages {
        let mut ops = String::from("BT /F1 12 Tf 50 700 Td 14 TL\n");
        for line in text.lines() {
            ops += &format!("({}) Tj T*\n", line.replace('(', "\\(").replace(')', "\\)"));
        }
        ops += "ET";
        let content = doc.add_object(Stream::new(dictionary! {}, ops.into_bytes()));
        let page = doc.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages_id, "Contents" => content,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()], "Resources" => res,
        });
        kids.push(Object::Reference(page));
    }
    let n = kids.len() as i64;
    doc.objects.insert(pages_id, Object::Dictionary(dictionary! { "Type" => "Pages", "Kids" => kids, "Count" => n }));
    let cat = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", cat);
    let mut out = Vec::new();
    doc.save_to(&mut out).unwrap();
    out
}

fn constancia(name: &str, code: &str) -> String {
    format!("CONSTANCIA DE ADMISION 2026 - 2\nHACE CONSTAR\nQue, Don {name} ha sido\nadmitido en el Periodo 2026-2, con codigo de alumno de posgrado:\n{code}, para seguir estudios.")
}

fn write_master(dir: &Path, pages: &[String]) -> PathBuf {
    let refs: Vec<&str> = pages.iter().map(String::as_str).collect();
    let p = dir.join("maestro.pdf");
    std::fs::write(&p, synthetic_pdf(&refs)).unwrap();
    p
}

fn sha(p: &Path) -> String { Sha256::digest(std::fs::read(p).unwrap()).iter().map(|b| format!("{b:02x}")).collect() }

#[test]
fn analiza_y_divide_sintetico_con_incidencias() {
    let d = tempfile::tempdir().unwrap();
    let pages = vec![
        constancia("ANA UNO", "20260001A"),
        "Pagina sin nada relevante".to_string(),               // no constancia
        constancia("LUIS DOS", "20260002B"),
        constancia("DUP UNO", "20260001A"),                    // duplicado de la 1
        "CONSTANCIA DE ADMISION\nHACE CONSTAR\nsin codigo".to_string(), // sin código
    ];
    let master = write_master(d.path(), &pages);
    let before = sha(&master);
    let parser = Parser::default();
    let mut rows = analyze(&master, &PdfExtractBackend, &parser).unwrap();
    assert_eq!(rows.len(), 5);
    assert_eq!(rows[0].source_page_number, Some(1));
    assert_eq!(rows[1].status, DocumentStatus::PaginaNoConstancia);
    assert_eq!(rows[4].status, DocumentStatus::SinCodigo);
    let out = d.path().join("out");
    validate_batch(&mut rows, Some(&out));
    assert_eq!(rows[0].status, DocumentStatus::DuplicadoLote);
    assert_eq!(rows[3].status, DocumentStatus::DuplicadoLote);

    // corrección manual resuelve el duplicado (revalidación)
    apply_correction(&mut rows[3], "20260003c", &parser).unwrap();
    validate_batch(&mut rows, Some(&out));
    assert_eq!(rows[0].status, DocumentStatus::Listo);
    assert_eq!(rows[3].effective_code.as_deref(), Some("20260003C"));

    process_master(&master, &mut rows, &out, &PdfExtractBackend, &AtomicBool::new(false), |_, _| {}).unwrap();
    assert_eq!(rows.iter().filter(|r| r.status == DocumentStatus::Procesado).count(), 3);
    for f in ["20260001A.pdf", "20260002B.pdf", "20260003C.pdf"] {
        let doc = Document::load(out.join(f)).unwrap();
        assert_eq!(doc.get_pages().len(), 1, "{f}");
    }
    // el nombre sigue al contenido de la página, no a la posición
    let t = PdfExtractBackend.pages_of(&out.join("20260002B.pdf"));
    assert!(t.contains("LUIS DOS"));
    assert_eq!(sha(&master), before, "el maestro no debe cambiar");
    assert!(std::fs::read_dir(&out).unwrap().all(|e| !e.unwrap().file_name().to_string_lossy().ends_with(".part")));
}

#[test]
fn no_sobrescribe_destino_existente() {
    let d = tempfile::tempdir().unwrap();
    let master = write_master(d.path(), &[constancia("ANA UNO", "20260001A")]);
    let out = d.path().join("out");
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(out.join("20260001A.pdf"), b"previo").unwrap();
    let mut rows = analyze(&master, &PdfExtractBackend, &Parser::default()).unwrap();
    validate_batch(&mut rows, Some(&out));
    assert_eq!(rows[0].status, DocumentStatus::ConflictoDestino);
    process_master(&master, &mut rows, &out, &PdfExtractBackend, &AtomicBool::new(false), |_, _| {}).unwrap();
    assert_eq!(std::fs::read(out.join("20260001A.pdf")).unwrap(), b"previo");
}

#[test]
fn cancelar_no_deja_parciales() {
    let d = tempfile::tempdir().unwrap();
    let master = write_master(d.path(), &[constancia("A B", "20260001A"), constancia("C D", "20260002B")]);
    let out = d.path().join("out");
    let mut rows = analyze(&master, &PdfExtractBackend, &Parser::default()).unwrap();
    process_master(&master, &mut rows, &out, &PdfExtractBackend, &AtomicBool::new(true), |_, _| {}).unwrap();
    assert!(rows.iter().all(|r| r.status == DocumentStatus::Cancelado));
    assert_eq!(std::fs::read_dir(&out).unwrap().count(), 0);
}

#[test]
fn pdf_invalido_devuelve_error() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("x.pdf");
    std::fs::write(&p, b"no soy pdf").unwrap();
    assert!(analyze(&p, &PdfExtractBackend, &Parser::default()).is_err());
}

#[test]
fn real_maestro_178() {
    let Ok(path) = std::env::var("DOCFLOW_REAL_MASTER") else { return };
    let master = PathBuf::from(path);
    let before = sha(&master);
    let parser = Parser::default();
    let mut rows = analyze(&master, &PdfExtractBackend, &parser).unwrap();
    assert_eq!(rows.len(), 178);
    // DOCFLOW_KEEP_OUT=/dir conserva las salidas para inspección visual local
    let keep = std::env::var_os("DOCFLOW_KEEP_OUT").map(PathBuf::from);
    let tmp = tempfile::tempdir().unwrap();
    let out_dir = keep.unwrap_or_else(|| tmp.path().to_path_buf());
    std::fs::create_dir_all(&out_dir).unwrap();
    struct O(PathBuf); impl O { fn path(&self) -> &Path { &self.0 } }
    let out = O(out_dir);
    assert_eq!(std::fs::read_dir(out.path()).unwrap().count(), 0);
    validate_batch(&mut rows, Some(out.path()));
    assert!(rows.iter().all(|r| r.status == DocumentStatus::Listo), "todas LISTO");
    assert!(rows.iter().all(|r| r.student_name.is_some()), "178 nombres");
    assert_eq!(rows[0].proposed_name.as_deref(), Some("20267722F.pdf"));
    assert_eq!(rows[1].proposed_name.as_deref(), Some("20267731E.pdf"));
    assert_eq!(rows[177].proposed_name.as_deref(), Some("20267672I.pdf"));
    let t = std::time::Instant::now();
    process_master(&master, &mut rows, out.path(), &PdfExtractBackend, &AtomicBool::new(false), |_, _| {}).unwrap();
    println!("generación: {:?}", t.elapsed());
    assert_eq!(rows.iter().filter(|r| r.status == DocumentStatus::Procesado).count(), 178);
    assert_eq!(std::fs::read_dir(out.path()).unwrap().count(), 178);
    for r in &rows {
        let p = out.path().join(r.proposed_name.as_ref().unwrap());
        assert_eq!(Document::load(&p).unwrap().get_pages().len(), 1);
    }
    assert_eq!(sha(&master), before);
}

trait PagesOf { fn pages_of(&self, p: &Path) -> String; }
impl PagesOf for PdfExtractBackend {
    fn pages_of(&self, p: &Path) -> String {
        use crate::pdf_text::PdfTextExtractor;
        self.pages(&std::fs::read(p).unwrap()).unwrap().join(" ")
    }
}
