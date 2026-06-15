use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::agent_context::{build_project_context, AgentProjectContext};
use crate::error::{JoiError, JoiResult};
use crate::models::{AgentRun, AgentRunEvent, MemoryEntry, PromptPackage, Shot};
use crate::repositories::{AgentRunCreate, AgentRunEventCreate, PromptPackageCreate, Repository};

const PROMPT_ADAPTER_ROLES: [&str; 3] = ["planner", "prompt_adapter", "reviewer"];
const PROMPT_PARAMETERS_FORMAT: &str = "joi.prompt_package_parameters.v1";
const VIDEO_NEGATIVE: &str = "low resolution, distorted garment shape, warped hands, extra limbs, unreadable text, heavy flicker, abrupt camera jitter, fabric texture lost, off-brand styling";
const IMAGE_NEGATIVE: &str = "low resolution, distorted hands, extra fingers, incorrect garment construction, warped seams, blurry fabric texture, messy background, harsh shadows, over-stylized illustration, off-brand colors";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptAdapterProfile {
    pub id: String,
    pub display_name: String,
    pub modality: String,
    pub default_negative_prompt: String,
    pub required_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptGenerationInput {
    pub project_id: String,
    pub shot_ids: Vec<String>,
    pub image_brief: String,
    pub target_platforms: Vec<String>,
    pub user_direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCompletenessCheck {
    pub field: String,
    pub label: String,
    pub present: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptPackageView {
    pub package: PromptPackage,
    pub adapter_display_name: String,
    pub completeness: Vec<PromptCompletenessCheck>,
    pub missing_fields: Vec<String>,
    pub copy_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptGenerationResult {
    pub packages: Vec<PromptPackageView>,
    pub agent_run: AgentRun,
    pub agent_events: Vec<AgentRunEvent>,
}

#[derive(Debug, Clone)]
struct ResearchImplication {
    report_id: String,
    text: String,
}

pub fn prompt_adapter_profiles() -> Vec<PromptAdapterProfile> {
    vec![
        profile(
            "jimeng_video",
            "Jimeng Video",
            "video",
            VIDEO_NEGATIVE,
            video_fields(),
        ),
        profile(
            "grok_video",
            "Grok Video",
            "video",
            VIDEO_NEGATIVE,
            video_fields(),
        ),
        profile(
            "banana_2_image",
            "Banana 2 Image",
            "image",
            IMAGE_NEGATIVE,
            image_fields(),
        ),
        profile(
            "jimeng_image",
            "Jimeng Image",
            "image",
            IMAGE_NEGATIVE,
            image_fields(),
        ),
        profile(
            "gpt_image_2",
            "GPT Image 2",
            "image",
            IMAGE_NEGATIVE,
            image_fields(),
        ),
    ]
}

pub fn generate_prompt_packages(
    repo: &Repository<'_>,
    input: PromptGenerationInput,
    hermes_version: String,
) -> JoiResult<PromptGenerationResult> {
    let context = build_project_context(repo, &input.project_id)?;
    let targets = resolve_target_profiles(&input.target_platforms)?;
    validate_generation_request(&input, &targets)?;
    let research = research_implications(repo, &input.project_id)?;
    let accepted_memory = accepted_memory(&context);

    let mut packages = Vec::new();
    for profile in &targets {
        if profile.modality == "video" {
            for shot_id in &input.shot_ids {
                let shot = repo.get_shot(shot_id)?;
                packages.push(create_video_prompt(
                    repo,
                    &context,
                    profile,
                    &input,
                    &shot,
                    &accepted_memory,
                    &research,
                )?);
            }
        } else {
            packages.push(create_image_prompt(
                repo,
                &context,
                profile,
                &input,
                &accepted_memory,
                &research,
            )?);
        }
    }

    let agent_run = repo.create_agent_run(AgentRunCreate {
        project_id: input.project_id.clone(),
        user_goal: prompt_generation_goal(&context, &input),
        status: "completed".to_string(),
        runtime_kind: "hermes_core".to_string(),
        runtime_mode: "local_prompt_adapter_bridge".to_string(),
        runtime_version: hermes_version,
        roles_json: json!(PROMPT_ADAPTER_ROLES),
        plan_json: build_prompt_plan_json(&input, &targets),
        result_summary: format!(
            "Generated {} prompt package(s) for {}.",
            packages.len(),
            context.project.title
        ),
    })?;

    let agent_events = create_prompt_events(repo, &agent_run.id, &context, &input, &packages)?;
    Ok(PromptGenerationResult {
        packages,
        agent_run,
        agent_events,
    })
}

pub fn prompt_package_view(package: PromptPackage) -> PromptPackageView {
    let adapter_display_name = string_field(&package.parameters_json, "adapter_display_name")
        .unwrap_or_else(|| package.platform.clone());
    let missing_fields = string_array_field(&package.parameters_json, "missing_fields");
    let completeness = completeness_from_parameters(&package);
    let copy_text = format!(
        "{}\n\nNegative prompt:\n{}",
        package.prompt_text, package.negative_prompt
    );

    PromptPackageView {
        package,
        adapter_display_name,
        completeness,
        missing_fields,
        copy_text,
    }
}

fn profile(
    id: &str,
    display_name: &str,
    modality: &str,
    default_negative_prompt: &str,
    required_fields: Vec<String>,
) -> PromptAdapterProfile {
    PromptAdapterProfile {
        id: id.to_string(),
        display_name: display_name.to_string(),
        modality: modality.to_string(),
        default_negative_prompt: default_negative_prompt.to_string(),
        required_fields,
    }
}

fn video_fields() -> Vec<String> {
    [
        "subject", "scene", "action", "camera", "material", "lighting", "style",
    ]
    .into_iter()
    .map(ToString::to_string)
    .collect()
}

fn image_fields() -> Vec<String> {
    [
        "subject", "scene", "garment", "material", "lighting", "style",
    ]
    .into_iter()
    .map(ToString::to_string)
    .collect()
}

fn resolve_target_profiles(ids: &[String]) -> JoiResult<Vec<PromptAdapterProfile>> {
    if ids.is_empty() {
        return Err(JoiError::Validation(
            "At least one prompt target platform is required".to_string(),
        ));
    }

    let profiles = prompt_adapter_profiles();
    ids.iter()
        .map(|id| {
            profiles
                .iter()
                .find(|profile| profile.id == *id)
                .cloned()
                .ok_or_else(|| JoiError::Validation(format!("unknown prompt adapter: {id}")))
        })
        .collect()
}

fn validate_generation_request(
    input: &PromptGenerationInput,
    targets: &[PromptAdapterProfile],
) -> JoiResult<()> {
    if targets.iter().any(|profile| profile.modality == "video") && input.shot_ids.is_empty() {
        return Err(JoiError::Validation(
            "Video prompt generation requires at least one shot".to_string(),
        ));
    }
    if targets.iter().any(|profile| profile.modality == "image")
        && input.image_brief.trim().is_empty()
    {
        return Err(JoiError::Validation(
            "Image prompt generation requires an image brief".to_string(),
        ));
    }
    Ok(())
}

fn create_video_prompt(
    repo: &Repository<'_>,
    context: &AgentProjectContext,
    profile: &PromptAdapterProfile,
    input: &PromptGenerationInput,
    shot: &Shot,
    accepted_memory: &[MemoryEntry],
    research: &[ResearchImplication],
) -> JoiResult<PromptPackageView> {
    let product = product_name(context);
    let style = visual_style(context);
    let garment_focus = string_field(&shot.metadata_json, "garment_focus")
        .unwrap_or_else(|| selling_points(context).join(", "));
    let transition = string_field(&shot.metadata_json, "transition").unwrap_or_default();
    let constraints = constraints(context);
    let memory_text = memory_instruction(accepted_memory);
    let prompt_text = if profile.id == "jimeng_video" {
        format!(
            "Jimeng video prompt:\n{}s fashion advertising shot. Subject: {}. Scene: {}. Action: {}. Camera: {}. Garment focus: {}. Lighting: {}. Style: {}. On-screen text: {}. Transition: {}. {}",
            shot.duration_seconds,
            product,
            shot.scene,
            shot.model_action,
            shot.camera_movement,
            garment_focus,
            shot.lighting,
            style,
            shot.subtitle_or_voiceover,
            transition,
            memory_text
        )
    } else {
        format!(
            "Grok video prompt:\nCreate a {} second fashion ad shot for {}. Show the model action: {} in {} with a {} camera move and {}. Prioritize {}, natural garment movement, and a polished {} style. Optional text: {}. Avoid {}.",
            shot.duration_seconds,
            product,
            shot.model_action,
            shot.scene,
            shot.camera_movement,
            shot.lighting,
            garment_focus,
            style,
            shot.subtitle_or_voiceover,
            constraints.join(", ")
        )
    };
    let fields = vec![
        ("subject", !product.trim().is_empty()),
        ("scene", !shot.scene.trim().is_empty()),
        ("action", !shot.model_action.trim().is_empty()),
        ("camera", !shot.camera_movement.trim().is_empty()),
        ("material", !garment_focus.trim().is_empty()),
        ("lighting", !shot.lighting.trim().is_empty()),
        ("style", !style.trim().is_empty()),
    ];
    let missing_fields = fields
        .iter()
        .filter_map(|(field, present)| (!present).then_some((*field).to_string()))
        .collect::<Vec<_>>();
    let parameters_json = prompt_parameters(
        profile,
        "storyboard_shot",
        Some(shot),
        "",
        &missing_fields,
        input,
        accepted_memory,
        research,
        &prompt_text,
    );
    let package = repo.create_prompt_package(PromptPackageCreate {
        project_id: input.project_id.clone(),
        shot_id: Some(shot.id.clone()),
        platform: profile.id.clone(),
        modality: profile.modality.clone(),
        prompt_text,
        negative_prompt: profile.default_negative_prompt.clone(),
        parameters_json,
    })?;
    Ok(prompt_package_view(package))
}

fn create_image_prompt(
    repo: &Repository<'_>,
    context: &AgentProjectContext,
    profile: &PromptAdapterProfile,
    input: &PromptGenerationInput,
    accepted_memory: &[MemoryEntry],
    research: &[ResearchImplication],
) -> JoiResult<PromptPackageView> {
    let product = product_name(context);
    let style = visual_style(context);
    let scene = scene_from_image_brief(input).unwrap_or_else(|| scene_direction(context));
    let material =
        material_from_image_brief(input).unwrap_or_else(|| selling_points(context).join(", "));
    let lighting = lighting_from_image_brief(input).unwrap_or_else(|| lighting(context));
    let prompt_text = match profile.id.as_str() {
        "banana_2_image" => format!(
            "Banana 2 image prompt:\nFashion model product photo for {}, {}, {}, {} visible, {}, {}, premium ecommerce-ready composition, natural hands, accurate garment construction.",
            product,
            input.image_brief.trim(),
            scene,
            material,
            style,
            lighting
        ),
        "jimeng_image" => format!(
            "Jimeng image prompt:\n服装广告模拍图，{}，{}，{}，突出 {}，{}，{}，服装结构准确，面料纹理清晰。",
            product,
            input.image_brief.trim(),
            scene,
            material,
            lighting,
            style
        ),
        _ => format!(
            "GPT Image 2 prompt:\nCreate a realistic fashion campaign image of a model wearing {} in {}. Image brief: {}. Emphasize {}, accurate garment construction, {}, {}, and natural pose.",
            product,
            scene,
            input.image_brief.trim(),
            material,
            lighting,
            style
        ),
    };
    let fields = vec![
        ("subject", !product.trim().is_empty()),
        ("scene", !scene.trim().is_empty()),
        (
            "garment",
            !product.trim().is_empty() || !input.image_brief.trim().is_empty(),
        ),
        ("material", !material.trim().is_empty()),
        ("lighting", !lighting.trim().is_empty()),
        ("style", !style.trim().is_empty()),
    ];
    let missing_fields = fields
        .iter()
        .filter_map(|(field, present)| (!present).then_some((*field).to_string()))
        .collect::<Vec<_>>();
    let parameters_json = prompt_parameters(
        profile,
        "image_brief",
        None,
        input.image_brief.trim(),
        &missing_fields,
        input,
        accepted_memory,
        research,
        &prompt_text,
    );
    let package = repo.create_prompt_package(PromptPackageCreate {
        project_id: input.project_id.clone(),
        shot_id: None,
        platform: profile.id.clone(),
        modality: profile.modality.clone(),
        prompt_text,
        negative_prompt: profile.default_negative_prompt.clone(),
        parameters_json,
    })?;
    Ok(prompt_package_view(package))
}

fn prompt_parameters(
    profile: &PromptAdapterProfile,
    source_type: &str,
    shot: Option<&Shot>,
    image_brief: &str,
    missing_fields: &[String],
    input: &PromptGenerationInput,
    accepted_memory: &[MemoryEntry],
    research: &[ResearchImplication],
    prompt_text: &str,
) -> Value {
    json!({
        "format_version": PROMPT_PARAMETERS_FORMAT,
        "adapter_id": profile.id,
        "adapter_display_name": profile.display_name,
        "source_type": source_type,
        "source_storyboard_id": shot.map(|item| item.storyboard_id.clone()),
        "source_shot_id": shot.map(|item| item.id.clone()),
        "source_image_brief": image_brief,
        "required_fields": profile.required_fields,
        "missing_fields": missing_fields,
        "copy_blocks": {
            "main_prompt": prompt_text,
            "negative_prompt": profile.default_negative_prompt,
            "notes": "Review garment accuracy and platform fit before generation."
        },
        "generation_context": {
            "stage": "0.17",
            "user_direction": input.user_direction.trim(),
            "accepted_memory_ids": accepted_memory.iter().map(|memory| memory.id.clone()).collect::<Vec<_>>(),
            "research_report_ids": research.iter().map(|item| item.report_id.clone()).collect::<Vec<_>>(),
            "research_implications": research.iter().map(|item| item.text.clone()).collect::<Vec<_>>()
        }
    })
}

fn completeness_from_parameters(package: &PromptPackage) -> Vec<PromptCompletenessCheck> {
    let required_fields = string_array_field(&package.parameters_json, "required_fields");
    let missing_fields = string_array_field(&package.parameters_json, "missing_fields");
    required_fields
        .into_iter()
        .map(|field| {
            let present = !missing_fields.iter().any(|missing| missing == &field);
            PromptCompletenessCheck {
                label: field_label(&field).to_string(),
                message: if present {
                    format!("{} is present.", field_label(&field))
                } else {
                    format!("{} is missing.", field_label(&field))
                },
                field,
                present,
            }
        })
        .collect()
}

fn create_prompt_events(
    repo: &Repository<'_>,
    agent_run_id: &str,
    context: &AgentProjectContext,
    input: &PromptGenerationInput,
    packages: &[PromptPackageView],
) -> JoiResult<Vec<AgentRunEvent>> {
    let missing_total = packages
        .iter()
        .map(|package| package.missing_fields.len())
        .sum::<usize>();
    let specs = vec![
        (
            1,
            "planner",
            "prompt_context_read",
            format!("Read prompt context for {}.", context.project.title),
            json!({
                "brand_name": context.brand.name,
                "project_title": context.project.title,
                "shot_count": input.shot_ids.len(),
            }),
        ),
        (
            2,
            "planner",
            "prompt_targets_resolved",
            format!(
                "Resolved {} prompt target(s).",
                input.target_platforms.len()
            ),
            json!({ "target_platforms": input.target_platforms }),
        ),
        (
            3,
            "prompt_adapter",
            "prompts_drafted",
            format!("Drafted {} prompt package(s).", packages.len()),
            json!({
                "prompt_package_ids": packages.iter().map(|item| item.package.id.clone()).collect::<Vec<_>>()
            }),
        ),
        (
            4,
            "reviewer",
            "prompt_completeness_checked",
            format!("Checked prompt completeness with {missing_total} missing field(s)."),
            json!({ "missing_field_count": missing_total }),
        ),
        (
            5,
            "prompt_adapter",
            "prompt_packages_saved",
            "Saved generated prompt packages.".to_string(),
            json!({ "package_count": packages.len() }),
        ),
    ];

    let mut events = Vec::new();
    for (sequence_number, role, event_type, message, payload_json) in specs {
        events.push(repo.create_agent_run_event(AgentRunEventCreate {
            agent_run_id: agent_run_id.to_string(),
            sequence_number,
            role: role.to_string(),
            event_type: event_type.to_string(),
            message,
            payload_json,
        })?);
    }
    Ok(events)
}

fn prompt_generation_goal(context: &AgentProjectContext, input: &PromptGenerationInput) -> String {
    if input.user_direction.trim().is_empty() {
        format!("Generate prompt packages for {}", context.project.title)
    } else {
        input.user_direction.trim().to_string()
    }
}

fn build_prompt_plan_json(
    input: &PromptGenerationInput,
    targets: &[PromptAdapterProfile],
) -> Value {
    json!([
        {
            "role": "planner",
            "stage": "0.17",
            "title": "Read prompt context",
            "status": "completed"
        },
        {
            "role": "planner",
            "stage": "0.17",
            "title": "Resolve adapter targets",
            "target_platforms": input.target_platforms,
            "modalities": targets.iter().map(|profile| profile.modality.clone()).collect::<Vec<_>>(),
            "status": "completed"
        },
        {
            "role": "prompt_adapter",
            "stage": "0.17",
            "title": "Generate prompt packages",
            "shot_ids": input.shot_ids,
            "image_brief_present": !input.image_brief.trim().is_empty(),
            "status": "completed"
        }
    ])
}

fn accepted_memory(context: &AgentProjectContext) -> Vec<MemoryEntry> {
    context
        .project_memory
        .iter()
        .filter(|memory| memory.status == "accepted")
        .cloned()
        .collect()
}

fn memory_instruction(accepted_memory: &[MemoryEntry]) -> String {
    accepted_memory
        .first()
        .map(|memory| memory.content.clone())
        .unwrap_or_else(|| "Keep the garment clearly visible.".to_string())
}

fn research_implications(
    repo: &Repository<'_>,
    project_id: &str,
) -> JoiResult<Vec<ResearchImplication>> {
    let reports = repo.list_research_reports(project_id)?;
    let mut implications = Vec::new();
    for report in reports {
        let Some(findings) = report.findings_json.as_array() else {
            continue;
        };
        for finding in findings {
            if let Some(text) = string_field(finding, "creative_implication")
                .or_else(|| string_field(finding, "insight"))
            {
                implications.push(ResearchImplication {
                    report_id: report.id.clone(),
                    text,
                });
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

fn selling_points(context: &AgentProjectContext) -> Vec<String> {
    let points = context
        .latest_product_understanding
        .as_ref()
        .and_then(|understanding| understanding.selling_points_json.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(str::trim).map(ToString::to_string))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if points.is_empty() {
        vec!["hero garment silhouette".to_string()]
    } else {
        points
    }
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

fn scene_from_image_brief(input: &PromptGenerationInput) -> Option<String> {
    string_contains_hint(&input.image_brief, "studio").map(|_| input.image_brief.trim().to_string())
}

fn material_from_image_brief(input: &PromptGenerationInput) -> Option<String> {
    string_contains_hint(&input.image_brief, "texture")
        .or_else(|| string_contains_hint(&input.image_brief, "cotton"))
        .map(|_| input.image_brief.trim().to_string())
}

fn lighting_from_image_brief(input: &PromptGenerationInput) -> Option<String> {
    string_contains_hint(&input.image_brief, "light")
        .or_else(|| string_contains_hint(&input.image_brief, "warm"))
        .map(|_| input.image_brief.trim().to_string())
}

fn string_contains_hint(value: &str, hint: &str) -> Option<()> {
    value.to_lowercase().contains(hint).then_some(())
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn string_array_field(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(str::trim).map(ToString::to_string))
        .filter(|value| !value.is_empty())
        .collect()
}

fn field_label(field: &str) -> &str {
    match field {
        "subject" => "Subject",
        "scene" => "Scene",
        "action" => "Action",
        "camera" => "Camera",
        "material" => "Material",
        "lighting" => "Lighting",
        "style" => "Style",
        "garment" => "Garment",
        _ => "Field",
    }
}
