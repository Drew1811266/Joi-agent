mod common;

use common::TestApp;
use joi_agent_lib::db::Database;
use joi_agent_lib::error::JoiError;
use joi_agent_lib::repositories::{
    BrandCreate, CreativeDirectionCreate, ProductUnderstandingCreate, ProjectCreate,
    PromptPackageCreate, Repository, ResearchReportCreate, ShotCreate, StoryboardCreate,
};
use serde_json::json;

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

    let research = repo
        .create_research_report(ResearchReportCreate {
            project_id: project_id.clone(),
            summary: "Market summary".into(),
        })
        .expect("research");
    assert_eq!(research.project_id, project_id);
    assert_eq!(research.summary, "Market summary");
    assert_eq!(research.findings_json, json!([]));
    assert_eq!(research.sources_json, json!([]));

    let understanding = repo
        .create_product_understanding(ProductUnderstandingCreate {
            project_id: project_id.clone(),
            product_name: "Wool coat".into(),
            category: "Outerwear".into(),
        })
        .expect("understanding");
    assert_eq!(understanding.project_id, project_id);
    assert_eq!(understanding.product_name, "Wool coat");
    assert_eq!(understanding.category, "Outerwear");
    assert_eq!(understanding.audience, "");
    assert_eq!(understanding.selling_points_json, json!([]));
    assert_eq!(understanding.constraints_json, json!([]));
    assert_eq!(understanding.notes, "");

    let creative = repo
        .create_creative_direction(CreativeDirectionCreate {
            project_id: project_id.clone(),
            title: "Quiet luxury".into(),
            concept: "Soft movement in city light".into(),
        })
        .expect("creative");
    assert_eq!(creative.project_id, project_id);
    assert_eq!(creative.title, "Quiet luxury");
    assert_eq!(creative.concept, "Soft movement in city light");
    assert_eq!(creative.tone, "");
    assert_eq!(creative.visual_style, "");
    assert_eq!(creative.scene_direction, "");
    assert_eq!(creative.rationale, "");

    let storyboard = repo
        .create_storyboard(StoryboardCreate {
            project_id: project_id.clone(),
            title: "15s film".into(),
            duration_seconds: 15,
        })
        .expect("storyboard");
    let second_shot = repo
        .create_shot(ShotCreate {
            storyboard_id: storyboard.id.clone(),
            shot_number: 2,
            duration_seconds: 4,
            description: "Coat detail cutaway".into(),
        })
        .expect("second shot");
    let shot = repo
        .create_shot(ShotCreate {
            storyboard_id: storyboard.id.clone(),
            shot_number: 1,
            duration_seconds: 3,
            description: "Model enters frame".into(),
        })
        .expect("shot");
    let prompt = repo
        .create_prompt_package(PromptPackageCreate {
            project_id: project_id.clone(),
            shot_id: shot.id.clone(),
            platform: "jimeng_video".into(),
            modality: "video".into(),
            prompt_text: "A refined fashion ad shot".into(),
        })
        .expect("prompt");

    let storyboards = repo.list_storyboards(&project_id).expect("storyboards");
    assert_eq!(storyboards.len(), 1);
    assert_eq!(storyboards[0].id, storyboard.id);
    assert_eq!(storyboards[0].project_id, project_id);
    assert_eq!(storyboards[0].title, "15s film");
    assert_eq!(storyboards[0].duration_seconds, 15);

    let shots = repo.list_shots(&storyboard.id).expect("shots");
    assert_eq!(shots.len(), 2);
    assert_eq!(shots[0].id, shot.id);
    assert_eq!(shots[0].storyboard_id, storyboard.id);
    assert_eq!(shots[0].shot_number, 1);
    assert_eq!(shots[0].duration_seconds, 3);
    assert_eq!(shots[0].description, "Model enters frame");
    assert!(!shots[0].is_locked);
    assert_eq!(shots[0].metadata_json, json!({}));
    assert_eq!(shots[1].id, second_shot.id);
    assert_eq!(shots[1].shot_number, 2);

    let prompts = repo.list_prompt_packages(&project_id).expect("prompts");
    assert_eq!(prompts.len(), 1);
    assert_eq!(prompts[0].id, prompt.id);
    assert_eq!(prompts[0].project_id, project_id);
    assert_eq!(prompts[0].shot_id, shot.id);
    assert_eq!(prompts[0].platform, "jimeng_video");
    assert_eq!(prompts[0].modality, "video");
    assert_eq!(prompts[0].prompt_text, "A refined fashion ad shot");
    assert_eq!(prompts[0].negative_prompt, "");
    assert!(!prompts[0].is_locked);
    assert_eq!(prompts[0].parameters_json, json!({}));
}

#[test]
fn rejects_non_positive_shot_number() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let (repo, project_id) = seeded_repo(&db);
    let storyboard = repo
        .create_storyboard(StoryboardCreate {
            project_id,
            title: "15s film".into(),
            duration_seconds: 15,
        })
        .expect("storyboard");

    for shot_number in [0, -1] {
        let error = repo
            .create_shot(ShotCreate {
                storyboard_id: storyboard.id.clone(),
                shot_number,
                duration_seconds: 3,
                description: "Invalid shot".into(),
            })
            .expect_err("reject non-positive shot number");
        assert!(
            matches!(error, JoiError::Validation(message) if message == "Shot number must be positive")
        );
    }
}

#[test]
fn rejects_invalid_boolean_values_when_listing_content() {
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
            storyboard_id: storyboard.id.clone(),
            shot_number: 1,
            duration_seconds: 3,
            description: "Model enters frame".into(),
        })
        .expect("shot");
    let prompt = repo
        .create_prompt_package(PromptPackageCreate {
            project_id: project_id.clone(),
            shot_id: shot.id.clone(),
            platform: "jimeng_video".into(),
            modality: "video".into(),
            prompt_text: "A refined fashion ad shot".into(),
        })
        .expect("prompt");

    db.connection()
        .execute(
            "UPDATE shots SET is_locked = 2 WHERE id = ?1",
            [shot.id.as_str()],
        )
        .expect("corrupt shot boolean");
    assert!(repo.list_shots(&storyboard.id).is_err());

    db.connection()
        .execute(
            "UPDATE shots SET is_locked = 0 WHERE id = ?1",
            [shot.id.as_str()],
        )
        .expect("restore shot boolean");
    db.connection()
        .execute(
            "UPDATE prompt_packages SET is_locked = 2 WHERE id = ?1",
            [prompt.id.as_str()],
        )
        .expect("corrupt prompt boolean");
    assert!(repo.list_prompt_packages(&project_id).is_err());
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
