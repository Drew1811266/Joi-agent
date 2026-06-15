mod common;

use common::TestApp;
use joi_agent_lib::db::Database;
use joi_agent_lib::error::JoiError;
use joi_agent_lib::models::Project;
use joi_agent_lib::repositories::{
    BrandCreate, CreativeDirectionCreate, MemoryCandidateCreate, MemoryStatusUpdate,
    ProductUnderstandingCreate, ProjectCreate, Repository, ResearchReportCreate,
};
use joi_agent_lib::storyboard::{
    generate_storyboard, regenerate_shot, ShotRegenerationInput, StoryboardGenerationInput,
};
use serde_json::json;

fn open_repo(app: &TestApp) -> Database {
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    db
}

fn seed_storyboard_project(repo: &Repository<'_>, duration_seconds: i64) -> Project {
    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".into(),
            description: "Contemporary womenswear with clean studio lighting".into(),
        })
        .expect("brand");
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id,
            title: "Spring Drop Film".into(),
            advertising_goal: "Launch a lightweight trench collection".into(),
            duration_seconds,
        })
        .expect("project");
    repo.create_product_understanding(ProductUnderstandingCreate {
        project_id: project.id.clone(),
        product_name: "Lightweight trench".into(),
        category: "outerwear".into(),
        audience: "urban commuters".into(),
        selling_points: vec![
            "water-resistant cotton".into(),
            "soft structure".into(),
            "easy movement".into(),
        ],
        constraints: vec!["avoid heavy winter styling".into()],
        notes: json!({
            "brief_summary": "15 second outerwear launch ad",
            "visual_direction": "clean studio walk with close fabric texture"
        })
        .to_string(),
    })
    .expect("understanding");
    repo.create_creative_direction(CreativeDirectionCreate {
        project_id: project.id.clone(),
        title: "Clean tactile motion".into(),
        concept: "Show fabric proof before model movement.".into(),
        tone: "premium and direct".into(),
        visual_style: "minimal warm studio, tactile close-ups".into(),
        scene_direction: "studio entrance, macro insert, walking motion, closing product pose"
            .into(),
        rationale: "Derived from brief and material understanding.".into(),
    })
    .expect("creative direction");
    repo.create_research_report(ResearchReportCreate {
        project_id: project.id.clone(),
        summary: "Reference-backed tactile product proof.".into(),
        findings_json: json!([
            {
                "title": "Texture proof",
                "insight": "Fabric detail supports premium positioning.",
                "creative_implication": "Use tactile close-ups as visual proof before the model movement."
            }
        ]),
        sources_json: json!([
            {
                "index": 1,
                "title": "Reference note",
                "url": "https://example.com/reference",
                "source_type": "reference",
                "excerpt": "Texture details support premium positioning."
            }
        ]),
    })
    .expect("research");
    project
}

#[test]
fn generates_duration_balanced_storyboard_from_project_context() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());
    let project = seed_storyboard_project(&repo, 15);

    let result = generate_storyboard(
        &repo,
        StoryboardGenerationInput {
            project_id: project.id.clone(),
            user_direction: "Make the opening feel tactile and premium.".into(),
            preferred_duration_seconds: None,
            preferred_shot_count: None,
        },
        "local-test".into(),
    )
    .expect("storyboard");

    assert_eq!(result.storyboard.project_id, project.id);
    assert_eq!(result.storyboard.duration_seconds, 15);
    assert_eq!(result.shots.len(), 5);
    assert_eq!(result.total_duration_seconds, 15);
    assert_eq!(
        result
            .shots
            .iter()
            .map(|item| item.shot.duration_seconds)
            .sum::<i64>(),
        15
    );
    assert_eq!(result.shots[0].shot.shot_number, 1);
    assert!(result.shots[0].visual_description.contains("trench"));
    assert!(result
        .shots
        .iter()
        .any(|item| item.garment_focus.contains("cotton")));
    assert_eq!(result.agent_run.runtime_mode, "local_storyboard_bridge");
    assert_eq!(result.agent_events.len(), 6);
}

