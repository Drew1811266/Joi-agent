mod common;

use common::TestApp;
use joi_agent_lib::db::Database;
use joi_agent_lib::quality_review::{
    generate_quality_review, QualityReviewGenerationInput, QualityReviewSuggestionStatus,
};
use joi_agent_lib::repositories::{
    BrandCreate, CreativeDirectionCreate, ProductUnderstandingCreate, ProjectCreate,
    PromptPackageCreate, Repository, ShotPlanCreate, StoryboardCreate,
};
use serde_json::json;

#[test]
fn generate_quality_review_detects_storyboard_prompt_and_product_issues() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".to_string(),
            description: "Contemporary premium womenswear".to_string(),
        })
        .expect("brand");
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id.clone(),
            title: "Spring Trench Film".to_string(),
            advertising_goal: "Launch a 15 second outerwear ad".to_string(),
            duration_seconds: 15,
        })
        .expect("project");

    repo.create_product_understanding(ProductUnderstandingCreate {
        project_id: project.id.clone(),
        product_name: "Lightweight Trench".to_string(),
        category: "outerwear".to_string(),
        audience: "urban commuters".to_string(),
        selling_points: vec!["water-resistant cotton".to_string()],
        constraints: vec!["avoid winter styling".to_string()],
        notes: "Focus on material proof.".to_string(),
    })
    .expect("understanding");
    repo.create_creative_direction(CreativeDirectionCreate {
        project_id: project.id.clone(),
        title: "Clean Studio".to_string(),
        concept: "studio walk with fabric inserts".to_string(),
        tone: "premium".to_string(),
        visual_style: "clean warm studio".to_string(),
        scene_direction: "warm studio".to_string(),
        rationale: "Matches brand setup.".to_string(),
    })
    .expect("direction");

    let storyboard = repo
        .create_storyboard(StoryboardCreate {
            project_id: project.id.clone(),
            title: "Repeated storyboard".to_string(),
            duration_seconds: 15,
        })
        .expect("storyboard");

    for shot_number in 1..=2 {
        repo.create_shot_plan(ShotPlanCreate {
            storyboard_id: storyboard.id.clone(),
            shot_number,
            duration_seconds: 9,
            visual_description: "Model walks forward in a studio.".to_string(),
            model_action: "Model walks forward.".to_string(),
            garment_focus: "movement".to_string(),
            camera_movement: "slow push-in".to_string(),
            scene: "studio".to_string(),
            lighting: "soft light".to_string(),
            transition: "cut".to_string(),
            subtitle_or_text: "New season energy".to_string(),
            rationale: "Creates opening motion.".to_string(),
            source_memory_ids: vec![],
            source_research_report_ids: vec![],
            generation_context: json!({}),
        })
        .expect("shot");
    }

    repo.create_prompt_package(PromptPackageCreate {
        project_id: project.id.clone(),
        shot_id: None,
        platform: "gpt_image_2".to_string(),
        modality: "image".to_string(),
        prompt_text: "Create a model photo.".to_string(),
        negative_prompt: "avoid distorted garment".to_string(),
        parameters_json: json!({
            "format_version": "joi.prompt_package_parameters.v1",
            "adapter_profile_id": "gpt_image_2",
            "adapter_display_name": "GPT Image 2",
            "required_fields": ["subject", "scene", "garment", "material", "lighting", "style"],
            "missing_fields": ["garment", "material", "lighting", "style"]
        }),
    })
    .expect("prompt");

    let result = generate_quality_review(
        &repo,
        QualityReviewGenerationInput {
            project_id: project.id.clone(),
            user_direction: "Review before delivery.".to_string(),
        },
        "0.19.0".to_string(),
    )
    .expect("quality review");

    assert_eq!(result.review.project_id, project.id);
    assert!(result.review.score < 100);
    assert!(result
        .checks
        .iter()
        .any(|check| check.category == "storyboard_duration"));
    assert!(result
        .checks
        .iter()
        .any(|check| check.category == "shot_repetition"));
    assert!(result
        .checks
        .iter()
        .any(|check| check.category == "garment_visibility"));
    assert!(result
        .checks
        .iter()
        .any(|check| check.category == "prompt_completeness"));
    assert!(result
        .suggestions
        .iter()
        .any(|suggestion| suggestion.target_type == "shot" && suggestion.field == "description"));
    assert!(result.suggestions.iter().any(|suggestion| {
        suggestion.target_type == "prompt_package" && suggestion.field == "prompt_text"
    }));
    assert!(result.suggestions.iter().all(|suggestion| {
        suggestion.status == QualityReviewSuggestionStatus::Pending.as_str()
    }));
    assert_eq!(result.agent_run.runtime_mode, "local_quality_review_bridge");
    assert!(!result.agent_events.is_empty());
    assert_eq!(brand.id, project.brand_id);
}
