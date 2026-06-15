mod common;

use common::TestApp;
use joi_agent_lib::agent_runtime::{start_agent_plan, AgentPlanInput};
use joi_agent_lib::db::Database;
use joi_agent_lib::hermes_bridge::{inspect_hermes_runtime, HermesRuntimeConfig};
use joi_agent_lib::repositories::{
    AgentRunCreate, AgentRunEventCreate, BrandCreate, CreativeDirectionCreate, MemoryEntryCreate,
    ProductUnderstandingCreate, ProjectCreate, Repository,
};
use serde_json::json;

fn migrated_database() -> (TestApp, Database) {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    (app, db)
}

#[test]
fn stores_agent_runs_and_ordered_events() {
    let (_app, db) = migrated_database();
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".to_string(),
            description: "Editorial womenswear".to_string(),
        })
        .expect("brand");
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id,
            title: "Spring Drop Film".to_string(),
            advertising_goal: "Launch awareness".to_string(),
            duration_seconds: 15,
        })
        .expect("project");

    let run = repo
        .create_agent_run(AgentRunCreate {
            project_id: project.id.clone(),
            user_goal: "Plan the next content workflow steps".to_string(),
            status: "completed".to_string(),
            runtime_kind: "hermes_core".to_string(),
            runtime_mode: "local_planner_bridge".to_string(),
            runtime_version: "0.16.0".to_string(),
            roles_json: json!(["planner", "researcher"]),
            plan_json: json!([{"role": "planner", "task": "Read context"}]),
            result_summary: "Created a two-role plan.".to_string(),
        })
        .expect("agent run");
    repo.create_agent_run_event(AgentRunEventCreate {
        agent_run_id: run.id.clone(),
        sequence_number: 2,
        role: "researcher".to_string(),
        event_type: "task_queued".to_string(),
        message: "Prepare research questions.".to_string(),
        payload_json: json!({"stage": "0.14"}),
    })
    .expect("event 2");
    repo.create_agent_run_event(AgentRunEventCreate {
        agent_run_id: run.id.clone(),
        sequence_number: 1,
        role: "planner".to_string(),
        event_type: "context_read".to_string(),
        message: "Read saved project context.".to_string(),
        payload_json: json!({"project_title": "Spring Drop Film"}),
    })
    .expect("event 1");

    let fetched = repo.get_agent_run(&run.id).expect("get agent run");
    let runs = repo.list_agent_runs(&project.id).expect("list runs");
    let events = repo.list_agent_run_events(&run.id).expect("list events");

    assert_eq!(fetched.user_goal, "Plan the next content workflow steps");
    assert_eq!(runs.len(), 1);
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].sequence_number, 1);
    assert_eq!(events[1].sequence_number, 2);
    assert_eq!(events[0].payload_json["project_title"], "Spring Drop Film");
}

#[test]
fn reports_ready_hermes_runtime_from_checkout_fixture() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let checkout_path = temp_dir.path().join("hermes-agent");
    std::fs::create_dir_all(&checkout_path).expect("checkout dir");
    std::fs::write(
        checkout_path.join("pyproject.toml"),
        "[project]\nname = \"hermes-agent\"\nversion = \"0.16.0\"\n",
    )
    .expect("pyproject");
    let phase0_report_path = temp_dir.path().join("hermes-phase0-report.md");
    std::fs::write(&phase0_report_path, "status: pass").expect("phase0 report");

    let status = inspect_hermes_runtime(HermesRuntimeConfig {
        checkout_path,
        phase0_report_path,
        runtime_mode: "local_planner_bridge".to_string(),
    });

    assert!(status.hermes_present);
    assert!(status.phase0_report_present);
    assert!(status.ready);
    assert_eq!(status.hermes_version, "0.16.0");
    assert_eq!(status.runtime_kind, "hermes_core");
    assert_eq!(status.runtime_mode, "local_planner_bridge");
}

#[test]
fn reports_not_ready_when_hermes_checkout_is_missing() {
    let temp_dir = tempfile::tempdir().expect("temp dir");

    let status = inspect_hermes_runtime(HermesRuntimeConfig {
        checkout_path: temp_dir.path().join("missing-hermes"),
        phase0_report_path: temp_dir.path().join("missing-report.md"),
        runtime_mode: "local_planner_bridge".to_string(),
    });

    assert!(!status.hermes_present);
    assert!(!status.phase0_report_present);
    assert!(!status.ready);
    assert_eq!(status.hermes_version, "");
    assert!(status
        .message
        .contains("Hermes Core checkout was not found"));
}

#[test]
fn starts_agent_plan_from_saved_project_context() {
    let (_app, db) = migrated_database();
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".to_string(),
            description: "Premium womenswear with clean studio lighting".to_string(),
        })
        .expect("brand");
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id.clone(),
            title: "Spring Drop Film".to_string(),
            advertising_goal: "Launch awareness".to_string(),
            duration_seconds: 15,
        })
        .expect("project");
    repo.create_product_understanding(ProductUnderstandingCreate {
        project_id: project.id.clone(),
        product_name: "Lightweight trench".to_string(),
        category: "outerwear".to_string(),
        audience: "urban commuters".to_string(),
        selling_points: vec![
            "water-resistant cotton".to_string(),
            "soft structure".to_string(),
        ],
        constraints: vec!["avoid heavy winter styling".to_string()],
        notes: "Use close fabric texture shots.".to_string(),
    })
    .expect("understanding");
    repo.create_creative_direction(CreativeDirectionCreate {
        project_id: project.id.clone(),
        title: "Clean studio walk".to_string(),
        concept: "Model walk with close fabric texture inserts".to_string(),
        tone: "refined".to_string(),
        visual_style: "soft daylight studio".to_string(),
        scene_direction: "seamless studio".to_string(),
        rationale: "Matches launch awareness goal.".to_string(),
    })
    .expect("creative direction");
    repo.create_memory_entry(MemoryEntryCreate {
        scope: "project".to_string(),
        brand_id: Some(brand.id),
        project_id: Some(project.id.clone()),
        content: "Keep product fabric texture visible".to_string(),
        source: "user note".to_string(),
    })
    .expect("memory");

    let result = start_agent_plan(
        &repo,
        AgentPlanInput {
            project_id: project.id.clone(),
            user_goal: "Plan the next content workflow steps".to_string(),
        },
        "0.16.0".to_string(),
    )
    .expect("start plan");

    assert_eq!(result.run.project_id, project.id);
    assert_eq!(result.run.status, "completed");
    assert_eq!(result.run.runtime_kind, "hermes_core");
    assert_eq!(result.run.runtime_mode, "local_planner_bridge");
    assert_eq!(result.run.roles_json.as_array().expect("roles").len(), 6);
    assert_eq!(result.run.plan_json.as_array().expect("plan").len(), 6);
    assert_eq!(result.events.len(), 7);
    assert_eq!(result.events[0].event_type, "context_read");
    assert!(result.events[0].message.contains("Spring Drop Film"));
    assert!(result.run.result_summary.contains("Lightweight trench"));

    let persisted_events = repo
        .list_agent_run_events(&result.run.id)
        .expect("persisted events");
    assert_eq!(persisted_events.len(), 7);
    assert_eq!(persisted_events[6].role, "memory_curator");
}
