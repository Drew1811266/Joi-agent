mod common;

use std::path::PathBuf;
use std::sync::Mutex;

use common::TestApp;
use joi_agent_lib::agent_runtime::AgentPlanInput;
use joi_agent_lib::commands::{
    create_brand, create_memory_entry, create_project, create_reference_asset,
    generate_brief_understanding, generate_memory_candidates, generate_prompt_packages,
    generate_research_report, generate_storyboard, get_agent_runtime_status, get_brand,
    get_project, get_prompt_adapter_profiles, joi_health_check, list_agent_runs, list_brands,
    list_creative_directions, list_memory_entries, list_product_understandings,
    list_project_versions, list_projects, list_prompt_packages, list_research_reports,
    list_storyboards, regenerate_shot, resolve_workspace_root, save_project_snapshot,
    start_agent_plan, update_brand, update_memory_status, update_project, update_prompt_package,
    update_shot, AppState, AssetImportCommandInput, BrandInput, BrandUpdateInput, MemoryEntryInput,
    MemoryListInput, MemoryStatusInput, ProjectExportCommandInput, ProjectImportCommandInput,
    ProjectInput, ProjectUpdateInput, PromptPackageUpdateInput, ReferenceAssetInput,
    RestoreVersionInput, ShotUpdateInput, SnapshotInput,
};
use joi_agent_lib::db::Database;
use joi_agent_lib::error::JoiError;
use joi_agent_lib::memory_curation::MemoryCurationInput;
use joi_agent_lib::prompt_adapter::PromptGenerationInput;
use joi_agent_lib::research::{ResearchReportInput, ResearchSourceInput};
use joi_agent_lib::storyboard::{ShotRegenerationInput, StoryboardGenerationInput};
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

    let memory_curation: MemoryCurationInput = serde_json::from_value(json!({
        "project_id": "project-1",
        "feedback_text": "Keep the opening shot more tactile.",
        "include_research_reports": true
    }))
    .expect("memory curation input");
    assert!(memory_curation.include_research_reports);

    let memory_status: MemoryStatusInput = serde_json::from_value(json!({
        "id": "memory-1",
        "status": "accepted"
    }))
    .expect("memory status input");
    assert_eq!(memory_status.status, "accepted");

    let storyboard_generation: StoryboardGenerationInput = serde_json::from_value(json!({
        "project_id": "project-1",
        "user_direction": "Keep tactile product proof before the walk.",
        "preferred_duration_seconds": 15,
        "preferred_shot_count": 5
    }))
    .expect("storyboard generation input");
    assert_eq!(storyboard_generation.preferred_shot_count, Some(5));

    let shot_update: ShotUpdateInput = serde_json::from_value(json!({
        "id": "shot-1",
        "duration_seconds": 3,
        "visual_description": "Close fabric texture detail.",
        "model_action": "Model lifts sleeve edge.",
        "garment_focus": "fabric texture",
        "camera_movement": "macro slide",
        "scene": "studio insert",
        "lighting": "soft side light",
        "transition": "match cut",
        "subtitle_or_text": "Texture that moves",
        "rationale": "Clarifies product proof.",
        "is_locked": false
    }))
    .expect("shot update input");
    assert_eq!(shot_update.garment_focus, "fabric texture");

    let shot_regeneration: ShotRegenerationInput = serde_json::from_value(json!({
        "project_id": "project-1",
        "storyboard_id": "storyboard-1",
        "shot_id": "shot-1",
        "revision_note": "Make the garment proof clearer."
    }))
    .expect("shot regeneration input");
    assert_eq!(shot_regeneration.shot_id, "shot-1");

    let prompt_generation: PromptGenerationInput = serde_json::from_value(json!({
        "project_id": "project-1",
        "shot_ids": ["shot-1"],
        "image_brief": "",
        "target_platforms": ["jimeng_video", "grok_video"],
        "user_direction": "Keep prompts concise."
    }))
    .expect("prompt generation input");
    assert_eq!(prompt_generation.target_platforms.len(), 2);

    let prompt_update: PromptPackageUpdateInput = serde_json::from_value(json!({
        "id": "prompt-1",
        "prompt_text": "edited prompt",
        "negative_prompt": "edited negative",
        "parameters_json": {"format_version": "joi.prompt_package_parameters.v1"},
        "is_locked": true
    }))
    .expect("prompt update input");
    assert!(prompt_update.is_locked);
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
fn state_helpers_generate_memory_candidates_and_update_status() {
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
    generate_research_report(
        &state,
        ResearchReportInput {
            project_id: project.id.clone(),
            research_goal: "Find reference angles".to_string(),
            market_focus: "outerwear".to_string(),
            platform_focus: vec!["jimeng_video".to_string()],
            source_materials: vec![ResearchSourceInput {
                title: "Reference note".to_string(),
                url: "https://example.com/reference".to_string(),
                source_type: "reference".to_string(),
                excerpt: "Texture details support premium positioning.".to_string(),
            }],
        },
    )
    .expect("research report");

    let result = generate_memory_candidates(
        &state,
        MemoryCurationInput {
            project_id: project.id.clone(),
            feedback_text: String::new(),
            include_research_reports: true,
        },
    )
    .expect("memory candidates");

    assert_eq!(result.candidates.len(), 1);
    assert_eq!(result.agent_run.runtime_mode, "local_memory_bridge");
    assert_eq!(result.agent_events.len(), 5);

    let accepted = update_memory_status(
        &state,
        MemoryStatusInput {
            id: result.candidates[0].entry.id.clone(),
            status: "accepted".to_string(),
        },
    )
    .expect("accept memory");
    assert_eq!(accepted.status, "accepted");

    let memories = list_memory_entries(
        &state,
        MemoryListInput {
            scope: "project".to_string(),
            brand_id: None,
            project_id: Some(project.id),
        },
    )
    .expect("list memory");
    assert_eq!(memories.len(), 1);
    assert_eq!(memories[0].status, "accepted");
}

