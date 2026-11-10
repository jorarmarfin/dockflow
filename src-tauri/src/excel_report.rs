//! Reporte XLSX de trazabilidad: Detalle, Resumen e Incidencias (spec §8).
//! Los datos que vienen del PDF se escriben siempre como TEXTO y se neutralizan si Excel
//! podría interpretarlos como fórmula. No se incluye el contenido de los PDF.
use crate::errors::DocFlowError;
use crate::models::{DocumentRow, DocumentStatus, SourceKind};
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, Worksheet, XlsxError};
use std::path::Path;

impl From<XlsxError> for DocFlowError {
    fn from(e: XlsxError) -> Self { DocFlowError::Otro(format!("Excel: {e}")) }
}

/// Neutraliza inyección de fórmulas (=, +, -, @, tab, CR al inicio).
pub fn safe_text(s: &str) -> String {
    match s.chars().next() {
        Some('=' | '+' | '-' | '@' | '\t' | '\r') => format!("'{s}"),
        _ => s.to_string(),
    }
}

fn kind_label(k: SourceKind) -> &'static str {
    match k { SourceKind::MasterPdf => "MASTER_PDF", SourceKind::PdfFile => "PDF_FILE", SourceKind::ZipEntry => "ZIP_ENTRY" }
}

pub fn status_label(s: DocumentStatus) -> &'static str {
    use DocumentStatus::*;
    match s {
        Listo => "LISTO", Procesado => "PROCESADO", SinCodigo => "SIN_CODIGO", FormatoInvalido => "FORMATO_INVALIDO",
        CodigoAmbiguo => "CODIGO_AMBIGUO", DuplicadoLote => "DUPLICADO_LOTE", ConflictoDestino => "CONFLICTO_DESTINO",
        PdfSinTexto => "PDF_SIN_TEXTO", PdfInvalido => "PDF_INVALIDO", PaginaNoConstancia => "PAGINA_NO_CONSTANCIA",
        MultipaginaAmbigua => "MULTIPAGINA_AMBIGUA", ErrorCopia => "ERROR_COPIA", Cancelado => "CANCELADO",
    }
}

fn recommended_action(s: DocumentStatus) -> &'static str {
    use DocumentStatus::*;
    match s {
        SinCodigo | CodigoAmbiguo | FormatoInvalido => "Revisar PDF y corregir código",
        DuplicadoLote => "Corregir código (duplicado en el lote)",
        ConflictoDestino => "Cambiar carpeta destino o retirar el archivo existente",
        PdfSinTexto | PdfInvalido => "Revisar PDF (sin texto o ilegible)",
        PaginaNoConstancia | MultipaginaAmbigua => "Revisar página manualmente",
        ErrorCopia => "Revisar permisos/espacio y reintentar",
        Cancelado | Listo => "Volver a generar",
        Procesado => "",
    }
}

fn tint(s: DocumentStatus) -> Color {
    use DocumentStatus::*;
    match s {
        Procesado | Listo => Color::RGB(0xD5F2F1),
        SinCodigo | FormatoInvalido | CodigoAmbiguo | DuplicadoLote | ConflictoDestino | PaginaNoConstancia
        | MultipaginaAmbigua | Cancelado => Color::RGB(0xFDEBC8),
        _ => Color::RGB(0xFAD4D4),
    }
}

fn opt(s: &Option<String>) -> String { safe_text(s.as_deref().unwrap_or("")) }

pub struct ReportMeta<'a> {
    pub master_name: &'a str,
    pub destination: &'a str,
    pub generated_at: &'a str, // "YYYY-MM-DD HH:MM:SS" local
}

