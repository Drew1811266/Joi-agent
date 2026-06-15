use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::assets::{AssetImportInput, AssetService};
use crate::db::Database;
use crate::error::{JoiError, JoiResult};
use crate::models::{
    Asset, Brand, CreativeDirection, MemoryEntry, ProductUnderstanding, Project, ProjectVersion,
};
use crate::project_package::{ProjectExportInput, ProjectImportInput, ProjectPackageService};
use crate::repositories::{
    AssetCreate, BrandCreate, BrandUpdate, MemoryEntryCreate, ProjectCreate, ProjectUpdate,
    Repository,
};
use crate::snapshots::{ProjectSnapshotService, SaveSnapshotInput};
use crate::understanding::{
    generate_brief_understanding as generate_understanding, BriefUnderstandingInput,
    BriefUnderstandingResult,
};

pub struct AppState {
    pub db: Mutex<Database>,
    pub asset_root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthResponse {
    pub status: String,
    pub app_name: String,
    pub phase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BrandInput {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BrandUpdateInput {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectInput {
    pub brand_id: String,
    pub title: String,
    pub advertising_goal: String,
    pub duration_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectUpdateInput {
    pub id: String,
    pub title: String,
    pub advertising_goal: String,
    pub duration_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetImportCommandInput {
    pub project_id: String,
    pub kind: String,
    pub source_path: PathBuf,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotInput {
    pub project_id: String,
    pub label: Option<String>,
    pub change_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RestoreVersionInput {
    pub project_id: String,
    pub version_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectExportCommandInput {
    pub project_id: String,
    pub export_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectExportCommandResult {
    pub project_json_path: PathBuf,
    pub assets_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectImportCommandInput {
    pub project_json_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectImportCommandResult {
    pub project_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryEntryInput {
    pub scope: String,
    pub brand_id: Option<String>,
    pub project_id: Option<String>,
    pub content: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryListInput {
    pub scope: String,
    pub brand_id: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReferenceAssetInput {
    pub project_id: String,
    pub kind: String,
    pub display_name: String,
    pub source_uri: String,
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_health_check() -> HealthResponse {
    HealthResponse {
        status: "ready".to_string(),
        app_name: "Joi Agent".to_string(),
        phase: "Phase 1 local data store".to_string(),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_create_brand(state: State<'_, AppState>, input: BrandInput) -> JoiResult<Brand> {
    create_brand(state.inner(), input)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_list_brands(state: State<'_, AppState>) -> JoiResult<Vec<Brand>> {
    list_brands(state.inner())
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_get_brand(state: State<'_, AppState>, id: String) -> JoiResult<Brand> {
    get_brand(state.inner(), id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_update_brand(state: State<'_, AppState>, input: BrandUpdateInput) -> JoiResult<Brand> {
    update_brand(state.inner(), input)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_create_project(state: State<'_, AppState>, input: ProjectInput) -> JoiResult<Project> {
    create_project(state.inner(), input)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_list_projects(
    state: State<'_, AppState>,
    brand_id: Option<String>,
) -> JoiResult<Vec<Project>> {
    list_projects(state.inner(), brand_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_get_project(state: State<'_, AppState>, id: String) -> JoiResult<Project> {
    get_project(state.inner(), id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_update_project(
    state: State<'_, AppState>,
    input: ProjectUpdateInput,
) -> JoiResult<Project> {
    update_project(state.inner(), input)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_import_asset(
    state: State<'_, AppState>,
    input: AssetImportCommandInput,
) -> JoiResult<Asset> {
    import_asset(state.inner(), input)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_list_assets(state: State<'_, AppState>, project_id: String) -> JoiResult<Vec<Asset>> {
    list_assets(state.inner(), project_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_save_project_snapshot(
    state: State<'_, AppState>,
    input: SnapshotInput,
) -> JoiResult<ProjectVersion> {
    save_project_snapshot(state.inner(), input)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_list_project_versions(
    state: State<'_, AppState>,
    project_id: String,
) -> JoiResult<Vec<ProjectVersion>> {
    list_project_versions(state.inner(), project_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_restore_project_version(
    state: State<'_, AppState>,
    input: RestoreVersionInput,
) -> JoiResult<()> {
    restore_project_version(state.inner(), input)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_export_project(
    state: State<'_, AppState>,
    input: ProjectExportCommandInput,
) -> JoiResult<ProjectExportCommandResult> {
    export_project(state.inner(), input)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_import_project(
    state: State<'_, AppState>,
    input: ProjectImportCommandInput,
) -> JoiResult<ProjectImportCommandResult> {
    import_project(state.inner(), input)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_create_memory_entry(
    state: State<'_, AppState>,
    input: MemoryEntryInput,
) -> JoiResult<MemoryEntry> {
    create_memory_entry(state.inner(), input)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_list_memory_entries(
    state: State<'_, AppState>,
    input: MemoryListInput,
) -> JoiResult<Vec<MemoryEntry>> {
    list_memory_entries(state.inner(), input)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_generate_brief_understanding(
    state: State<'_, AppState>,
    input: BriefUnderstandingInput,
) -> JoiResult<BriefUnderstandingResult> {
    generate_brief_understanding(state.inner(), input)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_list_product_understandings(
    state: State<'_, AppState>,
    project_id: String,
) -> JoiResult<Vec<ProductUnderstanding>> {
    list_product_understandings(state.inner(), project_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_list_creative_directions(
    state: State<'_, AppState>,
    project_id: String,
) -> JoiResult<Vec<CreativeDirection>> {
    list_creative_directions(state.inner(), project_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_create_reference_asset(
    state: State<'_, AppState>,
    input: ReferenceAssetInput,
) -> JoiResult<Asset> {
    create_reference_asset(state.inner(), input)
}

pub fn create_brand(state: &AppState, input: BrandInput) -> JoiResult<Brand> {
    let db = lock_db(state)?;
    Repository::new(db.connection()).create_brand(BrandCreate {
        name: input.name,
        description: input.description,
    })
}

pub fn list_brands(state: &AppState) -> JoiResult<Vec<Brand>> {
    let db = lock_db(state)?;
    Repository::new(db.connection()).list_brands()
}

pub fn get_brand(state: &AppState, id: String) -> JoiResult<Brand> {
    let db = lock_db(state)?;
    Repository::new(db.connection()).get_brand(&id)
}

pub fn update_brand(state: &AppState, input: BrandUpdateInput) -> JoiResult<Brand> {
    let db = lock_db(state)?;
    Repository::new(db.connection()).update_brand(BrandUpdate {
        id: input.id,
        name: input.name,
        description: input.description,
    })
}

pub fn create_project(state: &AppState, input: ProjectInput) -> JoiResult<Project> {
    let db = lock_db(state)?;
    Repository::new(db.connection()).create_project(ProjectCreate {
        brand_id: input.brand_id,
        title: input.title,
        advertising_goal: input.advertising_goal,
        duration_seconds: input.duration_seconds,
    })
}

pub fn list_projects(state: &AppState, brand_id: Option<String>) -> JoiResult<Vec<Project>> {
    let db = lock_db(state)?;
    Repository::new(db.connection()).list_projects(brand_id.as_deref())
}

pub fn get_project(state: &AppState, id: String) -> JoiResult<Project> {
    let db = lock_db(state)?;
    Repository::new(db.connection()).get_project(&id)
}

pub fn update_project(state: &AppState, input: ProjectUpdateInput) -> JoiResult<Project> {
    let db = lock_db(state)?;
    Repository::new(db.connection()).update_project(ProjectUpdate {
        id: input.id,
        title: input.title,
        advertising_goal: input.advertising_goal,
        duration_seconds: input.duration_seconds,
    })
}

pub fn import_asset(state: &AppState, input: AssetImportCommandInput) -> JoiResult<Asset> {
    let db = lock_db(state)?;
    AssetService::new(db.connection(), state.asset_root.clone()).import_local_file(
        AssetImportInput {
            project_id: input.project_id,
            kind: input.kind,
            source_path: input.source_path,
            display_name: input.display_name,
        },
    )
}

pub fn list_assets(state: &AppState, project_id: String) -> JoiResult<Vec<Asset>> {
    let db = lock_db(state)?;
    Repository::new(db.connection()).list_assets(&project_id)
}

pub fn save_project_snapshot(state: &AppState, input: SnapshotInput) -> JoiResult<ProjectVersion> {
    let db = lock_db(state)?;
    ProjectSnapshotService::new(db.connection()).save_snapshot(SaveSnapshotInput {
        project_id: input.project_id,
        label: input.label.unwrap_or_default(),
        change_reason: input.change_reason.unwrap_or_default(),
        changed_entities: Vec::new(),
        created_by: "user".to_string(),
        is_final_candidate: false,
    })
}

pub fn list_project_versions(
    state: &AppState,
    project_id: String,
) -> JoiResult<Vec<ProjectVersion>> {
    let db = lock_db(state)?;
    Repository::new(db.connection()).list_project_versions(&project_id)
}

pub fn restore_project_version(state: &AppState, input: RestoreVersionInput) -> JoiResult<()> {
    let db = lock_db(state)?;
    ProjectSnapshotService::new(db.connection())
        .restore_project_version(&input.project_id, &input.version_id)
}

pub fn export_project(
    state: &AppState,
    input: ProjectExportCommandInput,
) -> JoiResult<ProjectExportCommandResult> {
    let db = lock_db(state)?;
    let result = ProjectPackageService::new(db.connection(), state.asset_root.clone())
        .export_project(ProjectExportInput {
            project_id: input.project_id,
            export_dir: input.export_dir,
        })?;
    Ok(ProjectExportCommandResult {
        project_json_path: result.project_json_path,
        assets_dir: result.assets_dir,
    })
}

pub fn import_project(
    state: &AppState,
    input: ProjectImportCommandInput,
) -> JoiResult<ProjectImportCommandResult> {
    let db = lock_db(state)?;
    let result = ProjectPackageService::new(db.connection(), state.asset_root.clone())
        .import_project(ProjectImportInput {
            project_json_path: input.project_json_path,
        })?;
    Ok(ProjectImportCommandResult {
        project_id: result.project_id,
    })
}

pub fn create_memory_entry(state: &AppState, input: MemoryEntryInput) -> JoiResult<MemoryEntry> {
    let db = lock_db(state)?;
    Repository::new(db.connection()).create_memory_entry(MemoryEntryCreate {
        scope: input.scope,
        brand_id: input.brand_id,
        project_id: input.project_id,
        content: input.content,
        source: input.source,
    })
}

pub fn list_memory_entries(
    state: &AppState,
    input: MemoryListInput,
) -> JoiResult<Vec<MemoryEntry>> {
    let db = lock_db(state)?;
    Repository::new(db.connection()).list_memory_entries(
        &input.scope,
        input.brand_id.as_deref(),
        input.project_id.as_deref(),
    )
}

pub fn generate_brief_understanding(
    state: &AppState,
    input: BriefUnderstandingInput,
) -> JoiResult<BriefUnderstandingResult> {
    let db = lock_db(state)?;
    generate_understanding(&Repository::new(db.connection()), input)
}

pub fn list_product_understandings(
    state: &AppState,
    project_id: String,
) -> JoiResult<Vec<ProductUnderstanding>> {
    let db = lock_db(state)?;
    Repository::new(db.connection()).list_product_understandings(&project_id)
}

pub fn list_creative_directions(
    state: &AppState,
    project_id: String,
) -> JoiResult<Vec<CreativeDirection>> {
    let db = lock_db(state)?;
    Repository::new(db.connection()).list_creative_directions(&project_id)
}

pub fn create_reference_asset(state: &AppState, input: ReferenceAssetInput) -> JoiResult<Asset> {
    if input.display_name.trim().is_empty() {
        return Err(JoiError::Validation(
            "Reference display name is required".to_string(),
        ));
    }
    if input.source_uri.trim().is_empty() {
        return Err(JoiError::Validation(
            "Reference source URI is required".to_string(),
        ));
    }

    let db = lock_db(state)?;
    Repository::new(db.connection()).create_asset(AssetCreate {
        project_id: input.project_id,
        kind: input.kind,
        display_name: input.display_name.trim().to_string(),
        relative_path: String::new(),
        source_uri: input.source_uri.trim().to_string(),
        mime_type: "text/uri-list".to_string(),
        file_size_bytes: 0,
        sha256: String::new(),
    })
}

fn lock_db(state: &AppState) -> JoiResult<MutexGuard<'_, Database>> {
    state
        .db
        .lock()
        .map_err(|_| JoiError::Database("database lock poisoned".to_string()))
}
