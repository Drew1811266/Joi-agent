use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::delivery_report::{
    generate_delivery_report, preview_delivery_package, DeliveryPackagePreview,
    DeliveryReportGenerationInput,
};
use crate::error::JoiResult;
use crate::memory_curation::{curate_memory_candidates, MemoryCurationInput};
use crate::models::{AgentRun, AgentRunEvent};
use crate::prompt_adapter::{generate_prompt_packages, PromptGenerationInput};
use crate::quality_review::{generate_quality_review, QualityReviewGenerationInput};
use crate::repositories::{AgentRunCreate, AgentRunEventCreate, Repository, StoryboardWithShots};
use crate::research::{generate_research_report, ResearchReportInput, ResearchSourceInput};
use crate::snapshots::{ProjectSnapshotService, SaveSnapshotInput};
use crate::storyboard::{generate_storyboard, StoryboardGenerationInput};
use crate::understanding::{generate_brief_understanding, BriefUnderstandingInput};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BetaWorkflowStep {
    pub id: String,
    pub title: String,
    pub status: String,
    pub source_count: usize,
    pub target_tab: String,
    pub action_label: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BetaWorkflowStatusResult {
    pub project_id: String,
    pub ready: bool,
    pub score: i64,
    pub steps: Vec<BetaWorkflowStep>,
    pub next_action: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BetaWorkflowRunInput {
    pub project_id: String,
    pub user_direction: String,
    pub image_brief: String,
    pub reference_sources: Vec<ResearchSourceInput>,
    pub memory_feedback: String,
    pub save_snapshot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetaWorkflowRunResult {
    pub status: BetaWorkflowStatusResult,
    pub generated_steps: Vec<String>,
    pub skipped_steps: Vec<String>,
    pub delivery_report_id: Option<String>,
    pub package_preview: Option<DeliveryPackagePreview>,
    pub snapshot_id: Option<String>,
    pub agent_run: AgentRun,
    pub agent_events: Vec<AgentRunEvent>,
}

pub fn run_beta_workflow(
    repo: &Repository<'_>,
    input: BetaWorkflowRunInput,
    hermes_version: String,
) -> JoiResult<BetaWorkflowRunResult> {
    let project = repo.get_project(&input.project_id)?;
    let brand = repo.get_brand(&project.brand_id)?;
    let assets = repo.list_assets(&input.project_id)?;
    let mut generated_steps = Vec::new();
    let mut skipped_steps = Vec::new();

    if repo
        .list_product_understandings(&input.project_id)?
        .is_empty()
        || repo.list_creative_directions(&input.project_id)?.is_empty()
    {
        generate_brief_understanding(
            repo,
            BriefUnderstandingInput {
                project_id: input.project_id.clone(),
                brief_text: project.advertising_goal.clone(),
                product_name: project.title.clone(),
                category: "fashion collection".to_string(),
                audience: "short-form fashion ad viewers".to_string(),
                target_platforms: vec!["jimeng_video".to_string(), "grok_video".to_string()],
                selling_points_text: brand.description.clone(),
                visual_direction: default_visual_direction(&brand.description),
                constraints_text:
                    "Keep garment shape, fabric texture, and brand styling consistent.".to_string(),
                reference_asset_ids: assets.iter().map(|asset| asset.id.clone()).collect(),
            },
        )?;
        generated_steps.push("understanding".to_string());
    } else {
        skipped_steps.push("understanding".to_string());
    }

    if repo.list_research_reports(&input.project_id)?.is_empty()
        && !input.reference_sources.is_empty()
    {
        generate_research_report(
            repo,
            ResearchReportInput {
                project_id: input.project_id.clone(),
                research_goal: "Build source-backed fashion ad direction for beta workflow."
                    .to_string(),
                market_focus: "fashion advertising".to_string(),
                platform_focus: vec!["jimeng_video".to_string(), "grok_video".to_string()],
                source_materials: input.reference_sources.clone(),
            },
            hermes_version.clone(),
        )?;
        generated_steps.push("research".to_string());
    } else {
        skipped_steps.push("research".to_string());
    }

    let storyboards = repo.list_storyboards_with_typed_shots(&input.project_id)?;
    if !storyboards.iter().any(|item| !item.shots.is_empty()) {
        generate_storyboard(
            repo,
            StoryboardGenerationInput {
                project_id: input.project_id.clone(),
                user_direction: input.user_direction.clone(),
                preferred_duration_seconds: Some(project.duration_seconds),
                preferred_shot_count: Some(5),
            },
            hermes_version.clone(),
        )?;
        generated_steps.push("storyboard".to_string());
    } else {
        skipped_steps.push("storyboard".to_string());
    }

    let current_storyboards = repo.list_storyboards_with_typed_shots(&input.project_id)?;
    let shot_ids = current_storyboards
        .last()
        .map(|item| {
            item.shots
                .iter()
                .map(|shot| shot.id.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let prompt_packages = repo.list_prompt_packages(&input.project_id)?;
    if !shot_ids.is_empty()
        && prompt_packages
            .iter()
            .filter(|package| package.modality == "video")
            .count()
            < 2
    {
        generate_prompt_packages(
            repo,
            PromptGenerationInput {
                project_id: input.project_id.clone(),
                shot_ids: shot_ids.clone(),
                image_brief: String::new(),
                target_platforms: vec!["jimeng_video".to_string(), "grok_video".to_string()],
                user_direction: input.user_direction.clone(),
            },
            hermes_version.clone(),
        )?;
        generated_steps.push("video_prompts".to_string());
    } else {
        skipped_steps.push("video_prompts".to_string());
    }

    let prompt_packages = repo.list_prompt_packages(&input.project_id)?;
    if prompt_packages
        .iter()
        .filter(|package| package.modality == "image")
        .count()
        < 3
    {
        generate_prompt_packages(
            repo,
            PromptGenerationInput {
                project_id: input.project_id.clone(),
                shot_ids: Vec::new(),
                image_brief: default_image_brief(&project.title, &input.image_brief),
                target_platforms: vec![
                    "banana_2_image".to_string(),
                    "jimeng_image".to_string(),
                    "gpt_image_2".to_string(),
                ],
                user_direction: input.user_direction.clone(),
            },
            hermes_version.clone(),
        )?;
        generated_steps.push("image_prompts".to_string());
    } else {
        skipped_steps.push("image_prompts".to_string());
    }

    if repo.list_quality_reviews(&input.project_id)?.is_empty() {
        generate_quality_review(
            repo,
            QualityReviewGenerationInput {
                project_id: input.project_id.clone(),
                user_direction: "Review beta workflow outputs before delivery.".to_string(),
            },
            hermes_version.clone(),
        )?;
        generated_steps.push("quality_review".to_string());
    } else {
        skipped_steps.push("quality_review".to_string());
    }

    if repo
        .list_memory_entries_for_project(&input.project_id)?
        .iter()
        .all(|entry| entry.status != "accepted")
    {
        let feedback_text = if input.memory_feedback.trim().is_empty() {
            "Capture reusable brand and production preferences from the completed beta workflow."
                .to_string()
        } else {
            input.memory_feedback.clone()
        };
        curate_memory_candidates(
            repo,
            MemoryCurationInput {
                project_id: input.project_id.clone(),
                feedback_text,
                include_research_reports: true,
            },
            hermes_version.clone(),
        )?;
        generated_steps.push("memory_candidates".to_string());
    } else {
        skipped_steps.push("memory_candidates".to_string());
    }

    let delivery_reports = repo.list_delivery_reports(&input.project_id)?;
    let delivery_report_id = if delivery_reports.is_empty() {
        let report = generate_delivery_report(
            repo,
            DeliveryReportGenerationInput {
                project_id: input.project_id.clone(),
                user_direction: "Prepare beta delivery package summary.".to_string(),
            },
            hermes_version.clone(),
        )?;
        generated_steps.push("delivery_report".to_string());
        Some(report.report.id)
    } else {
        skipped_steps.push("delivery_report".to_string());
        delivery_reports.last().map(|report| report.id.clone())
    };

    let package_preview = Some(preview_delivery_package(
        repo,
        &input.project_id,
        delivery_report_id.as_deref(),
    )?);

    let snapshot_id = if input.save_snapshot {
        let snapshot =
            ProjectSnapshotService::new(repo.connection()).save_snapshot(SaveSnapshotInput {
                project_id: input.project_id.clone(),
                label: "0.20 beta workflow snapshot".to_string(),
                change_reason: "Saved after beta workflow run.".to_string(),
                changed_entities: generated_steps.clone(),
                created_by: "joi-beta-workflow".to_string(),
                is_final_candidate: true,
            })?;
        generated_steps.push("snapshot".to_string());
        Some(snapshot.id)
    } else {
        skipped_steps.push("snapshot".to_string());
        None
    };

    let agent_run = repo.create_agent_run(AgentRunCreate {
        project_id: input.project_id.clone(),
        user_goal: "Run Joi 0.20 usable beta workflow.".to_string(),
        status: "completed".to_string(),
        runtime_kind: "hermes_core".to_string(),
        runtime_mode: "local_beta_workflow_bridge".to_string(),
        runtime_version: hermes_version,
        roles_json: json!([
            "planner",
            "storyboard_writer",
            "prompt_adapter",
            "reviewer",
            "memory_curator"
        ]),
        plan_json: json!({
            "generated_steps": generated_steps.clone(),
            "skipped_steps": skipped_steps.clone(),
            "save_snapshot": input.save_snapshot
        }),
        result_summary: format!(
            "Completed beta workflow with {} generated step(s) and {} skipped step(s).",
            generated_steps.len(),
            skipped_steps.len()
        ),
    })?;
    let agent_events = create_beta_events(repo, &agent_run.id, &generated_steps, &skipped_steps)?;
    let status = assess_beta_workflow(repo, &input.project_id)?;

    Ok(BetaWorkflowRunResult {
        status,
        generated_steps,
        skipped_steps,
        delivery_report_id,
        package_preview,
        snapshot_id,
        agent_run,
        agent_events,
    })
}

pub fn assess_beta_workflow(
    repo: &Repository<'_>,
    project_id: &str,
) -> JoiResult<BetaWorkflowStatusResult> {
    let project = repo.get_project(project_id)?;
    let brand = repo.get_brand(&project.brand_id)?;
    let assets = repo.list_assets(project_id)?;
    let understandings = repo.list_product_understandings(project_id)?;
    let directions = repo.list_creative_directions(project_id)?;
    let research = repo.list_research_reports(project_id)?;
    let storyboards = repo.list_storyboards_with_typed_shots(project_id)?;
    let prompts = repo.list_prompt_packages(project_id)?;
    let reviews = repo.list_quality_reviews(project_id)?;
    let delivery_reports = repo.list_delivery_reports(project_id)?;
    let memory = repo.list_memory_entries_for_project(project_id)?;
    let versions = repo.list_project_versions(project_id)?;

    let has_storyboard_shots = storyboards.iter().any(|item| !item.shots.is_empty());
    let video_prompt_count = prompts
        .iter()
        .filter(|package| package.modality == "video")
        .count();
    let image_prompt_count = prompts
        .iter()
        .filter(|package| package.modality == "image")
        .count();
    let accepted_memory_count = memory
        .iter()
        .filter(|entry| entry.status == "accepted")
        .count();

    let steps = vec![
        step(
            "project_setup",
            "Project setup",
            "Overview",
            "Edit project",
            1,
            !brand.name.trim().is_empty()
                && !project.title.trim().is_empty()
                && !project.advertising_goal.trim().is_empty()
                && project.duration_seconds > 0,
            "Brand and project context are saved.",
            "Brand, project title, goal, and duration are required.",
        ),
        optional_step(
            "reference_materials",
            "Reference materials",
            "Brief",
            "Add reference",
            assets.len(),
            !assets.is_empty(),
            "Reference materials are available.",
            "Add at least one reference image, video, or link for the benchmark.",
        ),
        step(
            "understanding",
            "Product understanding",
            "Brief",
            "Generate understanding",
            understandings.len() + directions.len(),
            !understandings.is_empty() && !directions.is_empty(),
            "Product understanding and creative direction are saved.",
            "Generate product understanding and creative direction.",
        ),
        optional_step(
            "research",
            "Research report",
            "Research",
            "Generate research",
            research.len(),
            !research.is_empty(),
            "Research report is saved.",
            "Generate a source-backed research report.",
        ),
        step(
            "storyboard",
            "Storyboard",
            "Storyboard",
            "Generate storyboard",
            shot_count(&storyboards),
            has_storyboard_shots,
            "Storyboard shots are available.",
            "Generate a 15-30 second storyboard.",
        ),
        step(
            "video_prompts",
            "Video prompts",
            "Prompts",
            "Generate video prompts",
            video_prompt_count,
            video_prompt_count >= 2,
            "Jimeng and Grok video prompts are available.",
            "Generate Jimeng and Grok video prompts.",
        ),
        step(
            "image_prompts",
            "Image prompts",
            "Prompts",
            "Generate image prompts",
            image_prompt_count,
            image_prompt_count >= 3,
            "Banana 2, Jimeng Image, and GPT Image 2 prompts are available.",
            "Generate image prompt packages.",
        ),
        step(
            "quality_review",
            "Quality review",
            "Review",
            "Generate review",
            reviews.len(),
            !reviews.is_empty(),
            "Quality review is saved.",
            "Generate a quality review and apply selected suggestions.",
        ),
        step(
            "delivery_report",
            "Delivery report",
            "Delivery",
            "Generate report",
            delivery_reports.len(),
            !delivery_reports.is_empty(),
            "Delivery report is saved.",
            "Generate a delivery report.",
        ),
        step(
            "accepted_memory",
            "Accepted memory",
            "Memory",
            "Review memory",
            accepted_memory_count,
            accepted_memory_count > 0,
            "Accepted project memory is available.",
            "Accept at least one memory candidate or add project memory.",
        ),
        step(
            "snapshot",
            "Snapshot",
            "Versions",
            "Save snapshot",
            versions.len(),
            !versions.is_empty(),
            "At least one project snapshot is saved.",
            "Save a project snapshot.",
        ),
    ];

    let required_ids = [
        "project_setup",
        "understanding",
        "storyboard",
        "video_prompts",
        "image_prompts",
        "quality_review",
        "delivery_report",
        "accepted_memory",
        "snapshot",
    ];
    let ready = required_ids.iter().all(|id| {
        steps
            .iter()
            .any(|step| step.id == *id && step.status == "complete")
    });
    let score = steps
        .iter()
        .map(|step| match step.status.as_str() {
            "complete" => 10,
            "warning" => 6,
            _ => 0,
        })
        .sum::<i64>();
    let next_action = steps
        .iter()
        .find(|step| step.status == "action_required")
        .map(|step| step.action_label.clone())
        .unwrap_or_else(|| "Review beta package".to_string());
    let warnings = steps
        .iter()
        .filter(|step| step.status == "warning")
        .map(|step| step.message.clone())
        .collect();

    Ok(BetaWorkflowStatusResult {
        project_id: project_id.to_string(),
        ready,
        score,
        steps,
        next_action,
        warnings,
    })
}

fn step(
    id: &str,
    title: &str,
    target_tab: &str,
    action_label: &str,
    source_count: usize,
    complete: bool,
    complete_message: &str,
    missing_message: &str,
) -> BetaWorkflowStep {
    BetaWorkflowStep {
        id: id.to_string(),
        title: title.to_string(),
        status: if complete {
            "complete"
        } else {
            "action_required"
        }
        .to_string(),
        source_count,
        target_tab: target_tab.to_string(),
        action_label: action_label.to_string(),
        message: if complete {
            complete_message
        } else {
            missing_message
        }
        .to_string(),
    }
}

fn optional_step(
    id: &str,
    title: &str,
    target_tab: &str,
    action_label: &str,
    source_count: usize,
    complete: bool,
    complete_message: &str,
    missing_message: &str,
) -> BetaWorkflowStep {
    BetaWorkflowStep {
        id: id.to_string(),
        title: title.to_string(),
        status: if complete { "complete" } else { "warning" }.to_string(),
        source_count,
        target_tab: target_tab.to_string(),
        action_label: action_label.to_string(),
        message: if complete {
            complete_message
        } else {
            missing_message
        }
        .to_string(),
    }
}

fn shot_count(storyboards: &[StoryboardWithShots]) -> usize {
    storyboards.iter().map(|item| item.shots.len()).sum()
}

fn default_visual_direction(brand_description: &str) -> String {
    if brand_description.trim().is_empty() {
        "Clean fashion advertising visuals with clear garment visibility.".to_string()
    } else {
        brand_description.trim().to_string()
    }
}

fn default_image_brief(project_title: &str, image_brief: &str) -> String {
    if image_brief.trim().is_empty() {
        format!(
            "Full-body fashion model photo for {}, clean studio lighting, visible garment texture, brand-consistent styling.",
            project_title
        )
    } else {
        image_brief.trim().to_string()
    }
}

fn create_beta_events(
    repo: &Repository<'_>,
    agent_run_id: &str,
    generated_steps: &[String],
    skipped_steps: &[String],
) -> JoiResult<Vec<AgentRunEvent>> {
    let events = [
        (
            1,
            "planner",
            "beta_context_assessed",
            "Assessed project readiness for beta workflow.",
            json!({
                "generated_step_count": generated_steps.len(),
                "skipped_step_count": skipped_steps.len()
            }),
        ),
        (
            2,
            "planner",
            "beta_steps_generated",
            "Generated missing beta workflow outputs.",
            json!({ "generated_steps": generated_steps }),
        ),
        (
            3,
            "reviewer",
            "beta_steps_skipped",
            "Skipped outputs that were already present or needed manual input.",
            json!({ "skipped_steps": skipped_steps }),
        ),
    ];

    events
        .into_iter()
        .map(
            |(sequence_number, role, event_type, message, payload_json)| {
                repo.create_agent_run_event(AgentRunEventCreate {
                    agent_run_id: agent_run_id.to_string(),
                    sequence_number,
                    role: role.to_string(),
                    event_type: event_type.to_string(),
                    message: message.to_string(),
                    payload_json,
                })
            },
        )
        .collect()
}