pub fn write_report(path: &Path, rows: &[DocumentRow], meta: &ReportMeta) -> Result<(), DocFlowError> {
    let mut wb = Workbook::new();
    let head = Format::new().set_bold().set_font_color(Color::White).set_background_color(Color::RGB(0x0F172A))
        .set_border(FormatBorder::Thin).set_align(FormatAlign::VerticalCenter);
    let text = Format::new();

    // ---- Detalle
    let ws = wb.add_worksheet();
    ws.set_name("Detalle")?;
    let cols = ["N°", "Archivo original", "Página del maestro", "Tipo de origen", "Ruta relativa de origen", "Alumno",
        "Código extraído", "Código final", "Archivo destino", "Estado", "Observación", "Corrección manual", "Fecha y hora"];
    let widths = [6.0, 30.0, 12.0, 14.0, 22.0, 38.0, 16.0, 16.0, 20.0, 22.0, 46.0, 12.0, 20.0];
    header(ws, &cols, &widths, &head)?;
    for (i, r) in rows.iter().enumerate() {
        let row = i as u32 + 1;
        let st = Format::new().set_background_color(tint(r.status));
        ws.write_number(row, 0, row as f64)?;
        ws.write_string(row, 1, safe_text(&r.original_name))?;
        match r.source_page_number { Some(p) => ws.write_number(row, 2, p as f64)?, None => ws.write_blank(row, 2, &text)? };
        ws.write_string(row, 3, kind_label(r.source_kind))?;
        ws.write_string(row, 4, "")?;
        ws.write_string(row, 5, opt(&r.student_name))?;
        ws.write_string(row, 6, opt(&r.extracted_code))?;
        ws.write_string(row, 7, opt(&r.effective_code))?;
        let dest_name = if r.status == DocumentStatus::Procesado { opt(&r.proposed_name) } else { String::new() };
        ws.write_string(row, 8, dest_name)?;
        ws.write_string_with_format(row, 9, status_label(r.status), &st)?;
        ws.write_string(row, 10, opt(&r.observation))?;
        ws.write_string(row, 11, if r.corrected_manually { "Sí" } else { "No" })?;
        ws.write_string(row, 12, meta.generated_at)?;
    }
    ws.autofilter(0, 0, rows.len() as u32, cols.len() as u16 - 1)?;
    ws.set_freeze_panes(1, 0)?;

    // ---- Resumen
    let count = |f: &dyn Fn(DocumentStatus) -> bool| rows.iter().filter(|r| f(r.status)).count() as f64;
    let ws = wb.add_worksheet();
    ws.set_name("Resumen")?;
    ws.set_column_width(0, 34)?;
    ws.set_column_width(1, 40)?;
    let bold = Format::new().set_bold();
    let lines: Vec<(&str, Result<String, f64>)> = vec![
        ("Modo de entrada", Ok("PDF maestro multipágina".into())),
        ("Archivo de origen", Ok(safe_text(meta.master_name))),
        ("Total de páginas de origen", Err(rows.len() as f64)),
        ("Fecha y hora", Ok(meta.generated_at.to_string())),
        ("Carpeta destino", Ok(safe_text(meta.destination))),
        ("Total detectados (con código)", Err(rows.iter().filter(|r| r.extracted_code.is_some()).count() as f64)),
        ("Procesados", Err(count(&|s| s == DocumentStatus::Procesado))),
        ("Pendientes / listos", Err(count(&|s| matches!(s, DocumentStatus::Listo | DocumentStatus::Cancelado)))),
        ("Sin código", Err(count(&|s| s == DocumentStatus::SinCodigo))),
        ("Formato inválido / ambiguo", Err(count(&|s| matches!(s, DocumentStatus::FormatoInvalido | DocumentStatus::CodigoAmbiguo)))),
        ("Duplicados", Err(count(&|s| s == DocumentStatus::DuplicadoLote))),
        ("Conflictos de destino", Err(count(&|s| s == DocumentStatus::ConflictoDestino))),
        ("PDFs ilegibles / sin texto", Err(count(&|s| matches!(s, DocumentStatus::PdfInvalido | DocumentStatus::PdfSinTexto)))),
        ("Páginas que no son constancia", Err(count(&|s| matches!(s, DocumentStatus::PaginaNoConstancia | DocumentStatus::MultipaginaAmbigua)))),
        ("Errores de copia", Err(count(&|s| s == DocumentStatus::ErrorCopia))),
    ];
    for (i, (k, v)) in lines.iter().enumerate() {
        ws.write_string_with_format(i as u32, 0, *k, &bold)?;
        match v { Ok(s) => ws.write_string(i as u32, 1, s)?, Err(n) => ws.write_number(i as u32, 1, *n)? };
    }

    // ---- Incidencias
    let ws = wb.add_worksheet();
    ws.set_name("Incidencias")?;
    let cols = ["Página", "Alumno", "Código extraído", "Código final", "Estado", "Observación", "Acción recomendada"];
    header(ws, &cols, &[8.0, 38.0, 16.0, 16.0, 24.0, 46.0, 44.0], &head)?;
    let mut n = 0u32;
    for r in rows.iter().filter(|r| r.status != DocumentStatus::Procesado) {
        n += 1;
        let st = Format::new().set_background_color(tint(r.status));
        match r.source_page_number { Some(p) => ws.write_number(n, 0, p as f64)?, None => ws.write_blank(n, 0, &text)? };
        ws.write_string(n, 1, opt(&r.student_name))?;
        ws.write_string(n, 2, opt(&r.extracted_code))?;
        ws.write_string(n, 3, opt(&r.effective_code))?;
        ws.write_string_with_format(n, 4, status_label(r.status), &st)?;
        ws.write_string(n, 5, opt(&r.observation))?;
        ws.write_string(n, 6, recommended_action(r.status))?;
    }
    if n > 0 { ws.autofilter(0, 0, n, cols.len() as u16 - 1)?; }
    ws.set_freeze_panes(1, 0)?;

    wb.save(path)?;
    Ok(())
}

