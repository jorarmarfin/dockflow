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
