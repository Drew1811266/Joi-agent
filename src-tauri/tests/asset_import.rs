mod common;

use common::TestApp;
use joi_agent_lib::assets::{safe_join_asset_path, AssetImportInput, AssetService};
use joi_agent_lib::db::Database;
use joi_agent_lib::repositories::{BrandCreate, ProjectCreate, Repository};

#[test]
fn imports_local_asset_into_project_directory() {
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
            title: "Project".into(),
            advertising_goal: String::new(),
            duration_seconds: 15,
        })
        .expect("project");

    let source = app.temp_dir.path().join("source.jpg");
    std::fs::write(&source, b"fake image bytes").expect("write source");
    let assets_root = app.temp_dir.path().join("managed");

    let service = AssetService::new(db.connection(), assets_root.clone());
    let asset = service
        .import_local_file(AssetImportInput {
            project_id: project.id.clone(),
            kind: "product_image".into(),
            source_path: source.clone(),
            display_name: "Coat hero".into(),
        })
        .expect("import asset");

    assert_eq!(asset.kind, "product_image");
    assert_eq!(asset.sha256.len(), 64);
    assert!(assets_root.join(&asset.relative_path).exists());
    assert_eq!(repo.list_assets(&project.id).expect("list assets").len(), 1);
}

#[test]
fn rejects_missing_source_file() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let service = AssetService::new(db.connection(), app.temp_dir.path().join("managed"));

    let result = service.import_local_file(AssetImportInput {
        project_id: "missing".into(),
        kind: "product_image".into(),
        source_path: app.temp_dir.path().join("missing.jpg"),
        display_name: "Missing".into(),
    });

    assert!(result.is_err());
}

#[test]
fn safe_join_asset_path_rejects_path_traversal() {
    let app = TestApp::new();
    let root = app.temp_dir.path().join("managed");

    let result = safe_join_asset_path(&root, "../escape.jpg");

    assert!(result.is_err());
    assert!(!app.temp_dir.path().join("escape.jpg").exists());
}
