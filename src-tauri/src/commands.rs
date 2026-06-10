use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub app_name: String,
    pub phase: String,
}

#[tauri::command]
pub fn joi_health_check() -> HealthResponse {
    HealthResponse {
        status: "ready".to_string(),
        app_name: "Joi Agent".to_string(),
        phase: "Phase 1 local data store".to_string(),
    }
}
