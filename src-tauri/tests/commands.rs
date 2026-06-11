mod common;

use std::path::PathBuf;
use std::sync::Mutex;

use common::TestApp;
use joi_agent_lib::commands::{
    create_brand, create_memory_entry, create_project, get_brand, get_project, joi_health_check,
    list_brands, list_memory_entries, list_project_versions, list_projects, save_project_snapshot,
    AppState, AssetImportCommandInput, BrandInput, MemoryEntryInput, MemoryListInput,
    ProjectExportCommandInput, ProjectImportCommandInput, ProjectInput, RestoreVersionInput,
    SnapshotInput,
};
use joi_agent_lib::db::Database;
use serde_json::json;

fn test_state() -> (TestApp, AppState) {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let asset_root = app.temp_dir.path().join("assets");
    (
        app,
        AppState {
            db: Mutex::new(db),
            asset_root,
        },
    )
}

#[test]
fn health_response_reports_ready_app_and_phase() {
    let health = joi_health_check();
    let value = serde_json::to_value(&health).expect("serialize health");

    assert_eq!(value["ready"], true);
    assert_eq!(value["app"], "Joi Agent");
    assert_eq!(value["phase"], "Phase 1 local data store");
}

#[test]
fn command_inputs_round_trip_through_json() {
    let brand: BrandInput = serde_json::from_value(json!({
        "name": "Atelier Joi",
        "description": "Premium womenswear"
    }))
    .expect("brand input");
    assert_eq!(brand.name, "Atelier Joi");
    assert_eq!(
        serde_json::to_value(&brand).expect("serialize brand")["description"],
        "Premium womenswear"
    );

    let project: ProjectInput = serde_json::from_value(json!({
        "brand_id": "brand-1",
        "title": "15s launch film",
        "advertising_goal": "New seasonal drop",
        "duration_seconds": 15
    }))
    .expect("project input");
    assert_eq!(project.duration_seconds, 15);

    let asset: AssetImportCommandInput = serde_json::from_value(json!({
        "project_id": "project-1",
        "kind": "product_image",
        "source_path": "D:/tmp/source.png",
        "display_name": "Hero product"
    }))
    .expect("asset input");
    assert_eq!(asset.source_path, PathBuf::from("D:/tmp/source.png"));

    let snapshot: SnapshotInput = serde_json::from_value(json!({
        "project_id": "project-1",
        "label": "Draft",
        "change_reason": "Initial version"
    }))
    .expect("snapshot input");
    assert_eq!(snapshot.label.as_deref(), Some("Draft"));

    let restore: RestoreVersionInput = serde_json::from_value(json!({
        "project_id": "project-1",
        "version_id": "version-1"
    }))
    .expect("restore input");
    assert_eq!(restore.version_id, "version-1");

    let export: ProjectExportCommandInput = serde_json::from_value(json!({
        "project_id": "project-1",
        "export_dir": "D:/tmp/export"
    }))
    .expect("export input");
    assert_eq!(export.export_dir, PathBuf::from("D:/tmp/export"));

    let import: ProjectImportCommandInput = serde_json::from_value(json!({
        "project_json_path": "D:/tmp/export/project.joi-project.json"
    }))
    .expect("import input");
    assert_eq!(
        import.project_json_path,
        PathBuf::from("D:/tmp/export/project.joi-project.json")
    );

    let memory: MemoryEntryInput = serde_json::from_value(json!({
        "scope": "brand",
        "brand_id": "brand-1",
        "project_id": null,
        "content": "Use clean studio lighting",
        "source": "user"
    }))
    .expect("memory input");
    assert_eq!(memory.scope, "brand");

    let list_memory: MemoryListInput = serde_json::from_value(json!({
        "scope": "brand",
        "brand_id": "brand-1",
        "project_id": null
    }))
    .expect("memory list input");
    assert_eq!(list_memory.brand_id.as_deref(), Some("brand-1"));
}

#[test]
fn state_helpers_create_and_list_brand_project_memory_and_snapshot() {
    let (_app, state) = test_state();

    let brand = create_brand(
        &state,
        BrandInput {
            name: "Atelier Joi".to_string(),
            description: "Premium womenswear".to_string(),
        },
    )
    .expect("create brand");
    let fetched_brand = get_brand(&state, brand.id.clone()).expect("get brand");
    let brands = list_brands(&state).expect("list brands");
    assert_eq!(fetched_brand.id, brand.id);
    assert_eq!(brands.len(), 1);

    let project = create_project(
        &state,
        ProjectInput {
            brand_id: brand.id.clone(),
            title: "15s launch film".to_string(),
            advertising_goal: "New seasonal drop".to_string(),
            duration_seconds: 15,
        },
    )
    .expect("create project");
    let fetched_project = get_project(&state, project.id.clone()).expect("get project");
    let projects = list_projects(&state, Some(brand.id.clone())).expect("list projects");
    assert_eq!(fetched_project.id, project.id);
    assert_eq!(projects.len(), 1);

    let memory = create_memory_entry(
        &state,
        MemoryEntryInput {
            scope: "project".to_string(),
            brand_id: Some(brand.id.clone()),
            project_id: Some(project.id.clone()),
            content: "Keep product fabric texture visible".to_string(),
            source: "user".to_string(),
        },
    )
    .expect("create memory");
    let memories = list_memory_entries(
        &state,
        MemoryListInput {
            scope: "project".to_string(),
            brand_id: None,
            project_id: Some(project.id.clone()),
        },
    )
    .expect("list memory");
    assert_eq!(memories.len(), 1);
    assert_eq!(memories[0].id, memory.id);

    let version = save_project_snapshot(
        &state,
        SnapshotInput {
            project_id: project.id.clone(),
            label: Some("Draft".to_string()),
            change_reason: Some("Initial version".to_string()),
        },
    )
    .expect("save snapshot");
    let versions = list_project_versions(&state, project.id.clone()).expect("list versions");
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0].id, version.id);
}
