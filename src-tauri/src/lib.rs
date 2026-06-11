pub mod assets;
pub mod commands;
pub mod db;
pub mod error;
pub mod models;
pub mod project_package;
pub mod repositories;
pub mod snapshots;

mod validation;

use std::sync::Mutex;

use commands::AppState;
use db::Database;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;

            let db = Database::open(app_data_dir.join("joi.db"))?;
            db.migrate()?;

            let asset_root = app_data_dir.join("assets");
            std::fs::create_dir_all(&asset_root)?;
            app.manage(AppState {
                db: Mutex::new(db),
                asset_root,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::joi_health_check,
            commands::joi_create_brand,
            commands::joi_list_brands,
            commands::joi_get_brand,
            commands::joi_update_brand,
            commands::joi_create_project,
            commands::joi_list_projects,
            commands::joi_get_project,
            commands::joi_update_project,
            commands::joi_import_asset,
            commands::joi_list_assets,
            commands::joi_save_project_snapshot,
            commands::joi_list_project_versions,
            commands::joi_restore_project_version,
            commands::joi_export_project,
            commands::joi_import_project,
            commands::joi_create_memory_entry,
            commands::joi_list_memory_entries
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Joi Agent");
}