#[test]
fn state_helpers_generate_list_update_and_regenerate_storyboard() {
    let (_app, state) = test_state();
    let brand = create_brand(
        &state,
        BrandInput {
            name: "Atelier Joi".to_string(),
            description: "Premium womenswear".to_string(),
        },
    )
    .expect("brand");
    let project = create_project(
        &state,
        ProjectInput {
            brand_id: brand.id,
            title: "Spring Drop Film".to_string(),
            advertising_goal: "Launch awareness for a lightweight trench".to_string(),
            duration_seconds: 15,
        },
    )
    .expect("project");
    generate_brief_understanding(
        &state,
        BriefUnderstandingInput {
            project_id: project.id.clone(),
            brief_text: "15 second outerwear launch ad".to_string(),
            product_name: "Lightweight trench".to_string(),
            category: "outerwear".to_string(),
            audience: "urban commuters".to_string(),
            target_platforms: vec!["jimeng_video".to_string(), "grok_video".to_string()],
            selling_points_text: "water-resistant cotton, soft structure, easy movement"
                .to_string(),
            visual_direction: "clean studio walk with close fabric texture".to_string(),
            constraints_text: "avoid heavy winter styling".to_string(),
            reference_asset_ids: Vec::new(),
        },
    )
    .expect("understanding");

    let result = generate_storyboard(
        &state,
        StoryboardGenerationInput {
            project_id: project.id.clone(),
            user_direction: "Make the opening tactile.".to_string(),
            preferred_duration_seconds: Some(15),
            preferred_shot_count: Some(5),
        },
    )
    .expect("generate storyboard");

    assert_eq!(result.shots.len(), 5);
    assert_eq!(result.agent_run.runtime_mode, "local_storyboard_bridge");

    let storyboards = list_storyboards(&state, project.id.clone()).expect("list storyboards");
    assert_eq!(storyboards.len(), 1);
    assert_eq!(storyboards[0].shots.len(), 5);

    let edited = update_shot(
        &state,
        ShotUpdateInput {
            id: result.shots[0].shot.id.clone(),
            duration_seconds: result.shots[0].shot.duration_seconds,
            visual_description: "Edited opening product entrance.".to_string(),
            model_action: "Model steps into frame.".to_string(),
            garment_focus: "trench silhouette".to_string(),
            camera_movement: "slow push".to_string(),
            scene: "studio".to_string(),
            lighting: "soft side light".to_string(),
            transition: "cut on movement".to_string(),
            subtitle_or_text: "Built for changing weather".to_string(),
            rationale: "User edit makes opening clearer.".to_string(),
            is_locked: false,
        },
    )
    .expect("update shot");
    assert_eq!(
        edited.visual_description,
        "Edited opening product entrance."
    );

    let regenerated = regenerate_shot(
        &state,
        ShotRegenerationInput {
            project_id: project.id,
            storyboard_id: result.storyboard.id,
            shot_id: result.shots[1].shot.id.clone(),
            revision_note: "Make this shot a clearer macro fabric insert.".to_string(),
        },
    )
    .expect("regenerate shot");
    assert_eq!(
        regenerated.agent_run.runtime_mode,
        "local_storyboard_regeneration_bridge"
    );
    assert_eq!(regenerated.shot.shot.id, result.shots[1].shot.id);
}

