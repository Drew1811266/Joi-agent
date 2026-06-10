mod common;

use common::TestApp;
use joi_agent_lib::assets::{safe_join_asset_path, AssetImportInput, AssetService};
use joi_agent_lib::db::Database;
use joi_agent_lib::repositories::{BrandCreate, ProjectCreate, Repository};
use rusqlite::params;
use serde_json::json;
use std::path::Path;

const SOURCE_BYTES: &[u8] = b"fake image bytes";
const SOURCE_SHA256: &str = "43044b9f977ef333aa328b242d0e9ff0f9fed13e1c77abdd5ff12dd8edac5dd5";

fn create_project(repo: &Repository<'_>) -> String {
    let brand = repo
        .create_brand(BrandCreate {
            name: "Brand".into(),
            description: String::new(),
        })
        .expect("brand");
    repo.create_project(ProjectCreate {
        brand_id: brand.id,
        title: "Project".into(),
        advertising_goal: String::new(),
        duration_seconds: 15,
    })
    .expect("project")
    .id
}

fn write_source(app: &TestApp) -> std::path::PathBuf {
    let source = app.temp_dir.path().join("source.jpg");
    std::fs::write(&source, SOURCE_BYTES).expect("write source");
    source
}

fn assert_no_managed_files(root: &Path) {
    if !root.exists() {
        return;
    }

    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(path).expect("read managed directory") {
            let entry = entry.expect("read managed entry");
            let file_type = entry.file_type().expect("read managed file type");
            assert!(
                !file_type.is_file(),
                "unexpected managed file left behind: {}",
                entry.path().display()
            );
            if file_type.is_dir() {
                pending.push(entry.path());
            }
        }
    }
}

#[test]
fn imports_local_asset_into_project_directory() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let repo = Repository::new(db.connection());
    let project_id = create_project(&repo);
    let source = write_source(&app);
    let assets_root = app.temp_dir.path().join("managed");

    let service = AssetService::new(db.connection(), assets_root.clone());
    let asset = service
        .import_local_file(AssetImportInput {
            project_id: project_id.clone(),
            kind: "product_image".into(),
            source_path: source.clone(),
            display_name: "Coat hero".into(),
        })
        .expect("import asset");

    let destination = assets_root.join(&asset.relative_path);
    assert_eq!(asset.kind, "product_image");
    assert_eq!(asset.display_name, "Coat hero");
    assert_eq!(asset.sha256, SOURCE_SHA256);
    assert_eq!(asset.mime_type, "image/jpeg");
    assert_eq!(asset.file_size_bytes, SOURCE_BYTES.len() as i64);
    assert_eq!(asset.source_uri, source.to_string_lossy());
    assert!(asset
        .relative_path
        .starts_with(&format!("projects/{project_id}/assets/")));
    assert!(asset.relative_path.ends_with(".jpg"));
    assert_eq!(
        std::fs::read(&destination).expect("read managed asset"),
        SOURCE_BYTES
    );

    let assets = repo.list_assets(&project_id).expect("list assets");
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].id, asset.id);
    assert_eq!(assets[0].display_name, "Coat hero");
    assert_eq!(assets[0].sha256, SOURCE_SHA256);
    assert_eq!(assets[0].mime_type, "image/jpeg");
    assert_eq!(assets[0].file_size_bytes, SOURCE_BYTES.len() as i64);
    assert_eq!(assets[0].metadata_json, json!({}));
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
fn rejects_invalid_kind_without_leaving_managed_file() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let repo = Repository::new(db.connection());
    let project_id = create_project(&repo);
    let source = write_source(&app);
    let assets_root = app.temp_dir.path().join("managed");
    let service = AssetService::new(db.connection(), assets_root.clone());

    let result = service.import_local_file(AssetImportInput {
        project_id,
        kind: "bad_kind".into(),
        source_path: source,
        display_name: "Bad kind".into(),
    });

    assert!(result.is_err());
    assert_no_managed_files(&assets_root);
}

#[test]
fn rejects_missing_existing_project_without_leaving_managed_file() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let source = write_source(&app);
    let assets_root = app.temp_dir.path().join("managed");
    let service = AssetService::new(db.connection(), assets_root.clone());

    let result = service.import_local_file(AssetImportInput {
        project_id: "missing".into(),
        kind: "product_image".into(),
        source_path: source,
        display_name: "Missing project".into(),
    });

    assert!(result.is_err());
    assert_no_managed_files(&assets_root);
}

#[test]
fn rejects_unsafe_project_id_without_leaving_managed_file() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let repo = Repository::new(db.connection());
    let project_id = create_project(&repo);
    let unsafe_project_id = "nested/project";
    db.connection()
        .execute(
            "UPDATE projects SET id = ?1 WHERE id = ?2",
            params![unsafe_project_id, project_id],
        )
        .expect("make unsafe project id");
    let source = write_source(&app);
    let assets_root = app.temp_dir.path().join("managed");
    let service = AssetService::new(db.connection(), assets_root.clone());

    let result = service.import_local_file(AssetImportInput {
        project_id: unsafe_project_id.into(),
        kind: "product_image".into(),
        source_path: source,
        display_name: "Unsafe project".into(),
    });

    assert!(result.is_err());
    assert_no_managed_files(&assets_root);
}

#[test]
fn removes_copied_file_when_asset_insert_fails() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let repo = Repository::new(db.connection());
    let project_id = create_project(&repo);
    let source = write_source(&app);
    let assets_root = app.temp_dir.path().join("managed");
    db.connection()
        .execute_batch(
            "CREATE TRIGGER fail_asset_insert
             BEFORE INSERT ON assets
             BEGIN
               SELECT RAISE(ABORT, 'forced asset insert failure');
             END;",
        )
        .expect("create failing trigger");
    let service = AssetService::new(db.connection(), assets_root.clone());

    let result = service.import_local_file(AssetImportInput {
        project_id,
        kind: "product_image".into(),
        source_path: source,
        display_name: "Insert failure".into(),
    });

    assert!(result.is_err());
    assert_no_managed_files(&assets_root);
}

#[test]
fn safe_join_asset_path_rejects_path_traversal() {
    let app = TestApp::new();
    let root = app.temp_dir.path().join("managed");

    for relative_path in ["../escape.jpg", "..\\escape.jpg"] {
        let result = safe_join_asset_path(&root, relative_path);
        assert!(result.is_err(), "expected {relative_path:?} to be rejected");
    }
    assert!(!app.temp_dir.path().join("escape.jpg").exists());
}
