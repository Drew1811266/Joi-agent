mod common;

use common::TestApp;
use joi_agent_lib::assets::{AssetImportInput, AssetService};
use joi_agent_lib::db::Database;
use joi_agent_lib::project_package::{
    slugify_project_title, ProjectExportInput, ProjectPackageService,
};
use joi_agent_lib::repositories::{BrandCreate, ProjectCreate, Repository};
use serde_json::Value;

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
fn slugifies_project_titles_stably() {
    assert_eq!(slugify_project_title("Launch Film"), "launch-film");
    assert_eq!(slugify_project_title("  Launch___Film!!! "), "launch-film");
    assert_eq!(slugify_project_title("   "), "joi-project");
}
