mod common;

use common::TestApp;
use joi_agent_lib::db::Database;
use joi_agent_lib::error::JoiError;
use joi_agent_lib::repositories::{BrandCreate, MemoryEntryCreate, ProjectCreate, Repository};

fn open_repo(app: &TestApp) -> Database {
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    db
}

#[test]
fn creates_and_lists_brand_scoped_memory() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Joi Brand".into(),
            description: String::new(),
        })
        .expect("brand");

    let memory = repo
        .create_memory_entry(MemoryEntryCreate {
            scope: "brand".into(),
            brand_id: Some(brand.id.clone()),
            project_id: None,
            content: "  Prefer clean studio lighting  ".into(),
            source: "user note".into(),
        })
        .expect("memory");

    assert_eq!(memory.scope, "brand");
    assert_eq!(memory.brand_id.as_deref(), Some(brand.id.as_str()));
    assert_eq!(memory.project_id, None);
    assert_eq!(memory.content, "Prefer clean studio lighting");
    assert_eq!(memory.source, "user note");
    assert_eq!(memory.status, "proposed");
    assert_eq!(memory.confidence, 0.0);

    let memories = repo
        .list_memory_entries("brand", Some(brand.id.as_str()), None)
        .expect("list memories");

    assert_eq!(memories.len(), 1);
    assert_eq!(memories[0].id, memory.id);
}

#[test]
fn rejects_brand_memory_without_brand_id() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());

    let error = repo
        .create_memory_entry(MemoryEntryCreate {
            scope: "brand".into(),
            brand_id: None,
            project_id: None,
            content: "Use warmer styling".into(),
            source: "user note".into(),
        })
        .expect_err("reject missing brand id");

    assert!(
        matches!(&error, JoiError::Validation(message) if message.contains("brand_id")),
        "expected brand_id validation, got {error:?}"
    );
}

#[test]
fn rejects_project_memory_without_project_id() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());

    let error = repo
        .create_memory_entry(MemoryEntryCreate {
            scope: "project".into(),
            brand_id: None,
            project_id: None,
            content: "Keep model movement minimal".into(),
            source: "storyboard review".into(),
        })
        .expect_err("reject missing project id");

    assert!(
        matches!(&error, JoiError::Validation(message) if message.contains("project_id")),
        "expected project_id validation, got {error:?}"
    );
}

#[test]
fn rejects_blank_memory_content() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Joi Brand".into(),
            description: String::new(),
        })
        .expect("brand");

    let error = repo
        .create_memory_entry(MemoryEntryCreate {
            scope: "brand".into(),
            brand_id: Some(brand.id),
            project_id: None,
            content: "   ".into(),
            source: "user note".into(),
        })
        .expect_err("reject blank content");

    assert!(
        matches!(&error, JoiError::Validation(message) if message.contains("Memory content")),
        "expected content validation, got {error:?}"
    );
}

#[test]
fn rejects_user_memory_with_brand_or_project_ids() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Joi Brand".into(),
            description: String::new(),
        })
        .expect("brand");

    let error = repo
        .create_memory_entry(MemoryEntryCreate {
            scope: "user".into(),
            brand_id: Some(brand.id),
            project_id: None,
            content: "I prefer concise reports".into(),
            source: "user profile".into(),
        })
        .expect_err("reject user memory with brand id");

    assert!(
        matches!(&error, JoiError::Validation(message) if message.contains("user memory")),
        "expected user memory validation, got {error:?}"
    );
}

#[test]
fn filters_memory_entries_by_scope_brand_and_project() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());
    let brand_a = repo
        .create_brand(BrandCreate {
            name: "Brand A".into(),
            description: String::new(),
        })
        .expect("brand a");
    let brand_b = repo
        .create_brand(BrandCreate {
            name: "Brand B".into(),
            description: String::new(),
        })
        .expect("brand b");
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand_a.id.clone(),
            title: "Campaign".into(),
            advertising_goal: String::new(),
            duration_seconds: 15,
        })
        .expect("project");

    let brand_memory = repo
        .create_memory_entry(MemoryEntryCreate {
            scope: "brand".into(),
            brand_id: Some(brand_a.id.clone()),
            project_id: None,
            content: "Brand A prefers editorial framing".into(),
            source: "brand brief".into(),
        })
        .expect("brand memory");
    repo.create_memory_entry(MemoryEntryCreate {
        scope: "brand".into(),
        brand_id: Some(brand_b.id.clone()),
        project_id: None,
        content: "Brand B prefers retail framing".into(),
        source: "brand brief".into(),
    })
    .expect("other brand memory");
    let project_memory = repo
        .create_memory_entry(MemoryEntryCreate {
            scope: "project".into(),
            brand_id: None,
            project_id: Some(project.id.clone()),
            content: "This campaign needs a sharper opening shot".into(),
            source: "project review".into(),
        })
        .expect("project memory");

    let brand_memories = repo
        .list_memory_entries("brand", Some(brand_a.id.as_str()), None)
        .expect("list brand memories");
    let project_memories = repo
        .list_memory_entries("project", None, Some(project.id.as_str()))
        .expect("list project memories");

    assert_eq!(brand_memories.len(), 1);
    assert_eq!(brand_memories[0].id, brand_memory.id);
    assert_eq!(project_memories.len(), 1);
    assert_eq!(project_memories[0].id, project_memory.id);
}
