use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::agent_context::{build_project_context, AgentProjectContext};
use crate::error::{JoiError, JoiResult};
use crate::models::{AgentRun, AgentRunEvent, MemoryEntry};
use crate::repositories::{AgentRunCreate, AgentRunEventCreate, MemoryCandidateCreate, Repository};

const MEMORY_ROLES: [&str; 2] = ["memory_curator", "reviewer"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryCurationInput {
    pub project_id: String,
    pub feedback_text: String,
    pub include_research_reports: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCandidateResult {
    pub entry: MemoryEntry,
    pub reason: String,
    pub has_conflict: bool,
    pub conflict_memory_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCurationResult {
    pub candidates: Vec<MemoryCandidateResult>,
    pub agent_run: AgentRun,
    pub agent_events: Vec<AgentRunEvent>,
}

struct CandidateDraft {
    content: String,
    source: String,
    source_entity_type: String,
    source_entity_id: String,
    confidence: f64,
    reason: String,
}

struct MemoryEventSpec {
    sequence_number: i64,
    role: &'static str,
    event_type: &'static str,
    message: String,
    payload_json: Value,
}

pub fn curate_memory_candidates(
    repo: &Repository<'_>,
    input: MemoryCurationInput,
    hermes_version: String,
) -> JoiResult<MemoryCurationResult> {
    let context = build_project_context(repo, &input.project_id)?;
    let mut drafts = Vec::new();
    if input.include_research_reports {
        drafts.extend(research_candidates(repo, &input.project_id)?);
    }
    drafts.extend(feedback_candidates(&input.feedback_text));
    drafts.retain(|draft| !draft.content.trim().is_empty());

    if drafts.is_empty() {
        return Err(JoiError::Validation(
            "Memory curation requires candidate material".to_string(),
        ));
    }

    let existing_memory = repo.list_memory_entries("project", None, Some(&input.project_id))?;
    let mut candidates = Vec::new();
    for draft in drafts {
        let conflict_memory_ids = find_conflicts(&draft.content, &existing_memory);
        let entry = repo.create_memory_candidate(MemoryCandidateCreate {
            scope: "project".to_string(),
            brand_id: Some(context.project.brand_id.clone()),
            project_id: Some(input.project_id.clone()),
            content: draft.content,
            source: draft.source,
            source_entity_type: draft.source_entity_type,
            source_entity_id: draft.source_entity_id,
            confidence: draft.confidence,
        })?;
        candidates.push(MemoryCandidateResult {
            entry,
            reason: draft.reason,
            has_conflict: !conflict_memory_ids.is_empty(),
            conflict_memory_ids,
        });
    }

    let result_summary = format!(
        "Created {} proposed memory candidate(s) for {}.",
        candidates.len(),
        context.project.title
    );
    let agent_run = repo.create_agent_run(AgentRunCreate {
        project_id: input.project_id.clone(),
        user_goal: "Curate practical long-term memory candidates".to_string(),
        status: "completed".to_string(),
        runtime_kind: "hermes_core".to_string(),
        runtime_mode: "local_memory_bridge".to_string(),
        runtime_version: hermes_version,
        roles_json: json!(MEMORY_ROLES),
        plan_json: build_plan_json(&input, candidates.len()),
        result_summary,
    })?;

    let mut agent_events = Vec::new();
    for spec in build_event_specs(&context, &input, &candidates, existing_memory.len()) {
        agent_events.push(repo.create_agent_run_event(AgentRunEventCreate {
            agent_run_id: agent_run.id.clone(),
            sequence_number: spec.sequence_number,
            role: spec.role.to_string(),
            event_type: spec.event_type.to_string(),
            message: spec.message,
            payload_json: spec.payload_json,
        })?);
    }

    Ok(MemoryCurationResult {
        candidates,
        agent_run,
        agent_events,
    })
}

fn research_candidates(repo: &Repository<'_>, project_id: &str) -> JoiResult<Vec<CandidateDraft>> {
    let reports = repo.list_research_reports(project_id)?;
    let mut candidates = Vec::new();
    for report in reports {
        let Some(findings) = report.findings_json.as_array() else {
            continue;
        };
        for finding in findings {
            let content = string_field(finding, "creative_implication")
                .or_else(|| string_field(finding, "insight"))
                .unwrap_or_default();
            if content.trim().is_empty() {
                continue;
            }
            candidates.push(CandidateDraft {
                content: content.trim().to_string(),
                source: "research report".to_string(),
                source_entity_type: "research_report".to_string(),
                source_entity_id: report.id.clone(),
                confidence: 0.72,
                reason: "Source-backed research implication can guide future generation."
                    .to_string(),
            });
        }
    }
    Ok(candidates)
}

fn feedback_candidates(feedback_text: &str) -> Vec<CandidateDraft> {
    split_feedback(feedback_text)
        .into_iter()
        .map(|content| CandidateDraft {
            content,
            source: "user feedback".to_string(),
            source_entity_type: "feedback".to_string(),
            source_entity_id: String::new(),
            confidence: 0.86,
            reason: "User feedback expressed a reusable project preference.".to_string(),
        })
        .collect()
}

fn split_feedback(feedback_text: &str) -> Vec<String> {
    feedback_text
        .split(['\n', '.', '。', '!', '！', '?', '？', ';', '；'])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn find_conflicts(content: &str, existing_memory: &[MemoryEntry]) -> Vec<String> {
    let normalized_content = normalize_memory_content(content);
    existing_memory
        .iter()
        .filter(|memory| memory.status == "proposed" || memory.status == "accepted")
        .filter(|memory| normalize_memory_content(&memory.content) == normalized_content)
        .map(|memory| memory.id.clone())
        .collect()
}

fn normalize_memory_content(content: &str) -> String {
    content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_lowercase()
}

fn build_plan_json(input: &MemoryCurationInput, candidate_count: usize) -> Value {
    json!([
        {
            "role": "memory_curator",
            "stage": "0.15",
            "title": "Read project memory context",
            "status": "completed"
        },
        {
            "role": "memory_curator",
            "stage": "0.15",
            "title": "Draft memory candidates",
            "include_research_reports": input.include_research_reports,
            "has_feedback": !input.feedback_text.trim().is_empty(),
            "candidate_count": candidate_count,
            "status": "completed"
        },
        {
            "role": "reviewer",
            "stage": "0.15",
            "title": "Check duplicate memory conflicts",
            "status": "completed"
        }
    ])
}

fn build_event_specs(
    context: &AgentProjectContext,
    input: &MemoryCurationInput,
    candidates: &[MemoryCandidateResult],
    existing_memory_count: usize,
) -> Vec<MemoryEventSpec> {
    let conflict_count = candidates
        .iter()
        .filter(|candidate| candidate.has_conflict)
        .count();
    vec![
        MemoryEventSpec {
            sequence_number: 1,
            role: "memory_curator",
            event_type: "memory_context_read",
            message: format!("Read memory context for {}.", context.project.title),
            payload_json: json!({
                "brand_name": context.brand.name,
                "project_title": context.project.title,
                "existing_memory_count": existing_memory_count
            }),
        },
        MemoryEventSpec {
            sequence_number: 2,
            role: "memory_curator",
            event_type: "candidate_sources_collected",
            message: "Collected memory candidate source material.".to_string(),
            payload_json: json!({
                "include_research_reports": input.include_research_reports,
                "feedback_text_present": !input.feedback_text.trim().is_empty()
            }),
        },
        MemoryEventSpec {
            sequence_number: 3,
            role: "memory_curator",
            event_type: "memory_candidates_drafted",
            message: format!("Drafted {} memory candidate(s).", candidates.len()),
            payload_json: json!({
                "candidate_count": candidates.len(),
                "candidate_ids": candidates
                    .iter()
                    .map(|candidate| candidate.entry.id.clone())
                    .collect::<Vec<_>>()
            }),
        },
        MemoryEventSpec {
            sequence_number: 4,
            role: "reviewer",
            event_type: "memory_conflicts_checked",
            message: format!(
                "Checked memory conflicts; found {} duplicate(s).",
                conflict_count
            ),
            payload_json: json!({
                "conflict_count": conflict_count,
                "candidate_conflicts": candidates
                    .iter()
                    .map(|candidate| json!({
                        "memory_id": candidate.entry.id,
                        "has_conflict": candidate.has_conflict,
                        "conflict_memory_ids": candidate.conflict_memory_ids
                    }))
                    .collect::<Vec<_>>()
            }),
        },
        MemoryEventSpec {
            sequence_number: 5,
            role: "memory_curator",
            event_type: "memory_candidates_saved",
            message: "Saved proposed memory candidates.".to_string(),
            payload_json: json!({
                "saved_count": candidates.len()
            }),
        },
    ]
}
