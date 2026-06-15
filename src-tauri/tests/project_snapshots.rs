mod common;

use chrono::Utc;
use common::TestApp;
use joi_agent_lib::db::Database;
use joi_agent_lib::error::JoiError;
use joi_agent_lib::repositories::{
    AssetCreate, BrandCreate, CreativeDirectionCreate, ProductUnderstandingCreate, ProjectCreate,
    PromptPackageCreate, Repository, ResearchReportCreate, ShotCreate, StoryboardCreate,
};
use joi_agent_lib::snapshots::{ProjectSnapshotService, SaveSnapshotInput};
use rusqlite::params;
use serde_json::json;

#[test]
fn creates_project_snapshot_with_incrementing_version() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Brand".into(),
            description: String::new(),
        })
        .expect("brand");
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id,
            title: "Snapshot Project".into(),
            advertising_goal: String::new(),
            duration_seconds: 15,
        })
        .expect("project");

    let service = ProjectSnapshotService::new(db.connection());
    let version = service
        .save_snapshot(SaveSnapshotInput {
            project_id: project.id.clone(),
            label: "Initial".into(),
            change_reason: "Created project".into(),
            changed_entities: vec!["project".into()],
            created_by: "test".into(),
            is_final_candidate: false,
        })
        .expect("save snapshot");
    let second_version = service
        .save_snapshot(SaveSnapshotInput {
            project_id: project.id.clone(),
            label: "Edited".into(),
            change_reason: "Edited project".into(),
            changed_entities: vec!["project".into()],
            created_by: "test".into(),
            is_final_candidate: false,
        })
        .expect("save second snapshot");

    assert_eq!(version.version_number, 1);
    assert_eq!(version.snapshot_json["project"]["id"], project.id);
    assert_eq!(second_version.version_number, 2);
}

#[test]
fn creates_project_snapshot_with_related_sections() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Brand".into(),
            description: String::new(),
        })
        .expect("brand");
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id.clone(),
            title: "Rich Snapshot Project".into(),
            advertising_goal: String::new(),
            duration_seconds: 15,
        })
        .expect("project");
    let asset = repo
        .create_asset(AssetCreate {
            project_id: project.id.clone(),
            kind: "product_image".into(),
            display_name: "Hero Image".into(),
            relative_path: "projects/project/assets/hero.jpg".into(),
            source_uri: "file:///hero.jpg".into(),
            mime_type: "image/jpeg".into(),
            file_size_bytes: 123,
            sha256: "abc123".into(),
        })
        .expect("asset");
    let report = repo
        .create_research_report(ResearchReportCreate {
            project_id: project.id.clone(),
            summary: "Market summary".into(),
        })
        .expect("research report");
    let understanding = repo
        .create_product_understanding(ProductUnderstandingCreate {
            project_id: project.id.clone(),
            product_name: "Wool Coat".into(),
            category: "Outerwear".into(),
            audience: String::new(),
            selling_points: Vec::new(),
            constraints: Vec::new(),
            notes: String::new(),
        })
        .expect("product understanding");
    let direction = repo
        .create_creative_direction(CreativeDirectionCreate {
            project_id: project.id.clone(),
            title: "Quiet Luxury".into(),
            concept: "Soft movement in city light".into(),
            tone: String::new(),
            visual_style: String::new(),
            scene_direction: String::new(),
            rationale: String::new(),
        })
        .expect("creative direction");
    let storyboard = repo
        .create_storyboard(StoryboardCreate {
            project_id: project.id.clone(),
            title: "15s Film".into(),
            duration_seconds: 15,
        })
        .expect("storyboard");
    let shot = repo
        .create_shot(ShotCreate {
            storyboard_id: storyboard.id.clone(),
            shot_number: 1,
            duration_seconds: 4,
            description: "Model enters frame".into(),
        })
        .expect("shot");
    let prompt = repo
        .create_prompt_package(PromptPackageCreate {
            project_id: project.id.clone(),
            shot_id: shot.id.clone(),
            platform: "jimeng_video".into(),
            modality: "video".into(),
            prompt_text: "A refined fashion ad shot".into(),
        })
        .expect("prompt package");
    let now = Utc::now().to_rfc3339();
    db.connection()
        .execute(
            "INSERT INTO memory_entries (
                id, scope, brand_id, project_id, content, source, source_entity_type,
                source_entity_id, confidence, status, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                "memory-1",
                "project",
                Option::<&str>::None,
                project.id,
                "Use natural movement",
                "test",
                "project",
                project.id,
                0.8,
                "accepted",
                now,
                now,
            ],
        )
        .expect("memory entry");

    let service = ProjectSnapshotService::new(db.connection());
    let version = service
        .save_snapshot(SaveSnapshotInput {
            project_id: project.id.clone(),
            label: "Rich".into(),
            change_reason: "Fixture snapshot".into(),
            changed_entities: vec![
                "asset".into(),
                "research_report".into(),
                "product_understanding".into(),
                "creative_direction".into(),
                "storyboard".into(),
                "prompt_package".into(),
                "memory_entry".into(),
            ],
            created_by: "test".into(),
            is_final_candidate: false,
        })
        .expect("save snapshot");
    let snapshot = version.snapshot_json;

    assert_eq!(snapshot["format_version"], json!(1));
    assert_eq!(snapshot["assets"].as_array().expect("assets").len(), 1);
    assert_eq!(snapshot["assets"][0]["id"], asset.id);
    assert_eq!(snapshot["assets"][0]["display_name"], "Hero Image");
    assert_eq!(
        snapshot["research_reports"][0]["summary"],
        report.summary.as_str()
    );
    assert_eq!(
        snapshot["product_understandings"][0]["id"],
        understanding.id
    );
    assert_eq!(
        snapshot["product_understandings"][0]["product_name"],
        "Wool Coat"
    );
    assert_eq!(snapshot["creative_directions"][0]["id"], direction.id);
    assert_eq!(snapshot["creative_directions"][0]["title"], "Quiet Luxury");

    let storyboards = snapshot["storyboards"].as_array().expect("storyboards");
    assert_eq!(storyboards.len(), 1);
    assert_eq!(storyboards[0]["storyboard"]["id"], storyboard.id);
    assert_eq!(storyboards[0]["storyboard"]["title"], "15s Film");
    let shots = storyboards[0]["shots"].as_array().expect("shots");
    assert_eq!(shots.len(), 1);
    assert_eq!(shots[0]["id"], shot.id);
    assert_eq!(shots[0]["shot_number"], 1);
    assert_eq!(shots[0]["description"], "Model enters frame");

    assert_eq!(snapshot["prompt_packages"][0]["id"], prompt.id);
    assert_eq!(snapshot["prompt_packages"][0]["shot_id"], shot.id);
    assert_eq!(
        snapshot["prompt_packages"][0]["prompt_text"],
        "A refined fashion ad shot"
    );
    assert_eq!(snapshot["memory_entries"][0]["id"], "memory-1");
    assert_eq!(
        snapshot["memory_entries"][0]["content"],
        "Use natural movement"
    );
}

