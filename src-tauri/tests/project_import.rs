mod common;

use common::TestApp;
use joi_agent_lib::db::Database;
use joi_agent_lib::error::JoiError;
use joi_agent_lib::project_package::{
    ProjectExportInput, ProjectImportInput, ProjectPackageService,
};
use joi_agent_lib::repositories::{BrandCreate, ProjectCreate, Repository};
use serde_json::json;

#[test]
fn imports_exported_project_package() {
    let source_app = TestApp::new();
    let source_db = migrated_database(&source_app);
    let source_repo = Repository::new(source_db.connection());
    let source_brand = source_repo
        .create_brand(BrandCreate {
            name: "Runway Lab".into(),
            description: "Editorial sportswear".into(),
        })
        .expect("source brand");
    let source_project = source_repo
        .create_project(ProjectCreate {
            brand_id: source_brand.id.clone(),
            title: "Spring Drop Film".into(),
            advertising_goal: "Drive launch awareness".into(),
            duration_seconds: 30,
        })
        .expect("source project");
    let export_dir = source_app.temp_dir.path().join("exports");
    let package = ProjectPackageService::new(
        source_db.connection(),
        source_app.temp_dir.path().join("source-assets"),
    )
    .export_project(ProjectExportInput {
        project_id: source_project.id.clone(),
        export_dir,
    })
    .expect("export project");

    let target_app = TestApp::new();
    let target_db = migrated_database(&target_app);
    let import_result = ProjectPackageService::new(
        target_db.connection(),
        target_app.temp_dir.path().join("target-assets"),
    )
    .import_project(ProjectImportInput {
        project_json_path: package.project_json_path,
    })
    .expect("import project");

    assert_ne!(import_result.project_id, source_project.id);
    let target_repo = Repository::new(target_db.connection());
    let target_project = target_repo
        .get_project(&import_result.project_id)
        .expect("target project");
    assert_eq!(target_project.title, "Spring Drop Film");
    assert_eq!(target_project.advertising_goal, "Drive launch awareness");
    assert_eq!(target_project.duration_seconds, 30);

    let target_brand = target_repo
        .get_brand(&target_project.brand_id)
        .expect("target brand");
    assert_eq!(target_brand.name, "Runway Lab");
    assert_eq!(target_brand.description, "Editorial sportswear");
}

#[test]
fn rejects_unsupported_project_package_format() {
    let app = TestApp::new();
    let db = migrated_database(&app);
    let package_path = app.temp_dir.path().join("unsupported.joi-project.json");
    std::fs::write(
        &package_path,
        serde_json::to_vec_pretty(&json!({
            "format_version": 2,
            "snapshot": {
                "brand": { "name": "Brand" },
                "project": { "title": "Project" }
            }
        }))
        .expect("serialize package"),
    )
    .expect("write package");

    let error = ProjectPackageService::new(db.connection(), app.temp_dir.path().join("assets"))
        .import_project(ProjectImportInput {
            project_json_path: package_path,
        })
        .expect_err("reject unsupported format");

    assert_package_error(error, "format_version");
}

#[test]
fn rejects_package_without_snapshot() {
    let app = TestApp::new();
    let db = migrated_database(&app);
    let package_path = app
        .temp_dir
        .path()
        .join("missing-snapshot.joi-project.json");
    std::fs::write(
        &package_path,
        serde_json::to_vec_pretty(&json!({
            "format_version": 1,
            "exported_by": "Joi Agent"
        }))
        .expect("serialize package"),
    )
    .expect("write package");

    let error = ProjectPackageService::new(db.connection(), app.temp_dir.path().join("assets"))
        .import_project(ProjectImportInput {
            project_json_path: package_path,
        })
        .expect_err("reject missing snapshot");

    assert_package_error(error, "snapshot");
}

#[test]
fn rejects_malformed_project_package_json() {
    let app = TestApp::new();
    let db = migrated_database(&app);
    let package_path = app.temp_dir.path().join("malformed.joi-project.json");
    std::fs::write(&package_path, b"{ not json").expect("write malformed package");

    let error = ProjectPackageService::new(db.connection(), app.temp_dir.path().join("assets"))
        .import_project(ProjectImportInput {
            project_json_path: package_path,
        })
        .expect_err("reject malformed JSON");

    assert_package_error(error, "malformed");
}

fn migrated_database(app: &TestApp) -> Database {
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    db
}

fn assert_package_error(error: JoiError, expected_message: &str) {
    match error {
        JoiError::Package(message) => assert!(
            message.contains(expected_message),
            "expected {message:?} to contain {expected_message:?}"
        ),
        other => panic!("expected package error, got {other:?}"),
    }
}
