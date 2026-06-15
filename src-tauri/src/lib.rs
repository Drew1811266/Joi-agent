pub mod agent_context;
pub mod agent_runtime;
pub mod assets;
pub mod commands;
pub mod db;
pub mod delivery_report;
pub mod error;
pub mod hermes_bridge;
pub mod memory_curation;
pub mod models;
pub mod project_package;
pub mod prompt_adapter;
pub mod quality_review;
pub mod repositories;
pub mod research;
pub mod snapshots;
pub mod storyboard;
pub mod understanding;

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
            commands::joi_list_memory_entries,
            commands::joi_generate_memory_candidates,
            commands::joi_update_memory_status,
            commands::joi_generate_storyboard,
            commands::joi_list_storyboards,
            commands::joi_update_shot,
            commands::joi_regenerate_shot,
            commands::joi_get_prompt_adapter_profiles,
            commands::joi_generate_prompt_packages,
            commands::joi_list_prompt_packages,
            commands::joi_update_prompt_package,
            commands::joi_generate_quality_review,
            commands::joi_list_quality_reviews,
            commands::joi_apply_quality_review_suggestion,
            commands::joi_generate_delivery_report,
            commands::joi_list_delivery_reports,
            commands::joi_update_delivery_report,
            commands::joi_preview_delivery_package,
            commands::joi_generate_brief_understanding,
            commands::joi_list_product_understandings,
            commands::joi_list_creative_directions,
            commands::joi_generate_research_report,
            commands::joi_list_research_reports,
            commands::joi_create_reference_asset,
            commands::joi_get_agent_runtime_status,
            commands::joi_start_agent_plan,
            commands::joi_get_agent_run,
            commands::joi_list_agent_runs
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Joi Agent");
}
