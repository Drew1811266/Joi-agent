use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::agent_context::{build_project_context, AgentProjectContext};
use crate::error::{JoiError, JoiResult};
use crate::models::{AgentRun, AgentRunEvent, MemoryEntry, Shot, Storyboard};
use crate::repositories::{
    AgentRunCreate, AgentRunEventCreate, Repository, ShotPlanCreate, ShotUpdate, StoryboardCreate,
};

const STORYBOARD_ROLES: [&str; 3] = ["planner", "storyboard_writer", "reviewer"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryboardGenerationInput {
    pub project_id: String,
    pub user_direction: String,
    pub preferred_duration_seconds: Option<i64>,
    pub preferred_shot_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryboardShotView {
    pub shot: Shot,
    pub visual_description: String,
    pub garment_focus: String,
    pub transition: String,
}

impl StoryboardShotView {
    pub fn from_shot(shot: Shot) -> Self {
        let visual_description = shot.description.clone();
        let garment_focus = string_field(&shot.metadata_json, "garment_focus").unwrap_or_default();
        let transition = string_field(&shot.metadata_json, "transition").unwrap_or_default();
        Self {
            shot,
            visual_description,
            garment_focus,
            transition,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryboardGenerationResult {
    pub storyboard: Storyboard,
    pub shots: Vec<StoryboardShotView>,
    pub total_duration_seconds: i64,
    pub agent_run: AgentRun,
    pub agent_events: Vec<AgentRunEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShotRegenerationInput {
    pub project_id: String,
    pub storyboard_id: String,
    pub shot_id: String,
    pub revision_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShotRegenerationResult {
    pub shot: StoryboardShotView,
    pub agent_run: AgentRun,
    pub agent_events: Vec<AgentRunEvent>,
}

struct ShotDraft {
    shot_number: i64,
    duration_seconds: i64,
    visual_description: String,
    model_action: String,
    garment_focus: String,
    camera_movement: String,
    scene: String,
    lighting: String,
    transition: String,
    subtitle_or_text: String,
    rationale: String,
    source_memory_ids: Vec<String>,
    source_research_report_ids: Vec<String>,
    generation_context: Value,
}

struct StoryboardEventSpec {
    sequence_number: i64,
    role: &'static str,
    event_type: &'static str,
    message: String,
    payload_json: Value,
}

pub fn generate_storyboard(
    repo: &Repository<'_>,
    input: StoryboardGenerationInput,
    hermes_version: String,
) -> JoiResult<StoryboardGenerationResult> {
    let context = build_project_context(repo, &input.project_id)?;
    let duration_seconds = resolve_duration(&input, context.project.duration_seconds)?;
    let shot_count = resolve_shot_count(&input, duration_seconds)?;
    let research = research_implications(repo, &input.project_id)?;
    let drafts = build_shot_drafts(&context, &research, &input, duration_seconds, shot_count);
    let storyboard = repo.create_storyboard(StoryboardCreate {
        project_id: input.project_id.clone(),
        title: format!("{} storyboard", context.project.title),
        duration_seconds,
    })?;

    let mut shots = Vec::with_capacity(drafts.len());
    for draft in drafts {
        let shot = repo.create_shot_plan(ShotPlanCreate {
            storyboard_id: storyboard.id.clone(),
            shot_number: draft.shot_number,
            duration_seconds: draft.duration_seconds,
            visual_description: draft.visual_description,
            model_action: draft.model_action,
            garment_focus: draft.garment_focus,
            camera_movement: draft.camera_movement,
            scene: draft.scene,
            lighting: draft.lighting,
            transition: draft.transition,
            subtitle_or_text: draft.subtitle_or_text,
            rationale: draft.rationale,
            source_memory_ids: draft.source_memory_ids,
            source_research_report_ids: draft.source_research_report_ids,
            generation_context: draft.generation_context,
        })?;
        shots.push(StoryboardShotView::from_shot(shot));
    }

    let total_duration_seconds = shots
        .iter()
        .map(|item| item.shot.duration_seconds)
        .sum::<i64>();
    let result_summary = format!(
        "Generated {} shot storyboard for {} ({}s).",
        shots.len(),
        context.project.title,
        duration_seconds
    );
    let agent_run = repo.create_agent_run(AgentRunCreate {
        project_id: input.project_id.clone(),
        user_goal: generation_goal(&context, &input),
        status: "completed".to_string(),
        runtime_kind: "hermes_core".to_string(),
        runtime_mode: "local_storyboard_bridge".to_string(),
        runtime_version: hermes_version,
        roles_json: json!(STORYBOARD_ROLES),
        plan_json: build_generation_plan_json(&input, duration_seconds, shot_count),
        result_summary,
    })?;

    let mut agent_events = Vec::new();
    for spec in build_generation_event_specs(
        &context,
        &input,
        &storyboard,
        &shots,
        duration_seconds,
        shot_count,
        research.len(),
    ) {
        agent_events.push(repo.create_agent_run_event(AgentRunEventCreate {
            agent_run_id: agent_run.id.clone(),
            sequence_number: spec.sequence_number,
            role: spec.role.to_string(),
            event_type: spec.event_type.to_string(),
            message: spec.message,
            payload_json: spec.payload_json,
        })?);
    }

    Ok(StoryboardGenerationResult {
        storyboard,
        shots,
        total_duration_seconds,
        agent_run,
        agent_events,
    })
}

pub fn regenerate_shot(
    repo: &Repository<'_>,
    input: ShotRegenerationInput,
    hermes_version: String,
) -> JoiResult<ShotRegenerationResult> {
    let context = build_project_context(repo, &input.project_id)?;
    let storyboards = repo.list_storyboards_with_typed_shots(&input.project_id)?;
    if !storyboards
        .iter()
        .any(|item| item.storyboard.id == input.storyboard_id)
    {
        return Err(JoiError::NotFound(format!(
            "storyboard {}",
            input.storyboard_id
        )));
    }

    let original = repo.get_shot(&input.shot_id)?;
    if original.storyboard_id != input.storyboard_id {
        return Err(JoiError::Validation(
            "shot does not belong to storyboard".to_string(),
        ));
    }
    if original.is_locked {
        return Err(JoiError::Validation(
            "Locked shots cannot be regenerated".to_string(),
        ));
    }

    let research = research_implications(repo, &input.project_id)?;
    let draft = build_regenerated_shot_draft(&context, &research, &input, &original);
    let updated = repo.update_shot(ShotUpdate {
        id: original.id,
        duration_seconds: original.duration_seconds,
        visual_description: draft.visual_description,
        model_action: draft.model_action,
        garment_focus: draft.garment_focus,
        camera_movement: draft.camera_movement,
        scene: draft.scene,
        lighting: draft.lighting,
        transition: draft.transition,
        subtitle_or_text: draft.subtitle_or_text,
        rationale: draft.rationale,
        is_locked: false,
    })?;
    let shot = StoryboardShotView::from_shot(updated);

    let agent_run = repo.create_agent_run(AgentRunCreate {
        project_id: input.project_id.clone(),
        user_goal: regeneration_goal(&context, &input),
        status: "completed".to_string(),
        runtime_kind: "hermes_core".to_string(),
        runtime_mode: "local_storyboard_regeneration_bridge".to_string(),
        runtime_version: hermes_version,
        roles_json: json!(STORYBOARD_ROLES),
        plan_json: build_regeneration_plan_json(&input),
        result_summary: format!(
            "Regenerated shot {} for {}.",
            shot.shot.shot_number, context.project.title
        ),
    })?;

    let mut agent_events = Vec::new();
    for spec in build_regeneration_event_specs(&context, &input, &shot) {
        agent_events.push(repo.create_agent_run_event(AgentRunEventCreate {
            agent_run_id: agent_run.id.clone(),
            sequence_number: spec.sequence_number,
            role: spec.role.to_string(),
            event_type: spec.event_type.to_string(),
            message: spec.message,
            payload_json: spec.payload_json,
        })?);
    }

    Ok(ShotRegenerationResult {
        shot,
        agent_run,
        agent_events,
    })
}

fn resolve_duration(input: &StoryboardGenerationInput, project_duration: i64) -> JoiResult<i64> {
    let duration = input.preferred_duration_seconds.unwrap_or(project_duration);
    if !(15..=30).contains(&duration) {
        return Err(JoiError::Validation(
            "Storyboard duration must be between 15 and 30 seconds".to_string(),
        ));
    }
    Ok(duration)
}

fn resolve_shot_count(input: &StoryboardGenerationInput, duration_seconds: i64) -> JoiResult<i64> {
    if let Some(count) = input.preferred_shot_count {
        if !(3..=10).contains(&count) {
            return Err(JoiError::Validation(
                "Storyboard shot count must be between 3 and 10".to_string(),
            ));
        }
        return Ok(count);
    }

    Ok(match duration_seconds {
        15 => 5,
        16..=20 => 6,
        21..=25 => 7,
        _ => 8,
    })
}

fn distribute_durations(total_duration_seconds: i64, shot_count: i64) -> Vec<i64> {
    let base = total_duration_seconds / shot_count;
    let remainder = total_duration_seconds % shot_count;
    (0..shot_count)
        .map(|index| if index < remainder { base + 1 } else { base })
        .collect()
}

fn build_shot_drafts(
    context: &AgentProjectContext,
    research: &[(String, String)],
    input: &StoryboardGenerationInput,
    duration_seconds: i64,
    shot_count: i64,
) -> Vec<ShotDraft> {
    let durations = distribute_durations(duration_seconds, shot_count);
    let product = product_name(context);
    let scene = scene_direction(context);
    let visual_style = visual_style(context);
    let selling_points = selling_points(context);
    let constraints = constraints(context);
    let accepted_memory = accepted_memory(context);
    let memory_instruction = accepted_memory
        .first()
        .map(|memory| memory.content.clone())
        .unwrap_or_else(|| "Keep the garment clearly visible in each shot.".to_string());
    let memory_ids = accepted_memory
        .iter()
        .take(2)
        .map(|memory| memory.id.clone())
        .collect::<Vec<_>>();
    let research_pair = research.first().cloned();
    let research_text = research_pair
        .as_ref()
        .map(|(_, implication)| implication.clone())
        .unwrap_or_else(|| "Use one clear product proof moment.".to_string());
    let research_ids = research_pair
        .as_ref()
        .map(|(id, _)| vec![id.clone()])
        .unwrap_or_default();

    durations
        .into_iter()
        .enumerate()
        .map(|(index, duration)| {
            let shot_number = index as i64 + 1;
            let selling_point = selling_points
                .get(index % selling_points.len())
                .cloned()
                .unwrap_or_else(|| "hero garment silhouette".to_string());
            let source_memory_ids = if index == 1 {
                memory_ids.clone()
            } else {
                Vec::new()
            };
            let source_research_report_ids = if index == 1 {
                research_ids.clone()
            } else {
                Vec::new()
            };
            let arc = shot_arc_label(index, shot_count as usize);
            let visual_description = match arc {
                "opening" => format!(
                    "{product} enters a {visual_style} frame, immediately showing the campaign mood."
                ),
                "proof" => format!(
                    "A close product-proof shot translates {research_text} into a visible {selling_point} moment."
                ),
                "movement" => format!(
                    "The model moves through {scene}, letting {product} show easy movement and fit."
                ),
                "styling" => format!(
                    "A styling beat places {product} in the target audience context while keeping {selling_point} readable."
                ),
                _ => format!(
                    "Final composed frame holds {product} with brand clarity and a clean product memory."
                ),
            };
            let model_action = match arc {
                "opening" => "Model steps into frame and turns slightly toward camera.".to_string(),
                "proof" => "Model lifts or adjusts the garment edge to reveal tactile detail.".to_string(),
                "movement" => "Model walks, pivots, and lets the garment move naturally.".to_string(),
                "styling" => "Model pauses in a wearable styling pose with relaxed confidence.".to_string(),
                _ => "Model settles into a final product-forward pose.".to_string(),
            };
            let camera_movement = match arc {
                "proof" => "macro slide across fabric texture".to_string(),
                "movement" => "side tracking move".to_string(),
                "styling" => "medium handheld drift".to_string(),
                "closing" => "slow pull-back".to_string(),
                _ => "slow push-in".to_string(),
            };
            let transition = match arc {
                "opening" => "cut on movement".to_string(),
                "proof" => "match cut into model movement".to_string(),
                "movement" => "motion cut".to_string(),
                "styling" => "soft cut to final pose".to_string(),
                _ => "end card hold".to_string(),
            };
            let subtitle_or_text = match arc {
                "opening" => format!("{} in motion", product),
                "proof" => selling_point.clone(),
                "movement" => "Built for real movement".to_string(),
                "styling" => "Light structure, everyday polish".to_string(),
                _ => "Spring Drop".to_string(),
            };
            let rationale = format!(
                "Shot {shot_number} covers the {arc} beat, uses {selling_point}, follows memory guidance: {memory_instruction}, and avoids: {}.",
                constraints.join(", ")
            );

            ShotDraft {
                shot_number,
                duration_seconds: duration,
                visual_description,
                model_action,
                garment_focus: format!("{selling_point} on {product}"),
                camera_movement,
                scene: scene.clone(),
                lighting: lighting(context),
                transition,
                subtitle_or_text,
                rationale,
                source_memory_ids,
                source_research_report_ids,
                generation_context: json!({
                    "stage": "0.16",
                    "source": "storyboard_generation",
                    "arc": arc,
                    "selling_point": selling_point,
                    "user_direction": input.user_direction.trim(),
                }),
            }
        })
        .collect()
}

fn build_regenerated_shot_draft(
    context: &AgentProjectContext,
    research: &[(String, String)],
    input: &ShotRegenerationInput,
    original: &Shot,
) -> ShotDraft {
    let product = product_name(context);
    let selling_point = selling_points(context)
        .into_iter()
        .find(|item| {
            item.to_lowercase().contains("fabric") || item.to_lowercase().contains("cotton")
        })
        .unwrap_or_else(|| "fabric texture".to_string());
    let research_text = research
        .first()
        .map(|(_, implication)| implication.clone())
        .unwrap_or_else(|| "Use tactile product proof.".to_string());
    let source_research_report_ids = research
        .first()
        .map(|(id, _)| vec![id.clone()])
        .unwrap_or_default();
    let accepted_memory = accepted_memory(context);
    let source_memory_ids = accepted_memory
        .iter()
        .take(2)
        .map(|memory| memory.id.clone())
        .collect::<Vec<_>>();
    let revision_note = if input.revision_note.trim().is_empty() {
        "Make the shot more specific using current project context.".to_string()
    } else {
        input.revision_note.trim().to_string()
    };

    ShotDraft {
        shot_number: original.shot_number,
        duration_seconds: original.duration_seconds,
        visual_description: format!(
            "Regenerated shot {} turns the note '{}' into a clearer macro fabric insert for {product}.",
            original.shot_number, revision_note
        ),
        model_action: "Model guides attention to the garment surface with a small sleeve or collar adjustment."
            .to_string(),
        garment_focus: format!("fabric texture and {selling_point}"),
        camera_movement: "controlled macro slide".to_string(),
        scene: scene_direction(context),
        lighting: "grazing light that reveals fabric surface".to_string(),
        transition: "match cut back into model movement".to_string(),
        subtitle_or_text: "Texture in motion".to_string(),
        rationale: format!(
            "Regenerated from user note; preserves timing while applying research guidance: {research_text}."
        ),
        source_memory_ids,
        source_research_report_ids,
        generation_context: json!({
            "stage": "0.16",
            "source": "shot_regeneration",
            "revision_note": revision_note,
        }),
    }
}

fn accepted_memory(context: &AgentProjectContext) -> Vec<MemoryEntry> {
    context
        .project_memory
        .iter()
        .filter(|memory| memory.status == "accepted")
        .cloned()
        .collect()
}

fn selling_points(context: &AgentProjectContext) -> Vec<String> {
    context
        .latest_product_understanding
        .as_ref()
        .and_then(|understanding| understanding.selling_points_json.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(str::trim).map(ToString::to_string))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .into_iter()
        .chain(std::iter::once("hero garment silhouette".to_string()))
        .collect()
}

fn constraints(context: &AgentProjectContext) -> Vec<String> {
    let constraints = context
        .latest_product_understanding
        .as_ref()
        .and_then(|understanding| understanding.constraints_json.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(str::trim).map(ToString::to_string))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if constraints.is_empty() {
        vec!["no known constraints".to_string()]
    } else {
        constraints
    }
}

fn research_implications(
    repo: &Repository<'_>,
    project_id: &str,
) -> JoiResult<Vec<(String, String)>> {
    let reports = repo.list_research_reports(project_id)?;
    let mut implications = Vec::new();
    for report in reports {
        let Some(findings) = report.findings_json.as_array() else {
            continue;
        };
        for finding in findings {
            if let Some(value) = string_field(finding, "creative_implication")
                .or_else(|| string_field(finding, "insight"))
            {
                implications.push((report.id.clone(), value));
            }
        }
    }
    Ok(implications)
}

fn product_name(context: &AgentProjectContext) -> String {
    context
        .latest_product_understanding
        .as_ref()
        .map(|understanding| understanding.product_name.trim())
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| context.project.title.clone())
}

fn visual_style(context: &AgentProjectContext) -> String {
    context
        .latest_creative_direction
        .as_ref()
        .map(|direction| direction.visual_style.trim())
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| "clean fashion advertising".to_string())
}

fn scene_direction(context: &AgentProjectContext) -> String {
    context
        .latest_creative_direction
        .as_ref()
        .map(|direction| direction.scene_direction.trim())
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| "minimal studio set".to_string())
}

fn lighting(context: &AgentProjectContext) -> String {
    let style = visual_style(context).to_lowercase();
    if style.contains("warm") {
        "warm soft studio light".to_string()
    } else {
        "soft directional fashion light".to_string()
    }
}

fn shot_arc_label(index: usize, shot_count: usize) -> &'static str {
    if index == 0 {
        "opening"
    } else if index == 1 {
        "proof"
    } else if index + 1 == shot_count {
        "closing"
    } else if index % 2 == 0 {
        "movement"
    } else {
        "styling"
    }
}

fn generation_goal(context: &AgentProjectContext, input: &StoryboardGenerationInput) -> String {
    if input.user_direction.trim().is_empty() {
        format!("Generate storyboard for {}", context.project.title)
    } else {
        input.user_direction.trim().to_string()
    }
}

fn regeneration_goal(context: &AgentProjectContext, input: &ShotRegenerationInput) -> String {
    if input.revision_note.trim().is_empty() {
        format!("Regenerate selected shot for {}", context.project.title)
    } else {
        input.revision_note.trim().to_string()
    }
}

fn build_generation_plan_json(
    input: &StoryboardGenerationInput,
    duration_seconds: i64,
    shot_count: i64,
) -> Value {
    json!([
        {
            "role": "planner",
            "stage": "0.16",
            "title": "Read storyboard context",
            "status": "completed"
        },
        {
            "role": "planner",
            "stage": "0.16",
            "title": "Plan duration and shot count",
            "duration_seconds": duration_seconds,
            "shot_count": shot_count,
            "status": "completed"
        },
        {
            "role": "storyboard_writer",
            "stage": "0.16",
            "title": "Draft structured shots",
            "user_direction": input.user_direction.trim(),
            "status": "completed"
        }
    ])
}

fn build_regeneration_plan_json(input: &ShotRegenerationInput) -> Value {
    json!([
        {
            "role": "planner",
            "stage": "0.16",
            "title": "Read selected shot",
            "shot_id": input.shot_id,
            "status": "completed"
        },
        {
            "role": "storyboard_writer",
            "stage": "0.16",
            "title": "Apply revision note",
            "revision_note": input.revision_note.trim(),
            "status": "completed"
        }
    ])
}

fn build_generation_event_specs(
    context: &AgentProjectContext,
    input: &StoryboardGenerationInput,
    storyboard: &Storyboard,
    shots: &[StoryboardShotView],
    duration_seconds: i64,
    shot_count: i64,
    research_count: usize,
) -> Vec<StoryboardEventSpec> {
    let total_duration = shots
        .iter()
        .map(|item| item.shot.duration_seconds)
        .sum::<i64>();
    vec![
        StoryboardEventSpec {
            sequence_number: 1,
            role: "planner",
            event_type: "storyboard_context_read",
            message: format!("Read storyboard context for {}.", context.project.title),
            payload_json: json!({
                "brand_name": context.brand.name,
                "project_title": context.project.title,
                "product_name": product_name(context),
                "accepted_memory_count": accepted_memory(context).len(),
                "research_implication_count": research_count,
            }),
        },
        StoryboardEventSpec {
            sequence_number: 2,
            role: "planner",
            event_type: "duration_plan_created",
            message: format!("Planned {shot_count} shot(s) across {duration_seconds} seconds."),
            payload_json: json!({
                "duration_seconds": duration_seconds,
                "shot_count": shot_count,
                "preferred_duration_seconds": input.preferred_duration_seconds,
                "preferred_shot_count": input.preferred_shot_count,
            }),
        },
        StoryboardEventSpec {
            sequence_number: 3,
            role: "storyboard_writer",
            event_type: "shot_requirements_mapped",
            message:
                "Mapped product, memory, research, and creative direction into shot requirements."
                    .to_string(),
            payload_json: json!({
                "selling_points": selling_points(context),
                "constraints": constraints(context),
            }),
        },
        StoryboardEventSpec {
            sequence_number: 4,
            role: "storyboard_writer",
            event_type: "shots_drafted",
            message: format!("Drafted {} structured shot(s).", shots.len()),
            payload_json: json!({
                "shot_ids": shots.iter().map(|item| item.shot.id.clone()).collect::<Vec<_>>()
            }),
        },
        StoryboardEventSpec {
            sequence_number: 5,
            role: "reviewer",
            event_type: "duration_consistency_checked",
            message: format!("Checked total shot duration: {total_duration} seconds."),
            payload_json: json!({
                "total_duration_seconds": total_duration,
                "matches_storyboard_duration": total_duration == duration_seconds,
            }),
        },
        StoryboardEventSpec {
            sequence_number: 6,
            role: "storyboard_writer",
            event_type: "storyboard_saved",
            message: format!("Saved storyboard {}.", storyboard.id),
            payload_json: json!({
                "storyboard_id": storyboard.id,
                "shot_count": shots.len(),
            }),
        },
    ]
}

fn build_regeneration_event_specs(
    context: &AgentProjectContext,
    input: &ShotRegenerationInput,
    shot: &StoryboardShotView,
) -> Vec<StoryboardEventSpec> {
    vec![
        StoryboardEventSpec {
            sequence_number: 1,
            role: "planner",
            event_type: "shot_context_read",
            message: format!("Read selected shot context for {}.", context.project.title),
            payload_json: json!({
                "storyboard_id": input.storyboard_id,
                "shot_id": input.shot_id,
                "shot_number": shot.shot.shot_number,
            }),
        },
        StoryboardEventSpec {
            sequence_number: 2,
            role: "storyboard_writer",
            event_type: "revision_instruction_applied",
            message: "Applied selected-shot revision instruction.".to_string(),
            payload_json: json!({
                "revision_note": input.revision_note.trim(),
                "visual_description": shot.visual_description,
            }),
        },
        StoryboardEventSpec {
            sequence_number: 3,
            role: "reviewer",
            event_type: "shot_duration_preserved",
            message: format!(
                "Preserved shot duration at {} seconds.",
                shot.shot.duration_seconds
            ),
            payload_json: json!({
                "duration_seconds": shot.shot.duration_seconds,
            }),
        },
        StoryboardEventSpec {
            sequence_number: 4,
            role: "storyboard_writer",
            event_type: "shot_saved",
            message: format!("Saved regenerated shot {}.", shot.shot.id),
            payload_json: json!({
                "shot_id": shot.shot.id,
                "garment_focus": shot.garment_focus,
            }),
        },
    ]
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}
