use serde::{Deserialize, Serialize};

use crate::delivery_report::DeliveryPackagePreview;
use crate::error::JoiResult;
use crate::models::{AgentRun, AgentRunEvent};
use crate::repositories::{Repository, StoryboardWithShots};
use crate::research::ResearchSourceInput;

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
