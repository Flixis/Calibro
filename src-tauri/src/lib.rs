mod commands;
mod database;
mod models;
mod pdf;

use database::{init_db, migrate_db, DbState};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db = init_db(&app.handle())?;
            migrate_db(&db)?;
            app.manage(DbState(std::sync::Mutex::new(db)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::save_calibration,
            commands::update_calibration,
            commands::get_calibrations,
            commands::open_folder,
            commands::generate_pdf
        ])
        .run(tauri::generate_context!())
}
