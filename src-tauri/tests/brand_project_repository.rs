mod common;

use common::TestApp;
use joi_agent_lib::db::Database;
use joi_agent_lib::error::JoiError;
use joi_agent_lib::repositories::{BrandCreate, ProjectCreate, Repository};

#[test]
fn creates_gets_and_lists_brands_and_projects() {
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

    let second_brand = repo
        .create_brand(BrandCreate {
            name: "Joi Sport".to_string(),
            description: "Performance line".to_string(),
        })
        .expect("create second brand");
    let second_project = repo
        .create_project(ProjectCreate {
            brand_id: second_brand.id.clone(),
            title: "6s product cutdown".to_string(),
            advertising_goal: "Retargeting".to_string(),
            duration_seconds: 6,
        })
        .expect("create second project");

    let fetched_brand = repo.get_brand(&brand.id).expect("get brand");
    let fetched_project = repo.get_project(&project.id).expect("get project");
    let brands = repo.list_brands().expect("list brands");
    let projects = repo.list_projects(Some(&brand.id)).expect("list projects");
    let all_projects = repo.list_projects(None).expect("list all projects");
    let mut project_ids = all_projects
        .iter()
        .map(|project| project.id.as_str())
        .collect::<Vec<_>>();
    let mut expected_project_ids = vec![project.id.as_str(), second_project.id.as_str()];
    project_ids.sort_unstable();
    expected_project_ids.sort_unstable();

    assert_eq!(fetched_brand.id, brand.id);
    assert_eq!(fetched_project.id, project.id);
    assert_eq!(brands.len(), 2);
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].id, project.id);
    assert_eq!(project_ids, expected_project_ids);
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

    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".to_string(),
            description: String::new(),
        })
        .expect("create brand");
    assert!(repo
        .create_project(ProjectCreate {
            brand_id: brand.id,
            title: " ".to_string(),
            advertising_goal: String::new(),
            duration_seconds: 15,
        })
        .is_err());
}

#[test]
fn rejects_invalid_project_duration_and_unknown_brand() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let repo = Repository::new(db.connection());

    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".to_string(),
            description: String::new(),
        })
        .expect("create brand");

    let duration_error = repo
        .create_project(ProjectCreate {
            brand_id: brand.id,
            title: "15s launch film".to_string(),
            advertising_goal: String::new(),
            duration_seconds: -1,
        })
        .expect_err("reject negative duration");
    assert!(
        matches!(duration_error, JoiError::Validation(message) if message == "Project duration must be non-negative")
    );

    let missing_brand_error = repo
        .create_project(ProjectCreate {
            brand_id: "missing-brand".to_string(),
            title: "15s launch film".to_string(),
            advertising_goal: String::new(),
            duration_seconds: 15,
        })
        .expect_err("reject unknown brand");
    assert!(
        matches!(missing_brand_error, JoiError::NotFound(message) if message == "brand missing-brand")
    );
}

#[test]
fn reports_conversion_errors_with_source_column_index() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let repo = Repository::new(db.connection());

    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".to_string(),
            description: String::new(),
        })
        .expect("create brand");

    db.connection()
        .execute(
            "UPDATE brands SET style_keywords_json = ?1 WHERE id = ?2",
            ["not-json", brand.id.as_str()],
        )
        .expect("corrupt brand json");
    let json_error = repo.list_brands().expect_err("reject corrupt json");
    assert_error_mentions_column(json_error, 3);

    db.connection()
        .execute(
            "UPDATE brands SET style_keywords_json = ?1, created_at = ?2 WHERE id = ?3",
            ["[]", "not-a-timestamp", brand.id.as_str()],
        )
        .expect("corrupt brand timestamp");
    let time_error = repo.list_brands().expect_err("reject corrupt timestamp");
    assert_error_mentions_column(time_error, 9);
}

fn assert_error_mentions_column(error: JoiError, column_index: usize) {
    let message = error.to_string();
    let expected = format!("index: {}", column_index);
    assert!(
        message.contains(&expected),
        "expected {message:?} to contain {expected:?}"
    );
}
