mod common;

use common::TestApp;
use joi_agent_lib::beta_workflow::assess_beta_workflow;
use joi_agent_lib::db::Database;
use joi_agent_lib::models::MemoryScope;
use joi_agent_lib::repositories::{
    BrandCreate, CreativeDirectionCreate, MemoryEntryCreate, MemoryStatusUpdate,
    ProductUnderstandingCreate, ProjectCreate, Repository,
};

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
