mod common;

use common::TestApp;
use joi_agent_lib::db::Database;
use joi_agent_lib::repositories::{
    BrandCreate, CreativeDirectionCreate, ProductUnderstandingCreate, ProjectCreate,
    PromptPackageCreate, Repository, ResearchReportCreate, ShotCreate, StoryboardCreate,
};

fn seeded_repo<'a>(db: &'a Database) -> (Repository<'a>, String) {
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".into(),
            description: String::new(),
        })
        .expect("brand");
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id,
            title: "Structured content".into(),
            advertising_goal: String::new(),
            duration_seconds: 15,
        })
        .expect("project");
    (repo, project.id)
}

#[test]
fn stores_storyboard_shots_and_prompt_packages() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let (repo, project_id) = seeded_repo(&db);

    repo.create_research_report(ResearchReportCreate {
        project_id: project_id.clone(),
        summary: "Market summary".into(),
    })
    .expect("research");
    repo.create_product_understanding(ProductUnderstandingCreate {
        project_id: project_id.clone(),
        product_name: "Wool coat".into(),
        category: "Outerwear".into(),
    })
    .expect("understanding");
    repo.create_creative_direction(CreativeDirectionCreate {
        project_id: project_id.clone(),
        title: "Quiet luxury".into(),
        concept: "Soft movement in city light".into(),
    })
    .expect("creative");

    let storyboard = repo
        .create_storyboard(StoryboardCreate {
            project_id: project_id.clone(),
            title: "15s film".into(),
            duration_seconds: 15,
        })
        .expect("storyboard");
    let shot = repo
        .create_shot(ShotCreate {
            storyboard_id: storyboard.id.clone(),
            shot_number: 1,
            duration_seconds: 3,
            description: "Model enters frame".into(),
        })
        .expect("shot");
    repo.create_prompt_package(PromptPackageCreate {
        project_id: project_id.clone(),
        shot_id: shot.id.clone(),
        platform: "jimeng_video".into(),
        modality: "video".into(),
        prompt_text: "A refined fashion ad shot".into(),
    })
    .expect("prompt");

    assert_eq!(
        repo.list_storyboards(&project_id)
            .expect("storyboards")
            .len(),
        1
    );
    assert_eq!(repo.list_shots(&storyboard.id).expect("shots").len(), 1);
    assert_eq!(
        repo.list_prompt_packages(&project_id)
            .expect("prompts")
            .len(),
        1
    );
}

#[test]
fn rejects_prompt_platform_modality_mismatch() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let (repo, project_id) = seeded_repo(&db);
    let storyboard = repo
        .create_storyboard(StoryboardCreate {
            project_id: project_id.clone(),
            title: "15s film".into(),
            duration_seconds: 15,
        })
        .expect("storyboard");
    let shot = repo
        .create_shot(ShotCreate {
            storyboard_id: storyboard.id,
            shot_number: 1,
            duration_seconds: 3,
            description: "Model enters frame".into(),
        })
        .expect("shot");

    let result = repo.create_prompt_package(PromptPackageCreate {
        project_id,
        shot_id: shot.id,
        platform: "banana_2_image".into(),
        modality: "video".into(),
        prompt_text: "bad modality".into(),
    });
    assert!(result.is_err());
}
