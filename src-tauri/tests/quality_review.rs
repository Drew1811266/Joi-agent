mod common;

use common::TestApp;
use joi_agent_lib::db::Database;
use joi_agent_lib::quality_review::{
    apply_quality_review_suggestion, generate_quality_review, ApplyReviewSuggestionInput,
    QualityReviewGenerationInput, QualityReviewSuggestionStatus,
};
use joi_agent_lib::repositories::{
    BrandCreate, CreativeDirectionCreate, ProductUnderstandingCreate, ProjectCreate,
    PromptPackageCreate, PromptPackageUpdate, QualityReviewCreate, Repository, ShotPlanCreate,
    ShotUpdate, StoryboardCreate,
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

#[test]
fn apply_quality_review_suggestion_updates_shot_and_marks_suggestion_applied() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".to_string(),
            description: "Premium studio outerwear".to_string(),
        })
        .expect("brand");
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id,
            title: "Shot Edit Film".to_string(),
            advertising_goal: "Launch awareness".to_string(),
            duration_seconds: 15,
        })
        .expect("project");

    let storyboard = repo
        .create_storyboard(StoryboardCreate {
            project_id: project.id.clone(),
            title: "Shot edit storyboard".to_string(),
            duration_seconds: 15,
        })
        .expect("storyboard");
    let shot = repo
        .create_shot_plan(ShotPlanCreate {
            storyboard_id: storyboard.id,
            shot_number: 1,
            duration_seconds: 5,
            visual_description: "Model walks forward.".to_string(),
            model_action: "Model walks.".to_string(),
            garment_focus: "outerwear".to_string(),
            camera_movement: "push in".to_string(),
            scene: "studio".to_string(),
            lighting: "soft".to_string(),
            transition: "cut".to_string(),
            subtitle_or_text: "Light layer".to_string(),
            rationale: "Opening motion.".to_string(),
            source_memory_ids: vec![],
            source_research_report_ids: vec![],
            generation_context: json!({}),
        })
        .expect("shot");

    let suggestion_id = format!("suggest-shot-{}-description", shot.id);
    let review = repo
        .create_quality_review(QualityReviewCreate {
            project_id: project.id.clone(),
            summary: "Quality review scored 88/100 with 0 failed check(s), 1 warning(s), and 1 pending suggestion(s).".to_string(),
            score: 88,
            checklist_json: json!([]),
            suggestions_json: json!([
                {
                    "id": suggestion_id,
                    "target_type": "shot",
                    "target_id": shot.id,
                    "field": "description",
                    "current_value": "Model walks forward.",
                    "suggested_value": "Model walks forward while the outerwear silhouette stays visible.",
                    "rationale": "Make garment visibility explicit.",
                    "status": "pending",
                    "check_ids": []
                }
            ]),
        })
        .expect("review");

    let result = apply_quality_review_suggestion(
        &repo,
        ApplyReviewSuggestionInput {
            review_id: review.id,
            suggestion_id,
        },
        "0.19.0".to_string(),
    )
    .expect("applied");

    assert_eq!(result.applied_target_type, "shot");
    assert_eq!(result.suggestion.status, "applied");
    assert!(result.updated_review.suggestions_json[0]["status"]
        .as_str()
        .is_some_and(|status| status == "applied"));

    let updated_shot = repo
        .get_shot(&result.applied_target_id)
        .expect("updated shot");
    assert_eq!(
        updated_shot.description,
        "Model walks forward while the outerwear silhouette stays visible."
    );
    assert_eq!(
        result.agent_run.runtime_mode,
        "local_quality_iteration_bridge"
    );
}

