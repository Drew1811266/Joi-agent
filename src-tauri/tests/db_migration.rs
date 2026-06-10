mod common;

use common::TestApp;
use joi_agent_lib::db::Database;

#[test]
fn migration_creates_full_phase1_schema() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate database");

    let tables = db.table_names().expect("table names");
    for expected in [
        "brands",
        "projects",
        "assets",
        "research_reports",
        "product_understandings",
        "creative_directions",
        "storyboards",
        "shots",
        "prompt_packages",
        "project_versions",
        "memory_entries",
    ] {
        assert!(
            tables.contains(&expected.to_string()),
            "missing table {expected}"
        );
    }
}

#[test]
fn sqlite_foreign_keys_are_enabled() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    let enabled: i64 = db
        .connection()
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .expect("foreign key pragma");
    assert_eq!(enabled, 1);
}
