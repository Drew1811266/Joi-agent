mod common;

use common::TestApp;
use joi_agent_lib::db::Database;
use joi_agent_lib::memory_curation::{curate_memory_candidates, MemoryCurationInput};
use joi_agent_lib::repositories::{
    BrandCreate, MemoryCandidateCreate, MemoryStatusUpdate, ProjectCreate, Repository,
    ResearchReportCreate,
};
use serde_json::json;

fn migrated_database() -> (TestApp, Database) {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    (app, db)
}

#[test]
fn generates_memory_candidates_from_research_report() {
    let (_app, db) = migrated_database();
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".to_string(),
            description: "Contemporary womenswear".to_string(),
        })
        .unwrap();
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id,
            title: "Spring Drop Film".to_string(),
            advertising_goal: "Launch awareness".to_string(),
            duration_seconds: 15,
        })
        .unwrap();
    let report = repo
        .create_research_report(ResearchReportCreate {
            project_id: project.id.clone(),
            summary: "Research summary".to_string(),
            findings_json: json!([
                {
                    "insight": "Fabric texture should become a proof point.",
                    "creative_implication": "Use tactile close-ups as visual proof before the model movement."
                }
            ]),
            sources_json: json!([]),
        })
        .unwrap();

    let result = curate_memory_candidates(
        &repo,
        MemoryCurationInput {
            project_id: project.id.clone(),
            feedback_text: String::new(),
            include_research_reports: true,
        },
        "0.16.0".to_string(),
    )
    .unwrap();

    assert_eq!(result.candidates.len(), 1);
    assert_eq!(
        result.candidates[0].entry.content,
        "Use tactile close-ups as visual proof before the model movement."
    );
    assert_eq!(result.candidates[0].entry.status, "proposed");
    assert_eq!(result.candidates[0].entry.source, "research report");
    assert_eq!(
        result.candidates[0].entry.source_entity_type,
        "research_report"
    );
    assert_eq!(result.candidates[0].entry.source_entity_id, report.id);
    assert_eq!(result.candidates[0].entry.confidence, 0.72);
    assert!(!result.candidates[0].has_conflict);
    assert_eq!(result.agent_run.runtime_mode, "local_memory_bridge");
    assert_eq!(result.agent_events.len(), 5);
    assert_eq!(
        result.agent_events[3].event_type,
        "memory_conflicts_checked"
    );

    let memories = repo
        .list_memory_entries("project", None, Some(project.id.as_str()))
        .unwrap();
    assert_eq!(memories.len(), 1);
}

#[test]
fn marks_duplicate_memory_candidates_as_conflicts() {
    let (_app, db) = migrated_database();
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".to_string(),
            description: "Contemporary womenswear".to_string(),
        })
        .unwrap();
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id.clone(),
            title: "Spring Drop Film".to_string(),
            advertising_goal: "Launch awareness".to_string(),
            duration_seconds: 15,
        })
        .unwrap();
    let existing = repo
        .create_memory_candidate(MemoryCandidateCreate {
            scope: "project".to_string(),
            brand_id: Some(brand.id),
            project_id: Some(project.id.clone()),
            content: "Use tactile close-ups as visual proof before the model movement.".to_string(),
            source: "research report".to_string(),
            source_entity_type: "research_report".to_string(),
            source_entity_id: "research-previous".to_string(),
            confidence: 0.72,
        })
        .unwrap();
    repo.update_memory_entry_status(MemoryStatusUpdate {
        id: existing.id.clone(),
        status: "accepted".to_string(),
    })
    .unwrap();
    repo.create_research_report(ResearchReportCreate {
        project_id: project.id.clone(),
        summary: "Research summary".to_string(),
        findings_json: json!([
            {
                "creative_implication": "Use tactile close-ups as visual proof before the model movement."
            }
        ]),
        sources_json: json!([]),
    })
    .unwrap();

    let result = curate_memory_candidates(
        &repo,
        MemoryCurationInput {
            project_id: project.id,
            feedback_text: String::new(),
            include_research_reports: true,
        },
        "0.16.0".to_string(),
    )
    .unwrap();

    assert_eq!(result.candidates.len(), 1);
    assert!(result.candidates[0].has_conflict);
    assert_eq!(result.candidates[0].conflict_memory_ids, vec![existing.id]);
}

#[test]
fn rejects_memory_curation_without_candidate_material() {
    let (_app, db) = migrated_database();
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".to_string(),
            description: String::new(),
        })
        .unwrap();
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id,
            title: "Spring Drop Film".to_string(),
            advertising_goal: String::new(),
            duration_seconds: 15,
        })
        .unwrap();

    let error = curate_memory_candidates(
        &repo,
        MemoryCurationInput {
            project_id: project.id,
            feedback_text: String::new(),
            include_research_reports: false,
        },
        "0.16.0".to_string(),
    )
    .expect_err("missing candidate material");

    assert!(error.to_string().contains("candidate material"));
}
