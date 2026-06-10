mod common;

use common::TestApp;
use joi_agent_lib::db::Database;
use joi_agent_lib::repositories::{BrandCreate, ProjectCreate, Repository};
use joi_agent_lib::snapshots::{ProjectSnapshotService, SaveSnapshotInput};

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
