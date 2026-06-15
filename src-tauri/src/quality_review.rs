use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::agent_context::build_project_context;
use crate::error::{JoiError, JoiResult};
use crate::models::{
    AgentRun, AgentRunEvent, CreativeDirection, ProductUnderstanding, PromptPackage, QualityReview,
    Shot,
};
use crate::prompt_adapter::prompt_package_view;
use crate::repositories::{
    AgentRunCreate, AgentRunEventCreate, PromptPackageUpdate, QualityReviewCreate, Repository,
    ShotUpdate, StoryboardWithShots,
};

const REVIEW_ROLES: &[&str] = &[
    "planner",
    "reviewer",
    "storyboard_writer",
    "prompt_adapter",
    "memory_curator",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QualityReviewGenerationInput {
    pub project_id: String,
    pub user_direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QualityReviewCheck {
    pub id: String,
    pub category: String,
    pub title: String,
    pub status: String,
    pub severity: String,
    pub source_type: String,
    pub source_id: String,
    pub message: String,
    pub evidence: Vec<String>,
    pub suggestion_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QualityReviewSuggestion {
    pub id: String,
    pub target_type: String,
    pub target_id: String,
    pub field: String,
    pub current_value: String,
    pub suggested_value: String,
    pub rationale: String,
    pub status: String,
    pub check_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityReviewSuggestionStatus {
    Pending,
    Applied,
    Rejected,
}

impl QualityReviewSuggestionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Applied => "applied",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReviewGenerationResult {
    pub review: QualityReview,
    pub checks: Vec<QualityReviewCheck>,
    pub suggestions: Vec<QualityReviewSuggestion>,
    pub agent_run: AgentRun,
    pub agent_events: Vec<AgentRunEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApplyReviewSuggestionInput {
    pub review_id: String,
    pub suggestion_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyReviewSuggestionResult {
    pub updated_review: QualityReview,
    pub suggestion: QualityReviewSuggestion,
    pub applied_target_type: String,
    pub applied_target_id: String,
    pub agent_run: AgentRun,
    pub agent_events: Vec<AgentRunEvent>,
}

pub fn generate_quality_review(
    repo: &Repository<'_>,
    input: QualityReviewGenerationInput,
    hermes_version: String,
) -> JoiResult<QualityReviewGenerationResult> {
    let context = build_project_context(repo, &input.project_id)?;
    let storyboards = repo.list_storyboards_with_typed_shots(&input.project_id)?;
    let prompt_packages = repo.list_prompt_packages(&input.project_id)?;
    let product_understandings = repo.list_product_understandings(&input.project_id)?;
    let creative_directions = repo.list_creative_directions(&input.project_id)?;

    let product_terms = build_product_terms(&context.brand.description, &product_understandings);
    let brand_terms = build_brand_terms(
        &context.brand.name,
        &context.brand.description,
        &creative_directions,
    );

    let mut checks = Vec::new();
    let mut suggestions = Vec::new();

    review_storyboards(
        &context.project.id,
        context.project.duration_seconds,
        &storyboards,
        &product_terms,
        &brand_terms,
        &mut checks,
        &mut suggestions,
    );
    review_prompts(
        &prompt_packages,
        &product_terms,
        &brand_terms,
        &mut checks,
        &mut suggestions,
    );

    link_suggestions_to_checks(&mut checks, &suggestions);
    let score = review_score(&checks);
    let summary = review_summary(score, &checks, &suggestions);
    let review = repo.create_quality_review(QualityReviewCreate {
        project_id: input.project_id.clone(),
        summary: summary.clone(),
        score,
        checklist_json: json!(checks),
        suggestions_json: json!(suggestions),
    })?;

    let agent_run = repo.create_agent_run(AgentRunCreate {
        project_id: input.project_id.clone(),
        user_goal: quality_review_goal(&context.project.title, &input),
        status: "completed".to_string(),
        runtime_kind: "hermes_core".to_string(),
        runtime_mode: "local_quality_review_bridge".to_string(),
        runtime_version: hermes_version,
        roles_json: json!(REVIEW_ROLES),
        plan_json: quality_review_plan_json(&input),
        result_summary: summary,
    })?;

    let agent_events = create_quality_review_events(repo, &agent_run.id, &checks, &suggestions)?;

    Ok(QualityReviewGenerationResult {
        review,
        checks,
        suggestions,
        agent_run,
        agent_events,
    })
}

pub fn apply_quality_review_suggestion(
    repo: &Repository<'_>,
    input: ApplyReviewSuggestionInput,
    hermes_version: String,
) -> JoiResult<ApplyReviewSuggestionResult> {
    let review = repo.get_quality_review(&input.review_id)?;
    let mut suggestions = suggestions_from_review(&review)?;
    let index = suggestions
        .iter()
        .position(|suggestion| suggestion.id == input.suggestion_id)
        .ok_or_else(|| {
            JoiError::NotFound(format!("quality review suggestion {}", input.suggestion_id))
        })?;

    if suggestions[index].status != QualityReviewSuggestionStatus::Pending.as_str() {
        return Err(JoiError::Validation(format!(
            "quality review suggestion {} is not pending",
            input.suggestion_id
        )));
    }

    apply_supported_target(repo, &suggestions[index])?;
    suggestions[index].status = QualityReviewSuggestionStatus::Applied.as_str().to_string();
    let applied = suggestions[index].clone();
    let updated_review =
        repo.update_quality_review_suggestions(&review.id, suggestion_to_value(&suggestions))?;

    let agent_run = repo.create_agent_run(AgentRunCreate {
        project_id: review.project_id.clone(),
        user_goal: format!("Apply quality review suggestion {}.", applied.id),
        status: "completed".to_string(),
        runtime_kind: "hermes_core".to_string(),
        runtime_mode: "local_quality_iteration_bridge".to_string(),
        runtime_version: hermes_version,
        roles_json: json!(["reviewer", "storyboard_writer", "prompt_adapter"]),
        plan_json: json!([
            {
                "role": "reviewer",
                "title": "Validate selected review suggestion",
                "suggestion_id": applied.id
            },
            {
                "role": "storyboard_writer",
                "title": "Apply supported target update"
            }
        ]),
        result_summary: format!(
            "Applied review suggestion {} to {} {}.",
            applied.id, applied.target_type, applied.target_id
        ),
    })?;
    let agent_events = vec![repo.create_agent_run_event(AgentRunEventCreate {
        agent_run_id: agent_run.id.clone(),
        sequence_number: 1,
        role: "reviewer".to_string(),
        event_type: "suggestion_applied".to_string(),
        message: format!("Applied suggestion {}.", applied.id),
        payload_json: json!({ "suggestion": applied }),
    })?];

    Ok(ApplyReviewSuggestionResult {
        updated_review,
        applied_target_type: applied.target_type.clone(),
        applied_target_id: applied.target_id.clone(),
        suggestion: applied,
        agent_run,
        agent_events,
    })
}

fn suggestions_from_review(review: &QualityReview) -> JoiResult<Vec<QualityReviewSuggestion>> {
    serde_json::from_value(review.suggestions_json.clone()).map_err(|err| {
        JoiError::Validation(format!("quality review suggestions are malformed: {err}"))
    })
}

fn suggestion_to_value(suggestions: &[QualityReviewSuggestion]) -> Value {
    json!(suggestions)
}

fn apply_supported_target(
    repo: &Repository<'_>,
    suggestion: &QualityReviewSuggestion,
) -> JoiResult<()> {
    match (suggestion.target_type.as_str(), suggestion.field.as_str()) {
        ("shot", "description") => apply_shot_description(repo, suggestion),
        ("prompt_package", "prompt_text") => apply_prompt_text(repo, suggestion),
        (target_type, field) => Err(JoiError::Validation(format!(
            "review suggestion target is not supported: {target_type}.{field}"
        ))),
    }
}

fn apply_shot_description(
    repo: &Repository<'_>,
    suggestion: &QualityReviewSuggestion,
) -> JoiResult<()> {
    let shot = repo.get_shot(&suggestion.target_id)?;
    if shot.is_locked {
        return Err(JoiError::Validation(
            "Locked shots cannot be updated from review suggestions".to_string(),
        ));
    }

    let garment_focus = metadata_string_or(&shot, "garment_focus", &shot.description);
    let transition = metadata_string_or(&shot, "transition", "");
    repo.update_shot(ShotUpdate {
        id: shot.id,
        duration_seconds: shot.duration_seconds,
        visual_description: suggestion.suggested_value.clone(),
        model_action: shot.model_action,
        garment_focus,
        camera_movement: shot.camera_movement,
        scene: shot.scene,
        lighting: shot.lighting,
        transition,
        subtitle_or_text: shot.subtitle_or_voiceover,
        rationale: shot.rationale,
        is_locked: shot.is_locked,
    })?;
    Ok(())
}

fn apply_prompt_text(repo: &Repository<'_>, suggestion: &QualityReviewSuggestion) -> JoiResult<()> {
    let package = repo.get_prompt_package(&suggestion.target_id)?;
    if package.is_locked {
        return Err(JoiError::Validation(
            "Locked prompt packages cannot be updated from review suggestions".to_string(),
        ));
    }

    repo.update_prompt_package(PromptPackageUpdate {
        id: package.id,
        prompt_text: suggestion.suggested_value.clone(),
        negative_prompt: package.negative_prompt,
        parameters_json: package.parameters_json,
        is_locked: package.is_locked,
    })?;
    Ok(())
}

fn metadata_string_or(shot: &Shot, field: &str, fallback: &str) -> String {
    shot.metadata_json
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn build_product_terms(
    brand_description: &str,
    product_understandings: &[ProductUnderstanding],
) -> Vec<String> {
    let mut terms = Vec::new();
    if let Some(latest) = product_understandings.last() {
        push_term(&mut terms, &latest.product_name);
        push_term(&mut terms, &latest.category);
        for value in string_array(&latest.selling_points_json) {
            push_term(&mut terms, &value);
        }
    }
    for token in split_descriptive_terms(brand_description) {
        push_term(&mut terms, &token);
    }
    terms
}

fn build_brand_terms(
    brand_name: &str,
    brand_description: &str,
    creative_directions: &[CreativeDirection],
) -> Vec<String> {
    let mut terms = Vec::new();
    push_term(&mut terms, brand_name);
    for token in split_descriptive_terms(brand_description) {
        push_term(&mut terms, &token);
    }
    if let Some(latest) = creative_directions.last() {
        for token in split_descriptive_terms(&latest.tone) {
            push_term(&mut terms, &token);
        }
        for token in split_descriptive_terms(&latest.visual_style) {
            push_term(&mut terms, &token);
        }
    }
    terms
}

fn push_term(terms: &mut Vec<String>, value: &str) {
    let normalized = normalize_text(value);
    if normalized.len() >= 3 && !terms.iter().any(|term| term == &normalized) {
        terms.push(normalized);
    }
}

fn split_descriptive_terms(value: &str) -> Vec<String> {
    value
        .split(|character: char| {
            character == ','
                || character == ';'
                || character == '/'
                || character == '，'
                || character == '；'
        })
        .map(str::trim)
        .filter(|part| part.len() >= 3)
        .map(ToString::to_string)
        .collect()
}

fn string_array(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn normalize_text(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || character.is_whitespace() {
                character.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn text_contains_any_term(text: &str, terms: &[String]) -> bool {
    let normalized = normalize_text(text);
    terms.iter().any(|term| normalized.contains(term))
}

fn review_storyboards(
    project_id: &str,
    project_duration_seconds: i64,
    storyboards: &[StoryboardWithShots],
    product_terms: &[String],
    brand_terms: &[String],
    checks: &mut Vec<QualityReviewCheck>,
    suggestions: &mut Vec<QualityReviewSuggestion>,
) {
    for storyboard in storyboards {
        let shot_total = storyboard
            .shots
            .iter()
            .map(|shot| shot.duration_seconds)
            .sum::<i64>();
        if shot_total != storyboard.storyboard.duration_seconds
            || shot_total != project_duration_seconds
        {
            checks.push(QualityReviewCheck {
                id: format!("duration-{}", storyboard.storyboard.id),
                category: "storyboard_duration".to_string(),
                title: "Storyboard duration matches target".to_string(),
                status: "failed".to_string(),
                severity: "high".to_string(),
                source_type: "storyboard".to_string(),
                source_id: storyboard.storyboard.id.clone(),
                message: format!(
                    "Storyboard totals {}s while storyboard target is {}s and project target is {}s.",
                    shot_total, storyboard.storyboard.duration_seconds, project_duration_seconds
                ),
                evidence: vec![
                    format!("Project target: {}s", project_duration_seconds),
                    format!("Storyboard target: {}s", storyboard.storyboard.duration_seconds),
                    format!("Shot total: {}s", shot_total),
                ],
                suggestion_ids: vec![],
            });
        }

        for index in 0..storyboard.shots.len() {
            let shot = &storyboard.shots[index];
            review_shot_visibility(project_id, shot, product_terms, checks, suggestions);
            review_shot_brand(shot, brand_terms, checks, suggestions);
            if index > 0 {
                review_shot_repetition(&storyboard.shots[index - 1], shot, checks, suggestions);
            }
        }
    }
}

fn review_shot_visibility(
    project_id: &str,
    shot: &Shot,
    product_terms: &[String],
    checks: &mut Vec<QualityReviewCheck>,
    suggestions: &mut Vec<QualityReviewSuggestion>,
) {
    let shot_text = format!(
        "{} {} {} {} {}",
        shot.description,
        shot.model_action,
        shot.scene,
        shot.lighting,
        shot.metadata_json
            .get("garment_focus")
            .and_then(Value::as_str)
            .unwrap_or_default()
    );
    if product_terms.is_empty() || text_contains_any_term(&shot_text, product_terms) {
        return;
    }

    let check_id = format!("garment-{}", shot.id);
    checks.push(QualityReviewCheck {
        id: check_id.clone(),
        category: "garment_visibility".to_string(),
        title: "Shot keeps garment visible".to_string(),
        status: "failed".to_string(),
        severity: "high".to_string(),
        source_type: "shot".to_string(),
        source_id: shot.id.clone(),
        message: format!(
            "Shot {} does not clearly mention the garment or selling point.",
            shot.shot_number
        ),
        evidence: vec![shot.description.clone()],
        suggestion_ids: vec![],
    });

    suggestions.push(QualityReviewSuggestion {
        id: format!("suggest-shot-{}-description", shot.id),
        target_type: "shot".to_string(),
        target_id: shot.id.clone(),
        field: "description".to_string(),
        current_value: shot.description.clone(),
        suggested_value: append_sentence(
            &shot.description,
            "Keep the garment silhouette and key material benefit clearly visible in frame.",
        ),
        rationale: format!("Shot should surface a visible garment cue for project {project_id}."),
        status: QualityReviewSuggestionStatus::Pending.as_str().to_string(),
        check_ids: vec![check_id],
    });
}

fn review_shot_brand(
    shot: &Shot,
    brand_terms: &[String],
    checks: &mut Vec<QualityReviewCheck>,
    suggestions: &mut Vec<QualityReviewSuggestion>,
) {
    if brand_terms.is_empty() {
        return;
    }

    let shot_text = format!(
        "{} {} {} {}",
        shot.description, shot.scene, shot.lighting, shot.rationale
    );
    if text_contains_any_term(&shot_text, brand_terms) {
        return;
    }

    let check_id = format!("brand-shot-{}", shot.id);
    checks.push(QualityReviewCheck {
        id: check_id.clone(),
        category: "brand_consistency".to_string(),
        title: "Shot matches brand direction".to_string(),
        status: "warning".to_string(),
        severity: "medium".to_string(),
        source_type: "shot".to_string(),
        source_id: shot.id.clone(),
        message: format!(
            "Shot {} does not clearly carry saved brand or creative direction terms.",
            shot.shot_number
        ),
        evidence: vec![first_line(&shot.description)],
        suggestion_ids: vec![],
    });

    if suggestions.iter().any(|suggestion| {
        suggestion.target_type == "shot"
            && suggestion.target_id == shot.id
            && suggestion.field == "description"
    }) {
        return;
    }

    suggestions.push(QualityReviewSuggestion {
        id: format!("suggest-shot-{}-brand-description", shot.id),
        target_type: "shot".to_string(),
        target_id: shot.id.clone(),
        field: "description".to_string(),
        current_value: shot.description.clone(),
        suggested_value: append_sentence(
            &shot.description,
            "Maintain the brand's established visual tone in the shot.",
        ),
        rationale: "Shot should stay aligned with saved brand and creative direction.".to_string(),
        status: QualityReviewSuggestionStatus::Pending.as_str().to_string(),
        check_ids: vec![check_id],
    });
}

fn review_shot_repetition(
    previous: &Shot,
    current: &Shot,
    checks: &mut Vec<QualityReviewCheck>,
    suggestions: &mut Vec<QualityReviewSuggestion>,
) {
    let matching_fields = [
        normalize_text(&previous.description) == normalize_text(&current.description),
        normalize_text(&previous.model_action) == normalize_text(&current.model_action),
        normalize_text(&previous.camera_movement) == normalize_text(&current.camera_movement),
        normalize_text(&previous.scene) == normalize_text(&current.scene),
    ]
    .into_iter()
    .filter(|matched| *matched)
    .count();

    if matching_fields < 3 {
        return;
    }

    let check_id = format!("repetition-{}", current.id);
    checks.push(QualityReviewCheck {
        id: check_id.clone(),
        category: "shot_repetition".to_string(),
        title: "Shot advances visual story".to_string(),
        status: "warning".to_string(),
        severity: "medium".to_string(),
        source_type: "shot".to_string(),
        source_id: current.id.clone(),
        message: format!(
            "Shot {} repeats the previous shot too closely.",
            current.shot_number
        ),
        evidence: vec![
            format!("Previous: {}", previous.description),
            format!("Current: {}", current.description),
        ],
        suggestion_ids: vec![],
    });

    suggestions.push(QualityReviewSuggestion {
        id: format!("suggest-shot-{}-repetition-description", current.id),
        target_type: "shot".to_string(),
        target_id: current.id.clone(),
        field: "description".to_string(),
        current_value: current.description.clone(),
        suggested_value: append_sentence(
            &current.description,
            "Change the framing or action so this beat reveals a new garment detail.",
        ),
        rationale: "Repeated shots weaken the 15 to 30 second sequence.".to_string(),
        status: QualityReviewSuggestionStatus::Pending.as_str().to_string(),
        check_ids: vec![check_id],
    });
}

fn review_prompts(
    prompt_packages: &[PromptPackage],
    product_terms: &[String],
    brand_terms: &[String],
    checks: &mut Vec<QualityReviewCheck>,
    suggestions: &mut Vec<QualityReviewSuggestion>,
) {
    for package in prompt_packages {
        let view = prompt_package_view(package.clone());
        if !view.missing_fields.is_empty() {
            let check_id = format!("prompt-completeness-{}", package.id);
            checks.push(QualityReviewCheck {
                id: check_id.clone(),
                category: "prompt_completeness".to_string(),
                title: "Prompt contains required adapter fields".to_string(),
                status: "failed".to_string(),
                severity: "high".to_string(),
                source_type: "prompt_package".to_string(),
                source_id: package.id.clone(),
                message: format!(
                    "{} is missing required field(s): {}.",
                    view.adapter_display_name,
                    view.missing_fields.join(", ")
                ),
                evidence: view.missing_fields.clone(),
                suggestion_ids: vec![],
            });
            suggestions.push(QualityReviewSuggestion {
                id: format!("suggest-prompt-{}-missing-fields", package.id),
                target_type: "prompt_package".to_string(),
                target_id: package.id.clone(),
                field: "prompt_text".to_string(),
                current_value: package.prompt_text.clone(),
                suggested_value: append_sentence(
                    &package.prompt_text,
                    &format!("Include: {}.", view.missing_fields.join(", ")),
                ),
                rationale: "Prompt adapters need complete fields for reliable model handoff."
                    .to_string(),
                status: QualityReviewSuggestionStatus::Pending.as_str().to_string(),
                check_ids: vec![check_id],
            });
        }

        review_prompt_context(package, product_terms, brand_terms, checks, suggestions);
    }
}

fn review_prompt_context(
    package: &PromptPackage,
    product_terms: &[String],
    brand_terms: &[String],
    checks: &mut Vec<QualityReviewCheck>,
    suggestions: &mut Vec<QualityReviewSuggestion>,
) {
    let context_terms = product_terms
        .iter()
        .chain(brand_terms.iter())
        .cloned()
        .collect::<Vec<_>>();
    if context_terms.is_empty() || text_contains_any_term(&package.prompt_text, &context_terms) {
        return;
    }

    let check_id = format!("prompt-context-{}", package.id);
    checks.push(QualityReviewCheck {
        id: check_id.clone(),
        category: "prompt_context".to_string(),
        title: "Prompt carries project context".to_string(),
        status: "warning".to_string(),
        severity: "medium".to_string(),
        source_type: "prompt_package".to_string(),
        source_id: package.id.clone(),
        message: "Prompt does not clearly include saved brand or product context.".to_string(),
        evidence: vec![first_line(&package.prompt_text)],
        suggestion_ids: vec![],
    });

    if suggestions.iter().any(|suggestion| {
        suggestion.target_type == "prompt_package"
            && suggestion.target_id == package.id
            && suggestion.field == "prompt_text"
    }) {
        return;
    }

    suggestions.push(QualityReviewSuggestion {
        id: format!("suggest-prompt-{}-context", package.id),
        target_type: "prompt_package".to_string(),
        target_id: package.id.clone(),
        field: "prompt_text".to_string(),
        current_value: package.prompt_text.clone(),
        suggested_value: append_sentence(
            &package.prompt_text,
            "Reference the saved brand mood, garment category, and main material benefit.",
        ),
        rationale: "Prompt should carry the same brand/product context as the project brief."
            .to_string(),
        status: QualityReviewSuggestionStatus::Pending.as_str().to_string(),
        check_ids: vec![check_id],
    });
}

fn link_suggestions_to_checks(
    checks: &mut [QualityReviewCheck],
    suggestions: &[QualityReviewSuggestion],
) {
    for check in checks {
        check.suggestion_ids = suggestions
            .iter()
            .filter(|suggestion| suggestion.check_ids.iter().any(|id| id == &check.id))
            .map(|suggestion| suggestion.id.clone())
            .collect();
    }
}

fn review_score(checks: &[QualityReviewCheck]) -> i64 {
    let penalty = checks
        .iter()
        .map(
            |check| match (check.status.as_str(), check.severity.as_str()) {
                ("failed", "high") => 18,
                ("failed", _) => 12,
                ("warning", "high") => 10,
                ("warning", _) => 6,
                _ => 0,
            },
        )
        .sum::<i64>();
    (100 - penalty).clamp(0, 100)
}

fn review_summary(
    score: i64,
    checks: &[QualityReviewCheck],
    suggestions: &[QualityReviewSuggestion],
) -> String {
    let failed_count = checks
        .iter()
        .filter(|check| check.status == "failed")
        .count();
    let warning_count = checks
        .iter()
        .filter(|check| check.status == "warning")
        .count();
    let pending_suggestions = suggestions
        .iter()
        .filter(|suggestion| suggestion.status == QualityReviewSuggestionStatus::Pending.as_str())
        .count();
    format!(
        "Quality review scored {score}/100 with {failed_count} failed check(s), {warning_count} warning(s), and {pending_suggestions} pending suggestion(s)."
    )
}

fn quality_review_goal(project_title: &str, input: &QualityReviewGenerationInput) -> String {
    if input.user_direction.trim().is_empty() {
        format!("Review content quality for {project_title}.")
    } else {
        format!(
            "Review content quality for {}: {}",
            project_title,
            input.user_direction.trim()
        )
    }
}

fn quality_review_plan_json(input: &QualityReviewGenerationInput) -> Value {
    json!([
        {
            "role": "reviewer",
            "title": "Read saved storyboard, prompt, brand, and product context",
            "project_id": input.project_id
        },
        {
            "role": "reviewer",
            "title": "Run deterministic quality checklist"
        },
        {
            "role": "planner",
            "title": "Prepare user-accepted revision suggestions"
        }
    ])
}

fn create_quality_review_events(
    repo: &Repository<'_>,
    agent_run_id: &str,
    checks: &[QualityReviewCheck],
    suggestions: &[QualityReviewSuggestion],
) -> JoiResult<Vec<AgentRunEvent>> {
    let specs = [
        (
            1,
            "reviewer",
            "context_read",
            "Read project context for quality review.".to_string(),
            json!({ "check_count": checks.len() }),
        ),
        (
            2,
            "reviewer",
            "checklist_completed",
            format!("Completed {} quality check(s).", checks.len()),
            json!({ "checks": checks }),
        ),
        (
            3,
            "planner",
            "suggestions_prepared",
            format!("Prepared {} revision suggestion(s).", suggestions.len()),
            json!({ "suggestions": suggestions }),
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

fn append_sentence(current: &str, addition: &str) -> String {
    let trimmed = current.trim();
    if trimmed.is_empty() {
        return addition.trim().to_string();
    }
    if trimmed.ends_with('.') || trimmed.ends_with('!') || trimmed.ends_with('?') {
        format!("{} {}", trimmed, addition.trim())
    } else {
        format!("{}. {}", trimmed, addition.trim())
    }
}

fn first_line(value: &str) -> String {
    value.lines().next().unwrap_or_default().trim().to_string()
}