#[test]
fn generation_uses_accepted_memory_and_ignores_rejected_memory() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());
    let project = seed_storyboard_project(&repo, 15);
    let accepted = repo
        .create_memory_candidate(MemoryCandidateCreate {
            scope: "project".into(),
            brand_id: None,
            project_id: Some(project.id.clone()),
            content: "Use tactile close-ups before model movement.".into(),
            source: "user feedback".into(),
            source_entity_type: "feedback".into(),
            source_entity_id: String::new(),
            confidence: 0.86,
        })
        .expect("accepted memory seed");
    repo.update_memory_entry_status(MemoryStatusUpdate {
        id: accepted.id.clone(),
        status: "accepted".into(),
    })
    .expect("accept memory");
    let rejected = repo
        .create_memory_candidate(MemoryCandidateCreate {
            scope: "project".into(),
            brand_id: None,
            project_id: Some(project.id.clone()),
            content: "Make the opening shot dark and unrelated to the product.".into(),
            source: "user feedback".into(),
            source_entity_type: "feedback".into(),
            source_entity_id: String::new(),
            confidence: 0.86,
        })
        .expect("rejected memory seed");
    repo.update_memory_entry_status(MemoryStatusUpdate {
        id: rejected.id,
        status: "rejected".into(),
    })
    .expect("reject memory");

    let result = generate_storyboard(
        &repo,
        StoryboardGenerationInput {
            project_id: project.id,
            user_direction: String::new(),
            preferred_duration_seconds: Some(15),
            preferred_shot_count: Some(5),
        },
        "local-test".into(),
    )
    .expect("storyboard");

    let used_memory_ids = result
        .shots
        .iter()
        .flat_map(|item| {
            item.shot.metadata_json["source_memory_ids"]
                .as_array()
                .cloned()
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();

    assert!(used_memory_ids.contains(&json!(accepted.id)));
    assert!(!result
        .shots
        .iter()
        .any(|item| item.visual_description.to_lowercase().contains("unrelated")));
}

#[test]
fn rejects_storyboard_duration_outside_short_ad_range() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());
    let project = seed_storyboard_project(&repo, 45);

    let error = generate_storyboard(
        &repo,
        StoryboardGenerationInput {
            project_id: project.id,
            user_direction: String::new(),
            preferred_duration_seconds: None,
            preferred_shot_count: None,
        },
        "local-test".into(),
    )
    .expect_err("duration should fail");

    assert!(
        matches!(error, JoiError::Validation(message) if message == "Storyboard duration must be between 15 and 30 seconds")
    );
}

#[test]
fn regenerates_selected_unlocked_shot_and_preserves_duration() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());
    let project = seed_storyboard_project(&repo, 15);
    let generated = generate_storyboard(
        &repo,
        StoryboardGenerationInput {
            project_id: project.id.clone(),
            user_direction: String::new(),
            preferred_duration_seconds: Some(15),
            preferred_shot_count: Some(5),
        },
        "local-test".into(),
    )
    .expect("storyboard");
    let original = generated.shots[1].shot.clone();

    let result = regenerate_shot(
        &repo,
        ShotRegenerationInput {
            project_id: project.id,
            storyboard_id: generated.storyboard.id,
            shot_id: original.id.clone(),
            revision_note: "Make this shot a clearer fabric macro insert.".into(),
        },
        "local-test".into(),
    )
    .expect("regenerate shot");

    assert_eq!(result.shot.shot.id, original.id);
    assert_eq!(result.shot.shot.shot_number, original.shot_number);
    assert_eq!(result.shot.shot.duration_seconds, original.duration_seconds);
    assert!(result.shot.garment_focus.to_lowercase().contains("fabric"));
    assert_eq!(
        result.agent_run.runtime_mode,
        "local_storyboard_regeneration_bridge"
    );
    assert_eq!(result.agent_events.len(), 4);
}
