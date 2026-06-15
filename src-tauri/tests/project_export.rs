mod common;

use std::path::Path;

use common::TestApp;
use joi_agent_lib::assets::{AssetImportInput, AssetService};
use joi_agent_lib::db::Database;
use joi_agent_lib::error::JoiError;
use joi_agent_lib::project_package::{
    slugify_project_title, ProjectExportInput, ProjectPackageService,
};
use joi_agent_lib::repositories::{
    AssetCreate, BrandCreate, DeliveryReportCreate, ProjectCreate, Repository,
};
use serde_json::{json, Value};

const SOURCE_BYTES: &[u8] = b"exported asset bytes";

#[test]
fn exports_project_json_and_assets_folder() {
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
            title: "Launch Film".into(),
            advertising_goal: "Drive new collection awareness".into(),
            duration_seconds: 15,
        })
        .expect("project");
    let source_path = app.temp_dir.path().join("source.jpg");
    std::fs::write(&source_path, SOURCE_BYTES).expect("write source asset");
    let asset_root = app.temp_dir.path().join("managed-assets");
    let asset = AssetService::new(db.connection(), asset_root.clone())
        .import_local_file(AssetImportInput {
            project_id: project.id.clone(),
            kind: "product_image".into(),
            source_path,
            display_name: "Hero look".into(),
        })
        .expect("import asset");
    let export_dir = app.temp_dir.path().join("exports");

    let result = ProjectPackageService::new(db.connection(), asset_root)
        .export_project(ProjectExportInput {
            project_id: project.id.clone(),
            export_dir: export_dir.clone(),
            delivery_report_id: None,
        })
        .expect("export project");

    assert_eq!(
        result.project_json_path,
        export_dir.join("launch-film.joi-project.json")
    );
    assert_eq!(result.assets_dir, export_dir.join("launch-film-assets"));
    assert!(result.project_json_path.is_file());
    assert!(result.assets_dir.is_dir());

    let package: Value = serde_json::from_slice(
        &std::fs::read(&result.project_json_path).expect("read project package"),
    )
    .expect("parse project package");
    assert_eq!(package["format_version"], 1);
    assert_eq!(package["exported_by"], "Joi Agent");
    assert_eq!(package["project_id"], project.id);
    assert_eq!(package["assets_folder"], "launch-film-assets");
    assert_eq!(package["snapshot"]["project"]["title"], "Launch Film");
    assert_eq!(package["snapshot"]["assets"][0]["id"], asset.id);
    assert_eq!(
        package["snapshot"]["assets"][0]["display_name"],
        "Hero look"
    );

    let exported_asset = result.assets_dir.join(format!("{}.jpg", asset.id));
    assert_eq!(
        std::fs::read(exported_asset).expect("read exported asset"),
        SOURCE_BYTES
    );
}

#[test]
fn exports_delivery_report_markdown_with_project_package() {
    let app = TestApp::new();
    let db = migrated_database(&app);
    let repo = Repository::new(db.connection());
    let project_id = create_project(&repo, "Launch Film");
    let report = repo
        .create_delivery_report(DeliveryReportCreate {
            project_id: project_id.clone(),
            title: "Launch Film Delivery Report".into(),
            markdown: "# Launch Film Delivery Report".into(),
            sections_json: json!({
                "format_version": "joi.delivery_report_sections.v1",
                "sections": []
            }),
            is_final_candidate: true,
        })
        .expect("report");
    let export_dir = app.temp_dir.path().join("exports");

    let result =
        ProjectPackageService::new(db.connection(), app.temp_dir.path().join("managed-assets"))
            .export_project(ProjectExportInput {
                project_id,
                export_dir: export_dir.clone(),
                delivery_report_id: Some(report.id.clone()),
            })
            .expect("export");

    let report_path = result.delivery_report_path.expect("report path");
    assert_eq!(
        report_path.file_name().and_then(|name| name.to_str()),
        Some("launch-film-delivery-report.md")
    );
    assert_eq!(
        std::fs::read_to_string(&report_path).expect("read report"),
        "# Launch Film Delivery Report"
    );

    let package: Value = serde_json::from_slice(
        &std::fs::read(&result.project_json_path).expect("read project package"),
    )
    .expect("parse project package");
    assert_eq!(package["delivery_report"]["id"], report.id);
    assert_eq!(
        package["delivery_report"]["markdown_file"],
        "launch-film-delivery-report.md"
    );
}

#[test]
fn rejects_delivery_report_export_for_different_project() {
    let app = TestApp::new();
    let db = migrated_database(&app);
    let repo = Repository::new(db.connection());
    let project_id = create_project(&repo, "Launch Film");
    let other_project_id = create_project(&repo, "Other Film");
    let report = repo
        .create_delivery_report(DeliveryReportCreate {
            project_id: other_project_id,
            title: "Other Film Delivery Report".into(),
            markdown: "# Other Film Delivery Report".into(),
            sections_json: json!({
                "format_version": "joi.delivery_report_sections.v1",
                "sections": []
            }),
            is_final_candidate: true,
        })
        .expect("report");
    let export_dir = app.temp_dir.path().join("exports");

    let error =
        ProjectPackageService::new(db.connection(), app.temp_dir.path().join("managed-assets"))
            .export_project(ProjectExportInput {
                project_id,
                export_dir: export_dir.clone(),
                delivery_report_id: Some(report.id),
            })
            .expect_err("reject mismatched report");

    assert_package_integrity_error(error, "does not belong");
    assert!(!export_dir.join("launch-film.joi-project.json").exists());
}