fn header(ws: &mut Worksheet, cols: &[&str], widths: &[f64], f: &Format) -> Result<(), XlsxError> {
    for (c, (name, w)) in cols.iter().zip(widths).enumerate() {
        ws.write_string_with_format(0, c as u16, *name, f)?;
        ws.set_column_width(c as u16, *w)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use calamine::{open_workbook, Data, Reader, Xlsx};

    fn row(page: u32, code: Option<&str>, status: DocumentStatus, name: &str) -> DocumentRow {
        DocumentRow {
            id: format!("p{page}"), source_path: "/m.pdf".into(), original_name: "m.pdf".into(),
            source_kind: SourceKind::MasterPdf, source_page_number: Some(page), student_name: Some(name.into()),
            extracted_code: code.map(Into::into), effective_code: code.map(Into::into),
            proposed_name: code.map(|c| format!("{c}.pdf")), status, observation: None,
            corrected_manually: false, destination_path: None,
        }
    }

    fn cell(v: &Data) -> String { v.to_string() }

    #[test]
    fn neutraliza_formulas() {
        assert_eq!(safe_text("=1+1"), "'=1+1");
        assert_eq!(safe_text("@SUM(A1)"), "'@SUM(A1)");
        assert_eq!(safe_text("-2"), "'-2");
        assert_eq!(safe_text("ANA"), "ANA");
        assert_eq!(safe_text(""), "");
    }

    #[test]
    fn genera_tres_hojas_con_una_fila_por_pagina() {
        let d = tempfile::tempdir().unwrap();
        let rows = vec![
            row(1, Some("20260001A"), DocumentStatus::Procesado, "ANA UNO"),
            row(2, None, DocumentStatus::SinCodigo, "=HYPERLINK(\"http://x\")"),
            row(3, Some("20260002B"), DocumentStatus::DuplicadoLote, "LUIS"),
        ];
        let p = d.path().join("r.xlsx");
        write_report(&p, &rows, &ReportMeta { master_name: "m.pdf", destination: "/out", generated_at: "2026-11-10 16:00:00" }).unwrap();

        let mut wb: Xlsx<_> = open_workbook(&p).unwrap();
        assert_eq!(wb.sheet_names(), vec!["Detalle", "Resumen", "Incidencias"]);

        let det = wb.worksheet_range("Detalle").unwrap();
        assert_eq!(det.height(), 4, "encabezado + 1 fila por página");
        assert_eq!(cell(&det[(0, 2)]), "Página del maestro");
        assert_eq!(cell(&det[(1, 2)]), "1");
        assert_eq!(cell(&det[(3, 2)]), "3");
        assert_eq!(cell(&det[(1, 7)]), "20260001A");
        assert_eq!(cell(&det[(1, 8)]), "20260001A.pdf", "destino solo si PROCESADO");
        assert_eq!(cell(&det[(3, 8)]), "", "sin archivo si no se generó");
        assert_eq!(cell(&det[(2, 5)]), "'=HYPERLINK(\"http://x\")", "fórmula neutralizada, como texto");

        let inc = wb.worksheet_range("Incidencias").unwrap();
        assert_eq!(inc.height(), 3, "encabezado + 2 incidencias (no PROCESADO)");
        assert_eq!(cell(&inc[(1, 0)]), "2");

        let res = wb.worksheet_range("Resumen").unwrap();
        let find = |k: &str| res.rows().find(|r| cell(&r[0]) == k).map(|r| cell(&r[1])).unwrap();
        assert_eq!(find("Total de páginas de origen"), "3");
        assert_eq!(find("Procesados"), "1");
        assert_eq!(find("Sin código"), "1");
        assert_eq!(find("Duplicados"), "1");
    }
}
