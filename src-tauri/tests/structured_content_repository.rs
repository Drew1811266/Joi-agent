mod common;

use common::TestApp;
use joi_agent_lib::db::Database;
use joi_agent_lib::error::JoiError;
use joi_agent_lib::repositories::{
    BrandCreate, CreativeDirectionCreate, ProductUnderstandingCreate, ProjectCreate,
    PromptPackageCreate, Repository, ResearchReportCreate, ShotCreate, ShotPlanCreate, ShotUpdate,
    StoryboardCreate,
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
fn creates_product_understanding_with_full_material_context() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
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
            advertising_goal: "Launch a 15 second outerwear ad".to_string(),
            duration_seconds: 15,
        })
        .unwrap();

    let understanding = repo
        .create_product_understanding(ProductUnderstandingCreate {
            project_id: project.id,
            product_name: "Lightweight trench".to_string(),
            category: "outerwear".to_string(),
            audience: "urban commuters".to_string(),
            selling_points: vec![
                "water-resistant cotton".to_string(),
                "soft structure".to_string(),
            ],
            constraints: vec!["avoid heavy winter styling".to_string()],
            notes: "{\"format_version\":\"joi.product_understanding_notes.v1\"}".to_string(),
        })
        .unwrap();

    assert_eq!(understanding.product_name, "Lightweight trench");
    assert_eq!(understanding.category, "outerwear");
    assert_eq!(understanding.audience, "urban commuters");
    assert_eq!(
        understanding.selling_points_json,
        json!(["water-resistant cotton", "soft structure"])
    );
    assert_eq!(
        understanding.constraints_json,
        json!(["avoid heavy winter styling"])
    );
    assert!(understanding
        .notes
        .contains("joi.product_understanding_notes.v1"));
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
            findings_json: json!([]),
            sources_json: json!([]),
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
            audience: String::new(),
            selling_points: Vec::new(),
            constraints: Vec::new(),
            notes: String::new(),
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
            tone: String::new(),
            visual_style: String::new(),
            scene_direction: String::new(),
            rationale: String::new(),
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
fn creates_shot_plan_with_visible_storyboard_fields() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let (repo, project_id) = seeded_repo(&db);
    let storyboard = repo
        .create_storyboard(StoryboardCreate {
            project_id,
            title: "15s spring launch film".into(),
            duration_seconds: 15,
        })
        .expect("storyboard");

    let shot = repo
        .create_shot_plan(ShotPlanCreate {
            storyboard_id: storyboard.id,
            shot_number: 1,
            duration_seconds: 3,
            visual_description: "Model enters a clean studio frame wearing the trench.".into(),
            model_action: "Model walks forward and turns slightly toward camera.".into(),
            garment_focus: "relaxed trench silhouette and water-resistant cotton".into(),
            camera_movement: "slow push-in".into(),
            scene: "minimal warm studio".into(),
            lighting: "soft side light".into(),
            transition: "cut on movement".into(),
            subtitle_or_text: "Light enough for changing weather".into(),
            rationale: "Opening shot establishes product and brand mood.".into(),
            source_memory_ids: vec!["memory-1".into()],
            source_research_report_ids: vec!["report-1".into()],
            generation_context: json!({
                "stage": "0.16",
                "source": "storyboard_generation",
                "selling_point": "water-resistant cotton"
            }),
        })
        .expect("shot plan");

    assert_eq!(
        shot.description,
        "Model enters a clean studio frame wearing the trench."
    );
    assert_eq!(
        shot.model_action,
        "Model walks forward and turns slightly toward camera."
    );
    assert_eq!(shot.camera_movement, "slow push-in");
    assert_eq!(shot.scene, "minimal warm studio");
    assert_eq!(shot.lighting, "soft side light");
    assert_eq!(
        shot.subtitle_or_voiceover,
        "Light enough for changing weather"
    );
    assert_eq!(
        shot.rationale,
        "Opening shot establishes product and brand mood."
    );
    assert_eq!(shot.metadata_json["format_version"], "joi.shot_metadata.v1");
    assert_eq!(
        shot.metadata_json["garment_focus"],
        "relaxed trench silhouette and water-resistant cotton"
    );
    assert_eq!(shot.metadata_json["transition"], "cut on movement");
    assert_eq!(shot.metadata_json["source_memory_ids"], json!(["memory-1"]));
    assert_eq!(
        shot.metadata_json["source_research_report_ids"],
        json!(["report-1"])
    );
}

