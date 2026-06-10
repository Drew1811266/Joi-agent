mod common;

use common::TestApp;
use joi_agent_lib::db::Database;
use joi_agent_lib::repositories::{BrandCreate, ProjectCreate, Repository};

#[test]
fn creates_and_lists_brands_and_projects() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let repo = Repository::new(db.connection());

    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".to_string(),
            description: "Premium womenswear".to_string(),
        })
        .expect("create brand");
    assert_eq!(brand.name, "Atelier Joi");

    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id.clone(),
            title: "15s launch film".to_string(),
            advertising_goal: "New seasonal drop".to_string(),
            duration_seconds: 15,
        })
        .expect("create project");

    let brands = repo.list_brands().expect("list brands");
    let projects = repo.list_projects(Some(&brand.id)).expect("list projects");
    assert_eq!(brands.len(), 1);
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].id, project.id);
}

#[test]
fn rejects_empty_brand_and_project_names() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let repo = Repository::new(db.connection());

    assert!(repo
        .create_brand(BrandCreate {
            name: " ".to_string(),
            description: String::new()
        })
        .is_err());
}
