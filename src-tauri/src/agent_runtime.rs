use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::agent_context::{build_project_context, AgentProjectContext};
use crate::error::JoiResult;
use crate::models::{AgentRun, AgentRunEvent};
use crate::repositories::{AgentRunCreate, AgentRunEventCreate, Repository};
use crate::validation::validate_required_text;

pub const AGENT_ROLES: [&str; 6] = [
    "planner",
    "researcher",
    "storyboard_writer",
    "prompt_adapter",
    "reviewer",
    "memory_curator",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentPlanInput {
    pub project_id: String,
    pub user_goal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPlanResult {
    pub run: AgentRun,
    pub events: Vec<AgentRunEvent>,
}

struct AgentEventSpec {
    sequence_number: i64,
    role: &'static str,
    event_type: &'static str,
    message: String,
    payload_json: Value,
}

pub fn start_agent_plan(
    repo: &Repository<'_>,
    input: AgentPlanInput,
    hermes_version: String,
) -> JoiResult<AgentPlanResult> {
    validate_required_text("Agent goal", &input.user_goal)?;
    let context = build_project_context(repo, &input.project_id)?;
    let plan_json = build_plan_json(&context);
    let roles_json = json!(AGENT_ROLES);
    let result_summary = build_result_summary(&context);
    let run = repo.create_agent_run(AgentRunCreate {
        project_id: input.project_id,
        user_goal: input.user_goal,
        status: "completed".to_string(),
        runtime_kind: "hermes_core".to_string(),
        runtime_mode: "local_planner_bridge".to_string(),
        runtime_version: hermes_version,
        roles_json,
        plan_json,
        result_summary,
    })?;

    let mut events = Vec::new();
    for spec in build_event_specs(&context) {
        events.push(repo.create_agent_run_event(AgentRunEventCreate {
            agent_run_id: run.id.clone(),
            sequence_number: spec.sequence_number,
            role: spec.role.to_string(),
            event_type: spec.event_type.to_string(),
            message: spec.message,
            payload_json: spec.payload_json,
        })?);
    }

    Ok(AgentPlanResult { run, events })
}

pub fn build_plan_json(context: &AgentProjectContext) -> Value {
    let project_title = context.project.title.as_str();
    let product_name = product_name(context);
    json!([
        {
            "role": "planner",
            "stage": "0.13",
            "title": "Confirm brief and material context",
            "task": format!("Confirm the saved context for {} and {} before downstream execution.", project_title, product_name),
            "status": "queued"
        },
        {
            "role": "researcher",
            "stage": "0.14",
            "title": "Prepare research questions",
            "task": "Define the trend, competitor, audience, and platform questions that should be researched next.",
            "status": "queued"
        },
        {
            "role": "storyboard_writer",
            "stage": "0.16",
            "title": "Prepare storyboard generation inputs",
            "task": "Translate the project goal, product understanding, and creative direction into storyboard input requirements.",
            "status": "queued"
        },
        {
            "role": "prompt_adapter",
            "stage": "0.17",
            "title": "Prepare prompt adapter targets",
            "task": "Identify video and image prompt targets for Jimeng, Grok, Banana 2, Jimeng Image, and GPT Image 2.",
            "status": "queued"
        },
        {
            "role": "reviewer",
            "stage": "0.19",
            "title": "Prepare quality review checklist",
            "task": "Prepare checks for brand fit, prompt completeness, scene continuity, and platform-specific constraints.",
            "status": "queued"
        },
        {
            "role": "memory_curator",
            "stage": "0.15",
            "title": "Prepare memory capture points",
            "task": "Identify stable brand and project facts worth saving after future workflow runs.",
            "status": "queued"
        }
    ])
}

fn build_event_specs(context: &AgentProjectContext) -> Vec<AgentEventSpec> {
    let project_title = context.project.title.clone();
    let product_name = product_name(context);
    vec![
        AgentEventSpec {
            sequence_number: 1,
            role: "planner",
            event_type: "context_read",
            message: format!(
                "Read saved context for {} with product focus {}.",
                project_title, product_name
            ),
            payload_json: json!({
                "brand_name": context.brand.name.clone(),
                "project_title": context.project.title.clone(),
                "asset_count": context.assets.len(),
                "memory_count": context.project_memory.len(),
                "version_count": context.versions.len(),
                "has_product_understanding": context.latest_product_understanding.is_some(),
                "has_creative_direction": context.latest_creative_direction.is_some()
            }),
        },
        AgentEventSpec {
            sequence_number: 2,
            role: "planner",
            event_type: "plan_created",
            message: "Created a local planner bridge task plan with six Joi roles.".to_string(),
            payload_json: json!({
                "runtime_mode": "local_planner_bridge",
                "roles": AGENT_ROLES
            }),
        },
        AgentEventSpec {
            sequence_number: 3,
            role: "researcher",
            event_type: "task_queued",
            message: "Queued research question preparation for 0.14.".to_string(),
            payload_json: json!({"stage": "0.14"}),
        },
        AgentEventSpec {
            sequence_number: 4,
            role: "storyboard_writer",
            event_type: "task_queued",
            message: "Queued storyboard input preparation for 0.16.".to_string(),
            payload_json: json!({"stage": "0.16"}),
        },
        AgentEventSpec {
            sequence_number: 5,
            role: "prompt_adapter",
            event_type: "task_queued",
            message: "Queued prompt adapter target preparation for 0.17.".to_string(),
            payload_json: json!({
                "stage": "0.17",
                "video_platforms": ["jimeng_video", "grok_video"],
                "image_platforms": ["banana_2_image", "jimeng_image", "gpt_image_2"]
            }),
        },
        AgentEventSpec {
            sequence_number: 6,
            role: "reviewer",
            event_type: "task_queued",
            message: "Queued quality review checklist preparation for 0.19.".to_string(),
            payload_json: json!({"stage": "0.19"}),
        },
        AgentEventSpec {
            sequence_number: 7,
            role: "memory_curator",
            event_type: "task_queued",
            message: "Queued memory capture point preparation for 0.15.".to_string(),
            payload_json: json!({"stage": "0.15"}),
        },
    ]
}

fn build_result_summary(context: &AgentProjectContext) -> String {
    format!(
        "Created a local planner bridge run for {} / {} with {} role tasks.",
        context.project.title,
        product_name(context),
        AGENT_ROLES.len()
    )
}

fn product_name(context: &AgentProjectContext) -> String {
    context
        .latest_product_understanding
        .as_ref()
        .and_then(|understanding| {
            let name = understanding.product_name.trim();
            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            }
        })
        .unwrap_or_else(|| "unspecified product".to_string())
}
