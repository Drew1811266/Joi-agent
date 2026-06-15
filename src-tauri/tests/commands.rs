mod common;

use std::path::PathBuf;
use std::sync::Mutex;

use common::TestApp;
use joi_agent_lib::agent_runtime::AgentPlanInput;
use joi_agent_lib::commands::{
    create_brand, create_memory_entry, create_project, create_reference_asset,
    generate_brief_understanding, generate_research_report, get_agent_runtime_status, get_brand,
    get_project, joi_health_check, list_agent_runs, list_brands, list_creative_directions,
    list_memory_entries, list_product_understandings, list_project_versions, list_projects,
    list_research_reports, resolve_workspace_root, save_project_snapshot, start_agent_plan,
    update_brand, update_project, AppState, AssetImportCommandInput, BrandInput, BrandUpdateInput,
    MemoryEntryInput, MemoryListInput, ProjectExportCommandInput, ProjectImportCommandInput,
    ProjectInput, ProjectUpdateInput, ReferenceAssetInput, RestoreVersionInput, SnapshotInput,
};
use joi_agent_lib::db::Database;
use joi_agent_lib::error::JoiError;
use joi_agent_lib::research::{ResearchReportInput, ResearchSourceInput};
use joi_agent_lib::understanding::BriefUnderstandingInput;
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

    assert_eq!(value["status"], "ready");
    assert_eq!(value["app_name"], "Joi Agent");
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
    let brand_update: BrandUpdateInput = serde_json::from_value(json!({
        "id": "brand-1",
        "name": "Atelier Joi Studio",
        "description": "Campaign unit"
    }))
    .expect("brand update input");
    assert_eq!(brand_update.id, "brand-1");

    let project: ProjectInput = serde_json::from_value(json!({
        "brand_id": "brand-1",
        "title": "15s launch film",
        "advertising_goal": "New seasonal drop",
        "duration_seconds": 15
    }))
    .expect("project input");
    assert_eq!(project.duration_seconds, 15);
    let project_update: ProjectUpdateInput = serde_json::from_value(json!({
        "id": "project-1",
        "title": "30s launch film",
        "advertising_goal": "Evergreen brand lift",
        "duration_seconds": 30
    }))
    .expect("project update input");
    assert_eq!(project_update.id, "project-1");

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

    let agent_plan: AgentPlanInput = serde_json::from_value(json!({
        "project_id": "project-1",
        "user_goal": "Plan the next content workflow steps"
    }))
    .expect("agent plan input");
    assert_eq!(agent_plan.project_id, "project-1");

    let research_report: ResearchReportInput = serde_json::from_value(json!({
        "project_id": "project-1",
        "research_goal": "Find reference angles",
        "market_focus": "outerwear",
        "platform_focus": ["jimeng_video", "grok_video"],
        "source_materials": [
            {
                "title": "Reference note",
                "url": "https://example.com/reference",
                "source_type": "reference",
                "excerpt": "Texture details support premium positioning."
            }
        ]
    }))
    .expect("research report input");
    assert_eq!(research_report.source_materials[0].title, "Reference note");
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

#[test]
fn generates_brief_understanding_and_lists_saved_records() {
    let (_app, state) = test_state();

    let brand = create_brand(
        &state,
        BrandInput {
            name: "Atelier Joi".to_string(),
            description: "Contemporary womenswear with clean studio lighting".to_string(),
        },
    )
    .unwrap();
    let project = create_project(
        &state,
        ProjectInput {
            brand_id: brand.id,
            title: "Spring Drop Film".to_string(),
            advertising_goal: "Launch awareness".to_string(),
            duration_seconds: 15,
        },
    )
    .unwrap();

    let result = generate_brief_understanding(
        &state,
        BriefUnderstandingInput {
            project_id: project.id.clone(),
            brief_text: "15 second outerwear launch ad".to_string(),
            product_name: "Lightweight trench".to_string(),
            category: "outerwear".to_string(),
            audience: "urban commuters".to_string(),
            target_platforms: vec!["jimeng_video".to_string(), "grok_video".to_string()],
            selling_points_text: "water-resistant cotton, soft structure".to_string(),
            visual_direction: "clean studio walk with close fabric texture".to_string(),
            constraints_text: "avoid heavy winter styling".to_string(),
            reference_asset_ids: Vec::new(),
        },
    )
    .unwrap();

    assert_eq!(
        result.product_understanding.product_name,
        "Lightweight trench"
    );
    assert_eq!(
        result.selling_points,
        vec!["water-resistant cotton", "soft structure"]
    );
    assert_eq!(
        result.missing_questions,
        vec!["Which reference materials should Joi use as visual anchors?".to_string()]
    );

    let understandings = list_product_understandings(&state, project.id.clone()).unwrap();
    assert_eq!(understandings.len(), 1);
    let directions = list_creative_directions(&state, project.id).unwrap();
    assert_eq!(directions.len(), 1);
}

#[test]
fn creates_reference_asset_from_link_input() {
    let (_app, state) = test_state();

    let brand = create_brand(
        &state,
        BrandInput {
            name: "Atelier Joi".to_string(),
            description: "Contemporary womenswear".to_string(),
        },
    )
    .unwrap();
    let project = create_project(
        &state,
        ProjectInput {
            brand_id: brand.id,
            title: "Spring Drop Film".to_string(),
            advertising_goal: "Launch awareness".to_string(),
            duration_seconds: 15,
        },
    )
    .unwrap();

    let asset = create_reference_asset(
        &state,
        ReferenceAssetInput {
            project_id: project.id,
            kind: "link".to_string(),
            display_name: "Spring campaign moodboard".to_string(),
            source_uri: "https://example.com/moodboard".to_string(),
        },
    )
    .unwrap();

    assert_eq!(asset.kind, "link");
    assert_eq!(asset.display_name, "Spring campaign moodboard");
    assert_eq!(asset.source_uri, "https://example.com/moodboard");
    assert_eq!(asset.relative_path, "");
    assert_eq!(asset.mime_type, "text/uri-list");
    assert_eq!(asset.file_size_bytes, 0);
}