#[test]
fn apply_quality_review_suggestion_updates_prompt_text() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".to_string(),
            description: "Premium studio outerwear".to_string(),
        })
        .expect("brand");
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id,
            title: "Prompt Edit Film".to_string(),
            advertising_goal: "Launch awareness".to_string(),
            duration_seconds: 15,
        })
        .expect("project");
    let prompt = repo
        .create_prompt_package(PromptPackageCreate {
            project_id: project.id.clone(),
            shot_id: None,
            platform: "gpt_image_2".to_string(),
            modality: "image".to_string(),
            prompt_text: "Create a model photo.".to_string(),
            negative_prompt: "avoid distorted garment".to_string(),
            parameters_json: json!({
                "format_version": "joi.prompt_package_parameters.v1",
                "adapter_profile_id": "gpt_image_2",
                "missing_fields": ["garment"]
            }),
        })
        .expect("prompt");

    let suggestion_id = format!("suggest-prompt-{}-missing-fields", prompt.id);
    let review = repo
        .create_quality_review(QualityReviewCreate {
            project_id: project.id,
            summary: "Quality review scored 82/100 with 1 failed check(s), 0 warning(s), and 1 pending suggestion(s).".to_string(),
            score: 82,
            checklist_json: json!([]),
            suggestions_json: json!([
                {
                    "id": suggestion_id,
                    "target_type": "prompt_package",
                    "target_id": prompt.id,
                    "field": "prompt_text",
                    "current_value": "Create a model photo.",
                    "suggested_value": "Create a model photo. Include: garment.",
                    "rationale": "Complete provider fields.",
                    "status": "pending",
                    "check_ids": []
                }
            ]),
        })
        .expect("review");

    let result = apply_quality_review_suggestion(
        &repo,
        ApplyReviewSuggestionInput {
            review_id: review.id,
            suggestion_id,
        },
        "0.19.0".to_string(),
    )
    .expect("applied");

    let updated_prompt = repo
        .get_prompt_package(&result.applied_target_id)
        .expect("updated prompt");
    assert_eq!(
        updated_prompt.prompt_text,
        "Create a model photo. Include: garment."
    );
    assert_eq!(updated_prompt.negative_prompt, "avoid distorted garment");
    assert_eq!(
        updated_prompt.parameters_json["missing_fields"],
        json!(["garment"])
    );
}