#[test]
fn slugifies_project_titles_stably() {
    assert_eq!(slugify_project_title("Launch Film"), "launch-film");
    assert_eq!(slugify_project_title("  Launch___Film!!! "), "launch-film");
    assert_eq!(slugify_project_title("   "), "joi-project");
}

#[test]
fn rejects_export_when_managed_asset_source_is_missing() {
    let app = TestApp::new();
    let db = migrated_database(&app);
    let repo = Repository::new(db.connection());
    let project_id = create_project(&repo, "Launch Film");
    repo.create_asset(AssetCreate {
        project_id: project_id.clone(),
        kind: "product_image".into(),
        display_name: "Missing asset".into(),
        relative_path: format!("projects/{project_id}/assets/missing.jpg"),
        source_uri: "missing-source.jpg".into(),
        mime_type: "image/jpeg".into(),
        file_size_bytes: 128,
        sha256: "missing-sha".into(),
    })
    .expect("create missing asset record");
    let asset_root = app.temp_dir.path().join("managed-assets");
    let export_dir = app.temp_dir.path().join("exports");

    let error = ProjectPackageService::new(db.connection(), asset_root)
        .export_project(ProjectExportInput {
            project_id,
            export_dir: export_dir.clone(),
            delivery_report_id: None,
        })
        .expect_err("reject missing managed asset source");

    assert_package_integrity_error(error, "missing");
    assert!(!export_dir.join("launch-film.joi-project.json").exists());
}

#[test]
fn rejects_export_when_package_json_already_exists() {
    let app = TestApp::new();
    let db = migrated_database(&app);
    let repo = Repository::new(db.connection());
    let project_id = create_project(&repo, "Launch Film");
    let asset_root = app.temp_dir.path().join("managed-assets");
    import_asset(&app, &db, &asset_root, &project_id);
    let export_dir = app.temp_dir.path().join("exports");
    std::fs::create_dir_all(&export_dir).expect("create export dir");
    let package_path = export_dir.join("launch-film.joi-project.json");
    std::fs::write(&package_path, b"existing package").expect("write existing package");

    let error = ProjectPackageService::new(db.connection(), asset_root)
        .export_project(ProjectExportInput {
            project_id,
            export_dir,
            delivery_report_id: None,
        })
        .expect_err("reject existing project package");

    assert_package_integrity_error(error, "already exists");
    assert_eq!(
        std::fs::read(package_path).expect("read existing package"),
        b"existing package"
    );
}

#[test]
fn rejects_export_when_target_asset_file_already_exists() {
    let app = TestApp::new();
    let db = migrated_database(&app);
    let repo = Repository::new(db.connection());
    let project_id = create_project(&repo, "Launch Film");
    let asset_root = app.temp_dir.path().join("managed-assets");
    let asset_id = import_asset(&app, &db, &asset_root, &project_id);
    let export_dir = app.temp_dir.path().join("exports");
    let assets_dir = export_dir.join("launch-film-assets");
    std::fs::create_dir_all(&assets_dir).expect("create target assets dir");
    let target_asset = assets_dir.join(format!("{asset_id}.jpg"));
    std::fs::write(&target_asset, b"old asset bytes").expect("write existing target asset");

    let error = ProjectPackageService::new(db.connection(), asset_root)
        .export_project(ProjectExportInput {
            project_id,
            export_dir: export_dir.clone(),
            delivery_report_id: None,
        })
        .expect_err("reject existing target asset");

    assert_package_integrity_error(error, "already exists");
    assert_eq!(
        std::fs::read(target_asset).expect("read existing target asset"),
        b"old asset bytes"
    );
    assert!(!export_dir.join("launch-film.joi-project.json").exists());
}

fn migrated_database(app: &TestApp) -> Database {
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    db
}

fn create_project(repo: &Repository<'_>, title: &str) -> String {
    let brand = repo
        .create_brand(BrandCreate {
            name: "Brand".into(),
            description: String::new(),
        })
        .expect("brand");
    repo.create_project(ProjectCreate {
        brand_id: brand.id,
        title: title.into(),
        advertising_goal: "Drive new collection awareness".into(),
        duration_seconds: 15,
    })
    .expect("project")
    .id
}

fn import_asset(app: &TestApp, db: &Database, asset_root: &Path, project_id: &str) -> String {
    let source_path = app.temp_dir.path().join("source.jpg");
    std::fs::write(&source_path, SOURCE_BYTES).expect("write source asset");
    AssetService::new(db.connection(), asset_root.to_path_buf())
        .import_local_file(AssetImportInput {
            project_id: project_id.to_string(),
            kind: "product_image".into(),
            source_path,
            display_name: "Hero look".into(),
        })
        .expect("import asset")
        .id
}

fn assert_package_integrity_error(error: JoiError, expected_message: &str) {
    match error {
        JoiError::FileSystem(message) | JoiError::Package(message) => {
            assert!(
                message.contains(expected_message),
                "expected {message:?} to contain {expected_message:?}"
            );
            assert!(
                message.contains("asset")
                    || message.contains("package")
                    || message.contains("export"),
                "expected {message:?} to identify package/export asset integrity"
            );
        }
        other => panic!("expected package integrity error, got {other:?}"),
    }
}
