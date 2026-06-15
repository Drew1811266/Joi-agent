mod common;

use common::TestApp;
use joi_agent_lib::db::Database;
use joi_agent_lib::prompt_adapter::{
    generate_prompt_packages, prompt_adapter_profiles, PromptGenerationInput,
};
use joi_agent_lib::repositories::{
    BrandCreate, CreativeDirectionCreate, ProductUnderstandingCreate, ProjectCreate, Repository,
    ShotPlanCreate, StoryboardCreate,
};
use serde_json::json;

fn migrated_database() -> (TestApp, Database) {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open db");
    db.migrate().expect("migrate");
    (app, db)
}

#[test]
fn returns_expected_adapter_profiles() {
    let profiles = prompt_adapter_profiles();
    let ids = profiles
        .iter()
        .map(|profile| profile.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        ids,
        vec![
            "jimeng_video",
            "grok_video",
            "banana_2_image",
            "jimeng_image",
            "gpt_image_2"
        ]
    );
    assert_eq!(profiles[0].modality, "video");
    assert_eq!(profiles[2].modality, "image");
}

#[test]
fn generates_video_prompt_packages_for_selected_shots() {
    let (_app, db) = migrated_database();
    let repo = Repository::new(db.connection());
    let project_id = seed_prompt_project(&repo);
    let storyboard = repo
        .create_storyboard(StoryboardCreate {
            project_id: project_id.clone(),
            title: "Spring Drop storyboard".into(),
            duration_seconds: 15,
        })
        .expect("storyboard");
    let shot = repo
        .create_shot_plan(ShotPlanCreate {
            storyboard_id: storyboard.id.clone(),
            shot_number: 1,
            duration_seconds: 3,
            visual_description: "Model enters frame wearing a trench.".into(),
            model_action: "Model walks forward.".into(),
            garment_focus: "water-resistant cotton trench silhouette".into(),
            camera_movement: "slow push-in".into(),
            scene: "minimal warm studio".into(),
            lighting: "soft side light".into(),
            transition: "cut on movement".into(),
            subtitle_or_text: "Light enough for changing weather".into(),
            rationale: "Opening hook.".into(),
            source_memory_ids: Vec::new(),
            source_research_report_ids: Vec::new(),
            generation_context: json!({"stage": "0.16"}),
        })
        .expect("shot");

    let result = generate_prompt_packages(
        &repo,
        PromptGenerationInput {
            project_id: project_id.clone(),
            shot_ids: vec![shot.id.clone()],
            image_brief: String::new(),
            target_platforms: vec!["jimeng_video".into(), "grok_video".into()],
            user_direction: "Keep prompts concise.".into(),
        },
        "0.17.0".into(),
    )
    .expect("prompt generation");

    assert_eq!(result.packages.len(), 2);
    assert!(result
        .packages
        .iter()
        .all(|item| item.package.shot_id == Some(shot.id.clone())));
    assert!(result
        .packages
        .iter()
        .any(|item| item.package.platform == "jimeng_video"));
    assert!(result
        .packages
        .iter()
        .any(|item| item.package.platform == "grok_video"));
    assert!(result.packages[0]
        .package
        .prompt_text
        .contains("water-resistant cotton trench silhouette"));
    assert!(result.packages[0].missing_fields.is_empty());
    assert_eq!(result.agent_run.runtime_mode, "local_prompt_adapter_bridge");
    assert_eq!(result.agent_events.len(), 5);
}

#[test]
fn generates_project_bound_image_prompt_packages_from_image_brief() {
    let (_app, db) = migrated_database();
    let repo = Repository::new(db.connection());
    let project_id = seed_prompt_project(&repo);

    let result = generate_prompt_packages(
        &repo,
        PromptGenerationInput {
            project_id: project_id.clone(),
            shot_ids: Vec::new(),
            image_brief: "Full-body ecommerce model photo, warm studio, emphasize cotton texture."
                .into(),
            target_platforms: vec![
                "banana_2_image".into(),
                "jimeng_image".into(),
                "gpt_image_2".into(),
            ],
            user_direction: "Natural model pose.".into(),
        },
        "0.17.0".into(),
    )
    .expect("image prompts");

    assert_eq!(result.packages.len(), 3);
    assert!(result
        .packages
        .iter()
        .all(|item| item.package.shot_id.is_none()));
    assert!(result
        .packages
        .iter()
        .all(|item| item.package.modality == "image"));
    assert!(result
        .packages
        .iter()
        .all(|item| item.package.prompt_text.contains("Lightweight trench")));
    assert!(result.packages.iter().any(|item| {
        item.package.platform == "jimeng_image"
            && item.package.prompt_text.contains("服装广告模拍图")
    }));
}

#[test]
fn rejects_video_generation_without_shots_and_unknown_platforms() {
    let (_app, db) = migrated_database();
    let repo = Repository::new(db.connection());
    let project_id = seed_prompt_project(&repo);

    let missing_shot = generate_prompt_packages(
        &repo,
        PromptGenerationInput {
            project_id: project_id.clone(),
            shot_ids: Vec::new(),
            image_brief: String::new(),
            target_platforms: vec!["jimeng_video".into()],
            user_direction: String::new(),
        },
        "0.17.0".into(),
    );
    assert!(missing_shot.is_err());

    let unknown = generate_prompt_packages(
        &repo,
        PromptGenerationInput {
            project_id,
            shot_ids: Vec::new(),
            image_brief: "A studio still.".into(),
            target_platforms: vec!["unknown_platform".into()],
            user_direction: String::new(),
        },
        "0.17.0".into(),
    );
    assert!(unknown.is_err());
}

fn seed_prompt_project(repo: &Repository<'_>) -> String {
    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".into(),
            description: "Editorial womenswear with clean studio campaigns.".into(),
        })
        .expect("brand");
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id,
            title: "Spring Drop Film".into(),
            advertising_goal: "Launch awareness for lightweight trench.".into(),
            duration_seconds: 15,
        })
        .expect("project");
    repo.create_product_understanding(ProductUnderstandingCreate {
        project_id: project.id.clone(),
        product_name: "Lightweight trench".into(),
        category: "outerwear".into(),
        audience: "urban commuters".into(),
        selling_points: vec!["water-resistant cotton".into(), "soft structure".into()],
        constraints: vec!["avoid heavy winter styling".into()],
        notes: "{}".into(),
    })
    .expect("understanding");
    repo.create_creative_direction(CreativeDirectionCreate {
        project_id: project.id.clone(),
        title: "Clean studio movement".into(),
        concept: "clean studio walk with tactile close-up".into(),
        tone: "premium".into(),
        visual_style: "clean studio fashion ad".into(),
        scene_direction: "minimal warm studio".into(),
        rationale: "Matches launch goal.".into(),
    })
    .expect("direction");
    project.id
}
