//! División de páginas SIN rasterizar: se conserva el contenido vectorial/texto original.
use crate::errors::DocFlowError;
use lopdf::Document;

/// Devuelve un PDF de una sola página (`page_number` base 1) tomado del documento maestro.
pub fn extract_page(master: &Document, page_number: u32) -> Result<Vec<u8>, DocFlowError> {
    let total = master.get_pages().len() as u32;
    if page_number == 0 || page_number > total {
        return Err(DocFlowError::PdfInvalido(format!("página {page_number} fuera de rango (1..{total})")));
    }
    let mut doc = master.clone();
    let others: Vec<u32> = (1..=total).filter(|p| *p != page_number).collect();
    doc.delete_pages(&others);
    doc.prune_objects();
    doc.renumber_objects();
    let mut out = Vec::new();
    doc.save_to(&mut out).map_err(|e| DocFlowError::PdfInvalido(e.to_string()))?;
    Ok(out)
}