#[test]
fn rollback_restores_project_title_from_snapshot() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Brand".into(),
            description: String::new(),
        })
        .expect("brand");
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id,
            title: "Original Title".into(),
            advertising_goal: String::new(),
            duration_seconds: 15,
        })
        .expect("project");

    let service = ProjectSnapshotService::new(db.connection());
    let version = service
        .save_snapshot(SaveSnapshotInput {
            project_id: project.id.clone(),
            label: "Original".into(),
            change_reason: "Before edit".into(),
            changed_entities: vec!["project".into()],
            created_by: "test".into(),
            is_final_candidate: false,
        })
        .expect("save snapshot");

    repo.update_project_title(&project.id, "Edited Title")
        .expect("edit project");
    service
        .restore_project_version(&project.id, &version.id)
        .expect("restore");

    let restored = repo.get_project(&project.id).expect("project");
    assert_eq!(restored.title, "Original Title");
}

#[test]
fn rollback_rejects_malformed_snapshot_versions() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Brand".into(),
            description: String::new(),
        })
        .expect("brand");
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id,
            title: "Original Title".into(),
            advertising_goal: String::new(),
            duration_seconds: 15,
        })
        .expect("project");
    let service = ProjectSnapshotService::new(db.connection());
    let version = service
        .save_snapshot(SaveSnapshotInput {
            project_id: project.id.clone(),
            label: "Original".into(),
            change_reason: "Before edit".into(),
            changed_entities: vec!["project".into()],
            created_by: "test".into(),
            is_final_candidate: false,
        })
        .expect("save snapshot");

    for malformed_snapshot in [
        json!({"project": {"title": "Original Title"}}),
        json!({"format_version": 2, "project": {"title": "Original Title"}}),
        json!({"format_version": 1, "project": {}}),
        json!({"format_version": 1, "project": {"title": 7}}),
        json!({"format_version": 1, "project": {"title": "   "}}),
    ] {
        db.connection()
            .execute(
                "UPDATE project_versions SET snapshot_json = ?1 WHERE id = ?2",
                params![malformed_snapshot.to_string(), version.id.as_str()],
            )
            .expect("corrupt snapshot");
        repo.update_project_title(&project.id, "Edited Title")
            .expect("edit project");

        let error = service
            .restore_project_version(&project.id, &version.id)
            .expect_err("reject malformed snapshot");
        assert!(
            matches!(&error, JoiError::Validation(message) if message.contains("snapshot version malformed")),
            "expected snapshot malformed validation, got {error:?}"
        );
        let restored = repo.get_project(&project.id).expect("project");
        assert_eq!(restored.title, "Edited Title");
    }
}
