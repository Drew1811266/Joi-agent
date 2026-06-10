mod commands;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::joi_health_check])
        .run(tauri::generate_context!())
        .expect("failed to run Joi Agent");
}
