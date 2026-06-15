mod common;

use common::TestApp;
use joi_agent_lib::beta_workflow::{assess_beta_workflow, run_beta_workflow, BetaWorkflowRunInput};
use joi_agent_lib::db::Database;
use joi_agent_lib::models::MemoryScope;
use joi_agent_lib::repositories::{
    BrandCreate, CreativeDirectionCreate, MemoryEntryCreate, MemoryStatusUpdate,
    ProductUnderstandingCreate, ProjectCreate, Repository,
};
use joi_agent_lib::research::ResearchSourceInput;

#[test]
fn beta_status_reports_missing_and_complete_steps() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".to_string(),
            description: "Premium womenswear with soft studio light.".to_string(),
        })
        .expect("brand");
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id.clone(),
            title: "Spring Outerwear Launch".to_string(),
            advertising_goal: "Launch a 15 second ad for the spring trench collection.".to_string(),
            duration_seconds: 15,
        })
        .expect("project");

    let initial = assess_beta_workflow(&repo, &project.id).expect("initial status");
    assert!(!initial.ready);
    assert_eq!(initial.steps[0].id, "project_setup");
    assert_eq!(initial.steps[0].status, "complete");
    assert!(initial
        .steps
        .iter()
        .any(|step| step.id == "understanding" && step.status == "action_required"));

    repo.create_product_understanding(ProductUnderstandingCreate {
        project_id: project.id.clone(),
        product_name: "Spring trench collection".to_string(),
        category: "outerwear".to_string(),
        audience: "urban womenswear shoppers".to_string(),
        selling_points: vec!["water-resistant cotton".to_string()],
        constraints: vec!["keep fabric texture visible".to_string()],
        notes: "Generated beta fixture understanding.".to_string(),
    })
    .expect("understanding");
    repo.create_creative_direction(CreativeDirectionCreate {
        project_id: project.id.clone(),
        title: "Studio texture proof".to_string(),
        concept: "Premium outerwear with tactile proof.".to_string(),
        tone: "restrained".to_string(),
        visual_style: "warm studio".to_string(),
        scene_direction: "minimal studio movement".to_string(),
        rationale: "Fixture creative direction for beta readiness.".to_string(),
    })
    .expect("creative direction");
    let memory = repo
        .create_memory_entry(MemoryEntryCreate {
            scope: MemoryScope::Project.as_str().to_string(),
            brand_id: Some(brand.id.clone()),
            project_id: Some(project.id.clone()),
            content: "Always keep tactile fabric proof in the opening shot.".to_string(),
            source: "benchmark".to_string(),
        })
        .expect("memory");
    repo.update_memory_entry_status(MemoryStatusUpdate {
        id: memory.id,
        status: "accepted".to_string(),
    })
    .expect("accepted memory");

    let updated = assess_beta_workflow(&repo, &project.id).expect("updated status");
    assert!(updated.score > initial.score);
    assert!(updated
        .steps
        .iter()
        .any(|step| step.id == "understanding" && step.status == "complete"));
    assert!(updated
        .steps
        .iter()
        .any(|step| step.id == "accepted_memory" && step.status == "complete"));
}

#[test]
fn beta_run_generates_end_to_end_project_outputs() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".to_string(),
            description: "Premium womenswear, tactile fabric proof, clean warm studio lighting."
                .to_string(),
        })
        .expect("brand");
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id.clone(),
            title: "Spring Outerwear Launch".to_string(),
            advertising_goal: "Create a 15 second launch ad for a spring trench collection."
                .to_string(),
            duration_seconds: 15,
        })
        .expect("project");
    let memory = repo
        .create_memory_entry(MemoryEntryCreate {
            scope: MemoryScope::Project.as_str().to_string(),
            brand_id: Some(brand.id.clone()),
            project_id: Some(project.id.clone()),
            content: "Open with a tactile fabric proof before model movement.".to_string(),
            source: "benchmark".to_string(),
        })
        .expect("memory");
    repo.update_memory_entry_status(MemoryStatusUpdate {
        id: memory.id,
        status: "accepted".to_string(),
    })
    .expect("accepted memory");

    let result = run_beta_workflow(
        &repo,
        BetaWorkflowRunInput {
            project_id: project.id.clone(),
            user_direction:
                "Complete the beta benchmark with premium but practical fashion ad outputs."
                    .to_string(),
            image_brief:
                "Full-body ecommerce model photo, warm clean studio, visible trench texture."
                    .to_string(),
            reference_sources: vec![ResearchSourceInput {
                title: "Benchmark reference note".to_string(),
                url: "https://example.com/atelier-joi-reference".to_string(),
                source_type: "reference".to_string(),
                excerpt: "Texture close-ups and restrained studio movement support premium outerwear positioning."
                    .to_string(),
            }],
            memory_feedback:
                "Keep tactile proof and restrained styling as reusable brand preferences."
                    .to_string(),
            save_snapshot: true,
        },
        "0.20.0-test".to_string(),
    )
    .expect("beta run");

    assert!(result
        .generated_steps
        .contains(&"understanding".to_string()));
    assert!(result.generated_steps.contains(&"storyboard".to_string()));
    assert!(result
        .generated_steps
        .contains(&"video_prompts".to_string()));
    assert!(result
        .generated_steps
        .contains(&"image_prompts".to_string()));
    assert!(result
        .generated_steps
        .contains(&"quality_review".to_string()));
    assert!(result
        .generated_steps
        .contains(&"delivery_report".to_string()));
    assert!(result.snapshot_id.is_some());
    assert!(result.delivery_report_id.is_some());
    assert!(result.package_preview.is_some());
    assert!(result.status.ready);
}