#[test]
fn state_helpers_update_brand_and_project() {
    let (_app, state) = test_state();

    let brand = create_brand(
        &state,
        BrandInput {
            name: "Atelier Joi".to_string(),
            description: "Premium womenswear".to_string(),
        },
    )
    .expect("create brand");
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

    let updated_brand = update_brand(
        &state,
        BrandUpdateInput {
            id: brand.id.clone(),
            name: "Atelier Joi Studio".to_string(),
            description: "Campaign unit".to_string(),
        },
    )
    .expect("update brand");
    assert_eq!(updated_brand.name, "Atelier Joi Studio");
    assert_eq!(updated_brand.description, "Campaign unit");

    let updated_project = update_project(
        &state,
        ProjectUpdateInput {
            id: project.id.clone(),
            title: "30s launch film".to_string(),
            advertising_goal: "Evergreen brand lift".to_string(),
            duration_seconds: 30,
        },
    )
    .expect("update project");
    assert_eq!(updated_project.brand_id, brand.id);
    assert_eq!(updated_project.title, "30s launch film");
    assert_eq!(updated_project.advertising_goal, "Evergreen brand lift");
    assert_eq!(updated_project.duration_seconds, 30);
}

#[test]
fn update_helpers_return_not_found_for_missing_ids() {
    let (_app, state) = test_state();

    let brand_error = update_brand(
        &state,
        BrandUpdateInput {
            id: "missing-brand".to_string(),
            name: "Atelier Joi Studio".to_string(),
            description: String::new(),
        },
    )
    .expect_err("missing brand");
    assert!(matches!(brand_error, JoiError::NotFound(message) if message == "brand missing-brand"));

    let project_error = update_project(
        &state,
        ProjectUpdateInput {
            id: "missing-project".to_string(),
            title: "30s launch film".to_string(),
            advertising_goal: String::new(),
            duration_seconds: 30,
        },
    )
    .expect_err("missing project");
    assert!(
        matches!(project_error, JoiError::NotFound(message) if message == "project missing-project")
    );
}

#[test]
fn state_helpers_start_and_list_agent_runs() {
    let (_app, state) = test_state();

    let brand = create_brand(
        &state,
        BrandInput {
            name: "Atelier Joi".to_string(),
            description: "Premium womenswear".to_string(),
        },
    )
    .expect("create brand");
    let project = create_project(
        &state,
        ProjectInput {
            brand_id: brand.id,
            title: "Spring Drop Film".to_string(),
            advertising_goal: "Launch awareness".to_string(),
            duration_seconds: 15,
        },
    )
    .expect("create project");

    let runtime_status = get_agent_runtime_status(&state).expect("runtime status");
    assert_eq!(runtime_status.runtime_kind, "hermes_core");
    assert_eq!(runtime_status.runtime_mode, "local_planner_bridge");

    let result = start_agent_plan(
        &state,
        AgentPlanInput {
            project_id: project.id.clone(),
            user_goal: "Plan the next content workflow steps".to_string(),
        },
    )
    .expect("start agent plan");
    assert_eq!(result.events.len(), 7);
    assert_eq!(result.run.user_goal, "Plan the next content workflow steps");

    let runs = list_agent_runs(&state, project.id).expect("list agent runs");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].run.id, result.run.id);
    assert_eq!(runs[0].events.len(), 7);
}

#[test]
fn state_helpers_generate_and_list_research_reports() {
    let (_app, state) = test_state();

    let brand = create_brand(
        &state,
        BrandInput {
            name: "Atelier Joi".to_string(),
            description: "Premium womenswear".to_string(),
        },
    )
    .expect("create brand");
    let project = create_project(
        &state,
        ProjectInput {
            brand_id: brand.id,
            title: "Spring Drop Film".to_string(),
            advertising_goal: "Launch awareness".to_string(),
            duration_seconds: 15,
        },
    )
    .expect("create project");

    let result = generate_research_report(
        &state,
        ResearchReportInput {
            project_id: project.id.clone(),
            research_goal: "Find reference angles".to_string(),
            market_focus: "outerwear".to_string(),
            platform_focus: vec!["jimeng_video".to_string(), "grok_video".to_string()],
            source_materials: vec![ResearchSourceInput {
                title: "Reference note".to_string(),
                url: "https://example.com/reference".to_string(),
                source_type: "reference".to_string(),
                excerpt: "Texture details support premium positioning.".to_string(),
            }],
        },
    )
    .expect("generate research report");

    assert_eq!(result.report.project_id, project.id);
    assert_eq!(result.agent_run.runtime_mode, "local_research_bridge");
    assert_eq!(result.agent_events.len(), 5);

    let reports = list_research_reports(&state, project.id).expect("list research reports");
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].id, result.report.id);
    assert_eq!(reports[0].sources_json[0]["title"], "Reference note");
}

#[test]
fn resolves_workspace_root_from_src_tauri_child_directory() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let root = temp_dir.path().join("Joi-agent");
    let src_tauri = root.join("src-tauri");
    std::fs::create_dir_all(&src_tauri).expect("src-tauri");
    std::fs::write(root.join("package.json"), "{}").expect("package json");

    let resolved = resolve_workspace_root(&src_tauri);

    assert_eq!(resolved, root);
}