#[test]
fn updates_shot_details_and_preserves_source_metadata() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let (repo, project_id) = seeded_repo(&db);
    let storyboard = repo
        .create_storyboard(StoryboardCreate {
            project_id,
            title: "15s spring launch film".into(),
            duration_seconds: 15,
        })
        .expect("storyboard");
    let shot = repo
        .create_shot_plan(ShotPlanCreate {
            storyboard_id: storyboard.id,
            shot_number: 1,
            duration_seconds: 3,
            visual_description: "Original description".into(),
            model_action: "Original action".into(),
            garment_focus: "Original garment focus".into(),
            camera_movement: "Original camera".into(),
            scene: "Original scene".into(),
            lighting: "Original lighting".into(),
            transition: "Original transition".into(),
            subtitle_or_text: "Original text".into(),
            rationale: "Original rationale".into(),
            source_memory_ids: vec!["memory-1".into()],
            source_research_report_ids: vec!["report-1".into()],
            generation_context: json!({
                "stage": "0.16",
                "source": "storyboard_generation"
            }),
        })
        .expect("shot");

    let updated = repo
        .update_shot(ShotUpdate {
            id: shot.id,
            duration_seconds: 4,
            visual_description: "Close texture detail fills the frame.".into(),
            model_action: "Model lifts sleeve edge to reveal fabric movement.".into(),
            garment_focus: "fabric texture and sleeve construction".into(),
            camera_movement: "macro slide".into(),
            scene: "studio detail insert".into(),
            lighting: "grazing highlight".into(),
            transition: "match cut to walking shot".into(),
            subtitle_or_text: "Texture that moves".into(),
            rationale: "Edited to make product proof more specific.".into(),
            is_locked: true,
        })
        .expect("update shot");

    assert_eq!(updated.duration_seconds, 4);
    assert_eq!(updated.description, "Close texture detail fills the frame.");
    assert_eq!(
        updated.model_action,
        "Model lifts sleeve edge to reveal fabric movement."
    );
    assert!(updated.is_locked);
    assert_eq!(
        updated.metadata_json["garment_focus"],
        "fabric texture and sleeve construction"
    );
    assert_eq!(
        updated.metadata_json["transition"],
        "match cut to walking shot"
    );
    assert_eq!(
        updated.metadata_json["source_memory_ids"],
        json!(["memory-1"])
    );
    assert_eq!(
        updated.metadata_json["source_research_report_ids"],
        json!(["report-1"])
    );
}

#[test]
fn lists_storyboards_with_typed_shots_for_project() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let (repo, project_id) = seeded_repo(&db);
    let storyboard = repo
        .create_storyboard(StoryboardCreate {
            project_id: project_id.clone(),
            title: "15s spring launch film".into(),
            duration_seconds: 15,
        })
        .expect("storyboard");
    repo.create_shot_plan(ShotPlanCreate {
        storyboard_id: storyboard.id.clone(),
        shot_number: 1,
        duration_seconds: 3,
        visual_description: "Opening product entrance.".into(),
        model_action: "Model enters frame.".into(),
        garment_focus: "outerwear silhouette".into(),
        camera_movement: "push in".into(),
        scene: "studio".into(),
        lighting: "soft".into(),
        transition: "cut".into(),
        subtitle_or_text: String::new(),
        rationale: "Establish the product.".into(),
        source_memory_ids: Vec::new(),
        source_research_report_ids: Vec::new(),
        generation_context: json!({"stage": "0.16"}),
    })
    .expect("shot");

    let storyboards = repo
        .list_storyboards_with_typed_shots(&project_id)
        .expect("storyboards with shots");

    assert_eq!(storyboards.len(), 1);
    assert_eq!(storyboards[0].storyboard.id, storyboard.id);
    assert_eq!(storyboards[0].shots.len(), 1);
    assert_eq!(storyboards[0].shots[0].shot_number, 1);
}

#[test]
fn stores_research_report_findings_and_sources() {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".to_string(),
            description: "Premium womenswear".to_string(),
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
                    "title": "Texture proof point",
                    "insight": "Fabric closeups should lead the edit",
                    "source_index": 1
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
        .unwrap();

    let reports = repo.list_research_reports(&project.id).unwrap();

    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].id, report.id);
    assert_eq!(reports[0].findings_json[0]["title"], "Texture proof point");
    assert_eq!(reports[0].sources_json[0]["title"], "Reference note");
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
