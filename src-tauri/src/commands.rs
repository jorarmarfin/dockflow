use crate::batch::Batch;
use crate::errors::DocFlowError;
use crate::models::BatchView;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_opener::OpenerExt;

#[derive(Default)]
pub struct AppState {
    pub batch: Mutex<Option<Batch>>,
    pub cancel: AtomicBool,
    pub last_report: Mutex<Option<PathBuf>>,
}

fn preview_dir() -> PathBuf {
    std::env::temp_dir().join(format!("docflow-preview-{}", std::process::id()))
}

/// Borra las vistas previas temporales (datos personales) — al re-analizar y al salir.
pub fn cleanup_previews() {
    let _ = std::fs::remove_dir_all(preview_dir());
}

fn with_batch<T>(state: &State<AppState>, f: impl FnOnce(&mut Batch) -> Result<T, DocFlowError>) -> Result<T, DocFlowError> {
    let mut g = state.batch.lock().map_err(|_| DocFlowError::Otro("estado corrupto".into()))?;
    f(g.as_mut().ok_or_else(|| DocFlowError::Otro("Primero analiza un PDF".into()))?)
}

#[tauri::command]
pub fn ping() -> String {
    format!("DocFlow {} · local", env!("CARGO_PKG_VERSION"))
}

#[tauri::command(async)]
pub fn analyze_batch(source: String, state: State<AppState>) -> Result<BatchView, DocFlowError> {
    cleanup_previews();
    let batch = Batch::analyze(Path::new(&source))?;
    let view = batch.view();
    *state.batch.lock().map_err(|_| DocFlowError::Otro("estado corrupto".into()))? = Some(batch);
    Ok(view)
}

#[tauri::command]
pub fn set_destination(dest: String, state: State<AppState>) -> Result<BatchView, DocFlowError> {
    with_batch(&state, |b| { b.set_destination(Path::new(&dest))?; Ok(b.view()) })
}

#[tauri::command]
pub fn correct_code(row_id: String, code: String, state: State<AppState>) -> Result<BatchView, DocFlowError> {
    with_batch(&state, |b| { b.correct(&row_id, &code)?; Ok(b.view()) })
}

/// Abre SOLO la página elegida (PDF temporal de 1 página) en el visor del sistema.
#[tauri::command(async)]
pub fn preview_page(row_id: String, app: AppHandle, state: State<AppState>) -> Result<(), DocFlowError> {
    let bytes = with_batch(&state, |b| b.page_bytes(&row_id))?;
    let dir = preview_dir();
    std::fs::create_dir_all(&dir)?;
    let file = dir.join(format!("pagina-{}.pdf", row_id.trim_start_matches('p')));
    std::fs::write(&file, bytes)?;
    app.opener().open_path(file.to_string_lossy(), None::<&str>).map_err(|e| DocFlowError::Otro(e.to_string()))
}

#[tauri::command]
pub fn open_original(app: AppHandle, state: State<AppState>) -> Result<(), DocFlowError> {
    let path = with_batch(&state, |b| Ok(b.master.clone()))?;
    app.opener().open_path(path.to_string_lossy(), None::<&str>).map_err(|e| DocFlowError::Otro(e.to_string()))
}

#[derive(Clone, serde::Serialize)]
struct Progress { done: usize, total: usize }

/// Genera los PDFs (emite eventos `progress`). Cancelable con `cancel_batch`.
#[tauri::command(async)]
pub fn process_batch(app: AppHandle, state: State<AppState>) -> Result<BatchView, DocFlowError> {
    state.cancel.store(false, Ordering::Relaxed);
    with_batch(&state, |b| {
        b.process(&state.cancel, |done, total| { let _ = app.emit("progress", Progress { done, total }); })?;
        Ok(b.view())
    })
}

#[tauri::command]
pub fn cancel_batch(state: State<AppState>) {
    state.cancel.store(true, Ordering::Relaxed);
}

#[tauri::command]
pub fn open_destination(app: AppHandle, state: State<AppState>) -> Result<(), DocFlowError> {
    let dest = with_batch(&state, |b| b.destination.clone().ok_or_else(|| DocFlowError::Otro("Sin destino".into())))?;
    app.opener().open_path(dest.to_string_lossy(), None::<&str>).map_err(|e| DocFlowError::Otro(e.to_string()))
}

/// Exporta `reporte_procesamiento_YYYYMMDD_HHMMSS.xlsx` a la carpeta destino. Devuelve la ruta.
#[tauri::command]
pub fn export_report(state: State<AppState>) -> Result<String, DocFlowError> {
    let now = chrono::Local::now();
    let path = with_batch(&state, |b| {
        let dest = b.destination.clone().ok_or_else(|| DocFlowError::Otro("Primero elige la carpeta destino".into()))?;
        std::fs::create_dir_all(&dest)?;
        let path = dest.join(format!("reporte_procesamiento_{}.xlsx", now.format("%Y%m%d_%H%M%S")));
        let master_name = b.master.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        crate::excel_report::write_report(&path, &b.rows, &crate::excel_report::ReportMeta {
            master_name: &master_name,
            destination: &dest.to_string_lossy(),
            generated_at: &now.format("%Y-%m-%d %H:%M:%S").to_string(),
        })?;
        Ok(path)
    })?;
    *state.last_report.lock().map_err(|_| DocFlowError::Otro("estado corrupto".into()))? = Some(path.clone());
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn open_report(app: AppHandle, state: State<AppState>) -> Result<(), DocFlowError> {
    let path = state.last_report.lock().map_err(|_| DocFlowError::Otro("estado corrupto".into()))?
        .clone().ok_or_else(|| DocFlowError::Otro("Aún no hay reporte exportado".into()))?;
    app.opener().open_path(path.to_string_lossy(), None::<&str>).map_err(|e| DocFlowError::Otro(e.to_string()))
}