#[test]
fn apply_quality_review_suggestion_rejects_locked_and_unsupported_targets() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".to_string(),
            description: "Premium studio outerwear".to_string(),
        })
        .expect("brand");
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id,
            title: "Locked Review Targets".to_string(),
            advertising_goal: "Launch awareness".to_string(),
            duration_seconds: 15,
        })
        .expect("project");
    let storyboard = repo
        .create_storyboard(StoryboardCreate {
            project_id: project.id.clone(),
            title: "Locked shot storyboard".to_string(),
            duration_seconds: 15,
        })
        .expect("storyboard");
    let shot = repo
        .create_shot_plan(ShotPlanCreate {
            storyboard_id: storyboard.id,
            shot_number: 1,
            duration_seconds: 5,
            visual_description: "Model walks forward.".to_string(),
            model_action: "Model walks.".to_string(),
            garment_focus: "outerwear".to_string(),
            camera_movement: "push in".to_string(),
            scene: "studio".to_string(),
            lighting: "soft".to_string(),
            transition: "cut".to_string(),
            subtitle_or_text: "Light layer".to_string(),
            rationale: "Opening motion.".to_string(),
            source_memory_ids: vec![],
            source_research_report_ids: vec![],
            generation_context: json!({}),
        })
        .expect("shot");
    repo.update_shot(ShotUpdate {
        id: shot.id.clone(),
        duration_seconds: shot.duration_seconds,
        visual_description: shot.description.clone(),
        model_action: shot.model_action,
        garment_focus: "outerwear".to_string(),
        camera_movement: shot.camera_movement,
        scene: shot.scene,
        lighting: shot.lighting,
        transition: "cut".to_string(),
        subtitle_or_text: shot.subtitle_or_voiceover,
        rationale: shot.rationale,
        is_locked: true,
    })
    .expect("lock shot");

    let prompt = repo
        .create_prompt_package(PromptPackageCreate {
            project_id: project.id.clone(),
            shot_id: None,
            platform: "gpt_image_2".to_string(),
            modality: "image".to_string(),
            prompt_text: "Create a model photo.".to_string(),
            negative_prompt: "avoid distorted garment".to_string(),
            parameters_json: json!({
                "format_version": "joi.prompt_package_parameters.v1",
                "adapter_profile_id": "gpt_image_2",
                "missing_fields": ["garment"]
            }),
        })
        .expect("prompt");
    repo.update_prompt_package(PromptPackageUpdate {
        id: prompt.id.clone(),
        prompt_text: prompt.prompt_text,
        negative_prompt: prompt.negative_prompt,
        parameters_json: prompt.parameters_json,
        is_locked: true,
    })
    .expect("lock prompt");

    let locked_shot_suggestion_id = format!("suggest-shot-{}-description", shot.id);
    let locked_shot_review = repo
        .create_quality_review(QualityReviewCreate {
            project_id: project.id.clone(),
            summary: "Quality review scored 88/100 with 0 failed check(s), 1 warning(s), and 1 pending suggestion(s).".to_string(),
            score: 88,
            checklist_json: json!([]),
            suggestions_json: json!([
                {
                    "id": locked_shot_suggestion_id,
                    "target_type": "shot",
                    "target_id": shot.id,
                    "field": "description",
                    "current_value": "Model walks forward.",
                    "suggested_value": "Model walks forward while the outerwear silhouette stays visible.",
                    "rationale": "Make garment visibility explicit.",
                    "status": "pending",
                    "check_ids": []
                }
            ]),
        })
        .expect("locked shot review");
    let locked_shot_error = apply_quality_review_suggestion(
        &repo,
        ApplyReviewSuggestionInput {
            review_id: locked_shot_review.id,
            suggestion_id: locked_shot_suggestion_id,
        },
        "0.19.0".to_string(),
    )
    .expect_err("locked shot suggestion unexpectedly applied");
    assert!(locked_shot_error
        .to_string()
        .contains("Locked shots cannot be updated from review suggestions"));

    let locked_prompt_suggestion_id = format!("suggest-prompt-{}-missing-fields", prompt.id);
    let locked_prompt_review = repo
        .create_quality_review(QualityReviewCreate {
            project_id: project.id.clone(),
            summary: "Quality review scored 82/100 with 1 failed check(s), 0 warning(s), and 1 pending suggestion(s).".to_string(),
            score: 82,
            checklist_json: json!([]),
            suggestions_json: json!([
                {
                    "id": locked_prompt_suggestion_id,
                    "target_type": "prompt_package",
                    "target_id": prompt.id,
                    "field": "prompt_text",
                    "current_value": "Create a model photo.",
                    "suggested_value": "Create a model photo. Include: garment.",
                    "rationale": "Complete provider fields.",
                    "status": "pending",
                    "check_ids": []
                }
            ]),
        })
        .expect("locked prompt review");
    let locked_prompt_error = apply_quality_review_suggestion(
        &repo,
        ApplyReviewSuggestionInput {
            review_id: locked_prompt_review.id,
            suggestion_id: locked_prompt_suggestion_id,
        },
        "0.19.0".to_string(),
    )
    .expect_err("locked prompt suggestion unexpectedly applied");
    assert!(locked_prompt_error
        .to_string()
        .contains("Locked prompt packages cannot be updated from review suggestions"));

    let unsupported_suggestion_id = "suggest-asset-1-display-name".to_string();
    let unsupported_review = repo
        .create_quality_review(QualityReviewCreate {
            project_id: project.id,
            summary: "Quality review scored 94/100 with 0 failed check(s), 1 warning(s), and 1 pending suggestion(s).".to_string(),
            score: 94,
            checklist_json: json!([]),
            suggestions_json: json!([
                {
                    "id": unsupported_suggestion_id,
                    "target_type": "asset",
                    "target_id": "asset-1",
                    "field": "display_name",
                    "current_value": "Old",
                    "suggested_value": "New",
                    "rationale": "Unsupported in 0.19.",
                    "status": "pending",
                    "check_ids": []
                }
            ]),
        })
        .expect("unsupported review");
    let unsupported_error = apply_quality_review_suggestion(
        &repo,
        ApplyReviewSuggestionInput {
            review_id: unsupported_review.id,
            suggestion_id: unsupported_suggestion_id,
        },
        "0.19.0".to_string(),
    )
    .expect_err("unsupported target unexpectedly applied");
    assert!(unsupported_error
        .to_string()
        .contains("review suggestion target is not supported: asset.display_name"));
}
