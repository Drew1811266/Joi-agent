mod common;

use common::TestApp;
use joi_agent_lib::db::Database;
use joi_agent_lib::delivery_report::{generate_delivery_report, DeliveryReportGenerationInput};
use joi_agent_lib::repositories::{
    AssetCreate, BrandCreate, CreativeDirectionCreate, ProductUnderstandingCreate, ProjectCreate,
    PromptPackageCreate, Repository, ResearchReportCreate, ShotPlanCreate, StoryboardCreate,
};
use serde_json::json;

fn migrated_database() -> (TestApp, Database) {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    (app, db)
}

#[test]
fn generates_delivery_report_from_full_project_context() {
    let (_app, db) = migrated_database();
    let repo = Repository::new(db.connection());
    let project_id = seed_full_delivery_project(&repo);

    let result = generate_delivery_report(
        &repo,
        DeliveryReportGenerationInput {
            project_id: project_id.clone(),
            user_direction: "Keep the handoff concise.".into(),
        },
        "0.18.0".into(),
    )
    .expect("report");

    assert_eq!(result.report.project_id, project_id);
    assert!(result
        .report
        .markdown
        .contains("# Launch Film Delivery Report"));
    assert!(result.report.markdown.contains("## Storyboard"));
    assert!(result.report.markdown.contains("## Prompt Packages"));
    assert!(result.report.markdown.contains("jimeng_video"));
    assert!(result.report.markdown.contains("gpt_image_2"));
    assert!(result
        .package_preview
        .delivery_report_file_name
        .ends_with("-delivery-report.md"));
    assert_eq!(
        result.agent_run.runtime_mode,
        "local_delivery_report_bridge"
    );
    assert!(result.agent_events.len() >= 4);
}

#[test]
fn generated_report_marks_missing_sections_without_failing() {
    let (_app, db) = migrated_database();
    let repo = Repository::new(db.connection());
    let project_id = seed_minimal_project(&repo);

    let result = generate_delivery_report(
        &repo,
        DeliveryReportGenerationInput {
            project_id,
            user_direction: String::new(),
        },
        "0.18.0".into(),
    )
    .expect("report with warnings");

    assert!(result
        .report
        .markdown
        .contains("No saved research report yet."));
    assert!(result
        .sections
        .iter()
        .any(|section| section.status == "missing"));
    assert!(!result.package_preview.warnings.is_empty());
}

fn seed_full_delivery_project(repo: &Repository<'_>) -> String {
    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".into(),
            description: "Editorial womenswear with clean studio campaigns.".into(),
        })
        .expect("brand");
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id,
            title: "Launch Film".into(),
            advertising_goal: "Launch a lightweight trench through a short social ad.".into(),
            duration_seconds: 15,
        })
        .expect("project");
    repo.create_asset(AssetCreate {
        project_id: project.id.clone(),
        kind: "product_image".into(),
        display_name: "Hero product reference".into(),
        relative_path: "projects/launch-film/assets/hero.jpg".into(),
        source_uri: "file:///hero.jpg".into(),
        mime_type: "image/jpeg".into(),
        file_size_bytes: 1024,
        sha256: "hero-sha".into(),
    })
    .expect("asset");
    repo.create_product_understanding(ProductUnderstandingCreate {
        project_id: project.id.clone(),
        product_name: "Lightweight trench".into(),
        category: "outerwear".into(),
        audience: "urban commuters".into(),
        selling_points: vec!["water-resistant cotton".into(), "soft structure".into()],
        constraints: vec!["avoid heavy winter styling".into()],
        notes: "Focus on fabric movement.".into(),
    })
    .expect("product understanding");
    repo.create_research_report(ResearchReportCreate {
        project_id: project.id.clone(),
        summary: "Texture and movement should lead the edit.".into(),
        findings_json: json!([
            {
                "title": "Texture proof",
                "insight": "Close fabric details support premium positioning.",
                "creative_implication": "Use tactile close-ups before model movement."
            }
        ]),
        sources_json: json!([
            {
                "index": 1,
                "title": "Reference note",
                "url": "https://example.com/reference"
            }
        ]),
    })
    .expect("research");
    repo.create_creative_direction(CreativeDirectionCreate {
        project_id: project.id.clone(),
        title: "Clean tactile motion".into(),
        concept: "Show material proof before movement.".into(),
        tone: "premium and direct".into(),
        visual_style: "minimal warm studio, tactile close-ups".into(),
        scene_direction: "studio entrance, macro insert, walking motion".into(),
        rationale: "Matches campaign goal and source material.".into(),
    })
    .expect("creative direction");
    let storyboard = repo
        .create_storyboard(StoryboardCreate {
            project_id: project.id.clone(),
            title: "15s Launch Film".into(),
            duration_seconds: 15,
        })
        .expect("storyboard");
    let shot = repo
        .create_shot_plan(ShotPlanCreate {
            storyboard_id: storyboard.id,
            shot_number: 1,
            duration_seconds: 3,
            visual_description: "Model enters frame wearing the trench.".into(),
            model_action: "Model walks forward with natural movement.".into(),
            garment_focus: "water-resistant cotton trench silhouette".into(),
            camera_movement: "slow push-in".into(),
            scene: "minimal warm studio".into(),
            lighting: "soft side light".into(),
            transition: "cut on movement".into(),
            subtitle_or_text: "Light enough for changing weather".into(),
            rationale: "Opening hook.".into(),
            source_memory_ids: Vec::new(),
            source_research_report_ids: Vec::new(),
            generation_context: json!({"stage": "0.18-test"}),
        })
        .expect("shot");
    repo.create_prompt_package(PromptPackageCreate {
        project_id: project.id.clone(),
        shot_id: Some(shot.id),
        platform: "jimeng_video".into(),
        modality: "video".into(),
        prompt_text: "A refined fashion ad shot with tactile trench movement.".into(),
        negative_prompt: String::new(),
        parameters_json: json!({"format_version": "joi.prompt_package_parameters.v1"}),
    })
    .expect("video prompt");
    repo.create_prompt_package(PromptPackageCreate {
        project_id: project.id.clone(),
        shot_id: None,
        platform: "gpt_image_2".into(),
        modality: "image".into(),
        prompt_text: "A realistic fashion campaign still for the trench.".into(),
        negative_prompt: "distorted hands".into(),
        parameters_json: json!({"format_version": "joi.prompt_package_parameters.v1"}),
    })
    .expect("image prompt");
    project.id
}

fn seed_minimal_project(repo: &Repository<'_>) -> String {
    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".into(),
            description: String::new(),
        })
        .expect("brand");
    repo.create_project(ProjectCreate {
        brand_id: brand.id,
        title: "Minimal Launch".into(),
        advertising_goal: String::new(),
        duration_seconds: 15,
    })
    .expect("project")
    .id
}