#[test]
fn state_helpers_generate_list_and_update_prompt_packages() {
    let (_app, state) = test_state();
    let brand = create_brand(
        &state,
        BrandInput {
            name: "Atelier Joi".to_string(),
            description: "Premium womenswear".to_string(),
        },
    )
    .expect("brand");
    let project = create_project(
        &state,
        ProjectInput {
            brand_id: brand.id,
            title: "Spring Drop Film".to_string(),
            advertising_goal: "Launch awareness for a lightweight trench".to_string(),
            duration_seconds: 15,
        },
    )
    .expect("project");
    generate_brief_understanding(
        &state,
        BriefUnderstandingInput {
            project_id: project.id.clone(),
            brief_text: "15 second outerwear launch ad".to_string(),
            product_name: "Lightweight trench".to_string(),
            category: "outerwear".to_string(),
            audience: "urban commuters".to_string(),
            target_platforms: vec!["jimeng_video".to_string(), "grok_video".to_string()],
            selling_points_text: "water-resistant cotton, soft structure, easy movement"
                .to_string(),
            visual_direction: "clean studio walk with close fabric texture".to_string(),
            constraints_text: "avoid heavy winter styling".to_string(),
            reference_asset_ids: Vec::new(),
        },
    )
    .expect("understanding");
    let storyboard = generate_storyboard(
        &state,
        StoryboardGenerationInput {
            project_id: project.id.clone(),
            user_direction: "Make the opening tactile.".to_string(),
            preferred_duration_seconds: Some(15),
            preferred_shot_count: Some(5),
        },
    )
    .expect("storyboard");

    let profiles = get_prompt_adapter_profiles();
    assert_eq!(profiles.len(), 5);

    let result = generate_prompt_packages(
        &state,
        PromptGenerationInput {
            project_id: project.id.clone(),
            shot_ids: vec![storyboard.shots[0].shot.id.clone()],
            image_brief: "Full-body studio model photo.".to_string(),
            target_platforms: vec!["jimeng_video".into(), "gpt_image_2".into()],
            user_direction: "Make output production-ready.".into(),
        },
    )
    .expect("prompt generation");

    assert_eq!(result.packages.len(), 2);
    assert_eq!(result.agent_run.runtime_mode, "local_prompt_adapter_bridge");
    assert_eq!(result.agent_events.len(), 5);

    let listed = list_prompt_packages(&state, project.id).expect("listed prompts");
    assert_eq!(listed.len(), 2);

    let updated = update_prompt_package(
        &state,
        PromptPackageUpdateInput {
            id: listed[0].package.id.clone(),
            prompt_text: "edited prompt".into(),
            negative_prompt: "edited negative".into(),
            parameters_json: listed[0].package.parameters_json.clone(),
            is_locked: true,
        },
    )
    .expect("updated prompt");
    assert_eq!(updated.package.prompt_text, "edited prompt");
    assert_eq!(updated.package.negative_prompt, "edited negative");
    assert!(updated.package.is_locked);
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
