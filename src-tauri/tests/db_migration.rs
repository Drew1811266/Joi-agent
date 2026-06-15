mod common;

use common::TestApp;
use joi_agent_lib::db::Database;

const NOW: &str = "2026-01-01T00:00:00Z";

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
        "agent_runs",
        "agent_run_events",
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

#[test]
fn project_brand_foreign_key_is_enforced() {
    let db = migrated_in_memory_database();

    let result = db.connection().execute(
        "INSERT INTO projects (id, brand_id, title, created_at, updated_at)
         VALUES ('project-1', 'missing-brand', 'Project', ?1, ?1)",
        [NOW],
    );

    assert!(result.is_err(), "project insert unexpectedly succeeded");
}

#[test]
fn shot_number_is_unique_per_storyboard() {
    let db = migrated_in_memory_database();
    insert_brand(&db, "brand-1");
    insert_project(&db, "project-1", "brand-1");
    insert_storyboard(&db, "storyboard-1", "project-1");
    insert_shot(&db, "shot-1", "storyboard-1", 1);

    let result = db.connection().execute(
        "INSERT INTO shots (id, storyboard_id, shot_number, created_at, updated_at)
         VALUES ('shot-2', 'storyboard-1', 1, ?1, ?1)",
        [NOW],
    );

    assert!(result.is_err(), "duplicate shot number insert succeeded");
}

#[test]
fn project_version_number_is_unique_per_project() {
    let db = migrated_in_memory_database();
    insert_brand(&db, "brand-1");
    insert_project(&db, "project-1", "brand-1");
    insert_project_version(&db, "version-1", "project-1", 1);

    let result = db.connection().execute(
        "INSERT INTO project_versions (id, project_id, version_number, snapshot_json, created_at)
         VALUES ('version-2', 'project-1', 1, '{}', ?1)",
        [NOW],
    );

    assert!(
        result.is_err(),
        "duplicate project version insert succeeded"
    );
}

#[test]
fn prompt_package_shot_must_belong_to_prompt_project() {
    let db = migrated_in_memory_database();
    insert_brand(&db, "brand-1");
    insert_project(&db, "project-1", "brand-1");
    insert_project(&db, "project-2", "brand-1");
    insert_storyboard(&db, "storyboard-1", "project-1");
    insert_shot(&db, "shot-1", "storyboard-1", 1);

    let insert_result = insert_prompt_package(&db, "prompt-1", "project-2", "shot-1");
    let insert_error = insert_result.expect_err("cross-project prompt insert succeeded");
    assert!(
        insert_error
            .to_string()
            .contains("prompt package shot must belong to project"),
        "unexpected insert error: {insert_error}"
    );

    insert_prompt_package(&db, "prompt-2", "project-1", "shot-1")
        .expect("valid prompt package insert");
    let update_result = db.connection().execute(
        "UPDATE prompt_packages SET project_id = 'project-2' WHERE id = 'prompt-2'",
        [],
    );
    let update_error = update_result.expect_err("cross-project prompt update succeeded");
    assert!(
        update_error
            .to_string()
            .contains("prompt package shot must belong to project"),
        "unexpected update error: {update_error}"
    );
}

#[test]
fn prompt_packages_allow_project_bound_image_prompts() {
    let db = migrated_in_memory_database();
    insert_brand(&db, "brand-1");
    insert_project(&db, "project-1", "brand-1");
    insert_storyboard(&db, "storyboard-1", "project-1");
    insert_shot(&db, "shot-1", "storyboard-1", 1);

    db.connection()
        .execute(
            "INSERT INTO prompt_packages (
                id, project_id, shot_id, platform, modality, prompt_text, negative_prompt,
                parameters_json, is_locked, created_at, updated_at
            ) VALUES (
                'prompt-image-1', 'project-1', NULL, 'gpt_image_2', 'image',
                'image prompt', 'negative prompt', '{}', 0, ?1, ?1
            )",
            [NOW],
        )
        .expect("project-bound image prompt insert");
}

#[test]
fn migration_creates_expected_list_and_foreign_key_indexes() {
    let db = migrated_in_memory_database();
    let indexes = index_names(&db);

    for expected in [
        "idx_projects_brand_id",
        "idx_assets_project_id",
        "idx_research_reports_project_id",
        "idx_product_understandings_project_id",
        "idx_creative_directions_project_id",
        "idx_storyboards_project_id",
        "idx_shots_storyboard_id",
        "idx_prompt_packages_project_id",
        "idx_prompt_packages_shot_id",
        "idx_project_versions_project_id",
        "idx_memory_entries_scope",
        "idx_memory_entries_brand_id",
        "idx_memory_entries_project_id",
        "idx_agent_runs_project_id",
        "idx_agent_run_events_agent_run_id",
    ] {
        assert!(
            indexes.contains(&expected.to_string()),
            "missing index {expected}"
        );
    }
}

fn migrated_in_memory_database() -> Database {
    let db = Database::open_in_memory().expect("open in-memory database");
    db.migrate().expect("migrate database");
    db
}

fn insert_brand(db: &Database, id: &str) {
    db.connection()
        .execute(
            "INSERT INTO brands (id, name, created_at, updated_at)
             VALUES (?1, 'Brand', ?2, ?2)",
            (id, NOW),
        )
        .expect("insert brand");
}

fn insert_project(db: &Database, id: &str, brand_id: &str) {
    db.connection()
        .execute(
            "INSERT INTO projects (id, brand_id, title, created_at, updated_at)
             VALUES (?1, ?2, 'Project', ?3, ?3)",
            (id, brand_id, NOW),
        )
        .expect("insert project");
}

fn insert_storyboard(db: &Database, id: &str, project_id: &str) {
    db.connection()
        .execute(
            "INSERT INTO storyboards (id, project_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?3)",
            (id, project_id, NOW),
        )
        .expect("insert storyboard");
}

fn insert_shot(db: &Database, id: &str, storyboard_id: &str, shot_number: i64) {
    db.connection()
        .execute(
            "INSERT INTO shots (id, storyboard_id, shot_number, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            (id, storyboard_id, shot_number, NOW),
        )
        .expect("insert shot");
}

fn insert_project_version(db: &Database, id: &str, project_id: &str, version_number: i64) {
    db.connection()
        .execute(
            "INSERT INTO project_versions (id, project_id, version_number, snapshot_json, created_at)
             VALUES (?1, ?2, ?3, '{}', ?4)",
            (id, project_id, version_number, NOW),
        )
        .expect("insert project version");
}

fn insert_prompt_package(
    db: &Database,
    id: &str,
    project_id: &str,
    shot_id: &str,
) -> rusqlite::Result<usize> {
    db.connection().execute(
        "INSERT INTO prompt_packages (id, project_id, shot_id, platform, modality, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'jimeng_video', 'video', ?4, ?4)",
        (id, project_id, shot_id, NOW),
    )
}

fn index_names(db: &Database) -> Vec<String> {
    let mut statement = db
        .connection()
        .prepare(
            "SELECT name FROM sqlite_master
             WHERE type = 'index' AND name NOT LIKE 'sqlite_%'
             ORDER BY name",
        )
        .expect("prepare index query");
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .expect("query indexes");
    let mut names = Vec::new();
    for row in rows {
        names.push(row.expect("index row"));
    }
    names
}
