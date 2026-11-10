mod batch;
mod commands;
mod errors;
mod excel_report;
mod master_pdf;
mod models;
mod parser;
mod pdf_split;
mod pdf_text;
mod processor;
mod validator;

#[cfg(test)]
mod tests;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(commands::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::analyze_batch,
            commands::set_destination,
            commands::correct_code,
            commands::preview_page,
            commands::open_original,
            commands::process_batch,
            commands::cancel_batch,
            commands::open_destination,
            commands::export_report,
            commands::open_report,
        ])
        .build(tauri::generate_context!())
        .expect("error al iniciar DocFlow")
        .run(|_, event| {
            if let tauri::RunEvent::Exit = event {
                commands::cleanup_previews();
            }
        });
}
