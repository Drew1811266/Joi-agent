mod common;

use common::TestApp;
use joi_agent_lib::db::Database;
use joi_agent_lib::repositories::{
    BrandCreate, ProductUnderstandingCreate, ProjectCreate, Repository,
};
use joi_agent_lib::research::{generate_research_report, ResearchReportInput, ResearchSourceInput};

fn migrated_database() -> (TestApp, Database) {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    (app, db)
}

#[test]
fn generates_source_backed_research_report() {
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
    repo.create_product_understanding(ProductUnderstandingCreate {
        project_id: project.id.clone(),
        product_name: "Lightweight trench".to_string(),
        category: "outerwear".to_string(),
        audience: "urban commuters".to_string(),
        selling_points: vec!["water-resistant cotton".to_string()],
        constraints: vec!["avoid heavy winter styling".to_string()],
        notes: "Focus on fabric texture.".to_string(),
    })
    .unwrap();

    let result = generate_research_report(
        &repo,
        ResearchReportInput {
            project_id: project.id.clone(),
            research_goal: "Find visual references for a trench launch film".to_string(),
            market_focus: "urban commuter outerwear".to_string(),
            platform_focus: vec!["jimeng_video".to_string(), "grok_video".to_string()],
            source_materials: vec![ResearchSourceInput {
                title: "Reference campaign note".to_string(),
                url: "https://example.com/reference".to_string(),
                source_type: "reference".to_string(),
                excerpt:
                    "Close fabric texture and walking movement made the product benefit clear."
                        .to_string(),
            }],
        },
        "0.16.0".to_string(),
    )
    .unwrap();

    assert_eq!(result.report.project_id, project.id);
    assert_eq!(result.findings.len(), 1);
    assert_eq!(result.sources.len(), 1);
    assert!(result.rationale.contains("Lightweight trench"));
    assert!(result.creative_implications[0].contains("close-ups"));
    assert_eq!(result.agent_events.len(), 5);
    assert_eq!(result.agent_events[1].event_type, "sources_collected");

    let reports = repo.list_research_reports(&project.id).unwrap();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].findings_json[0]["source_index"], 1);
    assert_eq!(
        reports[0].sources_json[0]["title"],
        "Reference campaign note"
    );
}

#[test]
fn rejects_research_report_without_sources() {
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
            advertising_goal: "Launch awareness".to_string(),
            duration_seconds: 15,
        })
        .unwrap();

    let error = generate_research_report(
        &repo,
        ResearchReportInput {
            project_id: project.id,
            research_goal: "Find references".to_string(),
            market_focus: String::new(),
            platform_focus: Vec::new(),
            source_materials: Vec::new(),
        },
        "0.16.0".to_string(),
    )
    .expect_err("missing sources should fail");

    assert!(error.to_string().contains("at least one research source"));
}
