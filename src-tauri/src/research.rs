use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::agent_context::{build_project_context, AgentProjectContext};
use crate::error::{JoiError, JoiResult};
use crate::models::{AgentRun, AgentRunEvent, ResearchReport};
use crate::repositories::{AgentRunCreate, AgentRunEventCreate, Repository, ResearchReportCreate};
use crate::validation::validate_required_text;

const RESEARCH_ROLES: [&str; 3] = ["researcher", "planner", "reviewer"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchSourceInput {
    pub title: String,
    pub url: String,
    pub source_type: String,
    pub excerpt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchReportInput {
    pub project_id: String,
    pub research_goal: String,
    pub market_focus: String,
    pub platform_focus: Vec<String>,
    pub source_materials: Vec<ResearchSourceInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchFinding {
    pub title: String,
    pub insight: String,
    pub evidence: String,
    pub source_index: usize,
    pub creative_implication: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchSourceCitation {
    pub index: usize,
    pub title: String,
    pub url: String,
    pub source_type: String,
    pub excerpt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchReportResult {
    pub report: ResearchReport,
    pub findings: Vec<ResearchFinding>,
    pub sources: Vec<ResearchSourceCitation>,
    pub rationale: String,
    pub creative_implications: Vec<String>,
    pub agent_run: AgentRun,
    pub agent_events: Vec<AgentRunEvent>,
}

struct ResearchEventSpec {
    sequence_number: i64,
    role: &'static str,
    event_type: &'static str,
    message: String,
    payload_json: Value,
}

pub fn generate_research_report(
    repo: &Repository<'_>,
    input: ResearchReportInput,
    hermes_version: String,
) -> JoiResult<ResearchReportResult> {
    validate_input(&input)?;
    let context = build_project_context(repo, &input.project_id)?;
    let sources = normalize_sources(&input.source_materials);
    let product_name = product_name(&context);
    let findings = build_findings(&product_name, &input, &sources);
    let creative_implications = findings
        .iter()
        .map(|finding| finding.creative_implication.clone())
        .collect::<Vec<_>>();
    let summary = build_summary(&context, &input, findings.len());
    let rationale = build_rationale(&context, &input, &product_name, sources.len());

    let report = repo.create_research_report(ResearchReportCreate {
        project_id: input.project_id.clone(),
        summary: summary.clone(),
        findings_json: json!(findings),
        sources_json: json!(sources),
    })?;

    let agent_run = repo.create_agent_run(AgentRunCreate {
        project_id: input.project_id.clone(),
        user_goal: input.research_goal.clone(),
        status: "completed".to_string(),
        runtime_kind: "hermes_core".to_string(),
        runtime_mode: "local_research_bridge".to_string(),
        runtime_version: hermes_version,
        roles_json: json!(RESEARCH_ROLES),
        plan_json: build_plan_json(&input, sources.len(), findings.len()),
        result_summary: summary,
    })?;

    let mut agent_events = Vec::new();
    for spec in build_event_specs(&context, &input, &report, &sources, &findings) {
        agent_events.push(repo.create_agent_run_event(AgentRunEventCreate {
            agent_run_id: agent_run.id.clone(),
            sequence_number: spec.sequence_number,
            role: spec.role.to_string(),
            event_type: spec.event_type.to_string(),
            message: spec.message,
            payload_json: spec.payload_json,
        })?);
    }

    Ok(ResearchReportResult {
        report,
        findings,
        sources,
        rationale,
        creative_implications,
        agent_run,
        agent_events,
    })
}

fn validate_input(input: &ResearchReportInput) -> JoiResult<()> {
    validate_required_text("Research goal", &input.research_goal)?;
    if input.source_materials.is_empty() {
        return Err(JoiError::Validation(
            "Research report requires at least one research source".to_string(),
        ));
    }
    for (index, source) in input.source_materials.iter().enumerate() {
        validate_required_text(
            &format!("Research source {} title", index + 1),
            &source.title,
        )?;
        validate_required_text(
            &format!("Research source {} excerpt", index + 1),
            &source.excerpt,
        )?;
    }
    Ok(())
}

fn normalize_sources(input: &[ResearchSourceInput]) -> Vec<ResearchSourceCitation> {
    input
        .iter()
        .enumerate()
        .map(|(index, source)| ResearchSourceCitation {
            index: index + 1,
            title: source.title.trim().to_string(),
            url: source.url.trim().to_string(),
            source_type: if source.source_type.trim().is_empty() {
                "reference".to_string()
            } else {
                source.source_type.trim().to_string()
            },
            excerpt: source.excerpt.trim().to_string(),
        })
        .collect()
}

fn build_findings(
    product_name: &str,
    input: &ResearchReportInput,
    sources: &[ResearchSourceCitation],
) -> Vec<ResearchFinding> {
    sources
        .iter()
        .map(|source| {
            let source_excerpt = source.excerpt.to_lowercase();
            let creative_implication = if source_excerpt.contains("texture")
                || source_excerpt.contains("fabric")
            {
                "Use tactile close-ups as visual proof before the model movement.".to_string()
            } else if source_excerpt.contains("movement") || source_excerpt.contains("motion") {
                "Use model motion to demonstrate garment behavior in the first half of the film."
                    .to_string()
            } else {
                "Translate the source observation into one clear shot requirement.".to_string()
            };
            ResearchFinding {
                title: format!("{} insight from {}", product_name, source.title),
                insight: format!(
                    "For {}, the source suggests a usable fashion advertising angle: {}",
                    input.research_goal.trim(),
                    first_sentence(&source.excerpt)
                ),
                evidence: source.excerpt.clone(),
                source_index: source.index,
                creative_implication,
            }
        })
        .collect()
}

fn build_summary(
    context: &AgentProjectContext,
    input: &ResearchReportInput,
    finding_count: usize,
) -> String {
    format!(
        "Research for {}: {} source-backed {} for {}, focused on {}.",
        context.project.title,
        finding_count,
        if finding_count == 1 {
            "finding"
        } else {
            "findings"
        },
        audience_or_market(context, input),
        platform_focus_label(input)
    )
}

fn build_rationale(
    context: &AgentProjectContext,
    input: &ResearchReportInput,
    product_name: &str,
    source_count: usize,
) -> String {
    format!(
        "Research for {} uses {} source material(s) to support {} for {} with focus on {}.",
        context.project.title,
        source_count,
        input.research_goal.trim(),
        product_name,
        audience_or_market(context, input)
    )
}

fn build_plan_json(
    input: &ResearchReportInput,
    source_count: usize,
    finding_count: usize,
) -> Value {
    json!([
        {
            "role": "researcher",
            "stage": "0.14",
            "title": "Read project research context",
            "task": input.research_goal.trim(),
            "status": "completed"
        },
        {
            "role": "researcher",
            "stage": "0.14",
            "title": "Normalize provided sources",
            "source_count": source_count,
            "status": "completed"
        },
        {
            "role": "reviewer",
            "stage": "0.14",
            "title": "Check source-backed findings",
            "finding_count": finding_count,
            "status": "completed"
        }
    ])
}

fn build_event_specs(
    context: &AgentProjectContext,
    input: &ResearchReportInput,
    report: &ResearchReport,
    sources: &[ResearchSourceCitation],
    findings: &[ResearchFinding],
) -> Vec<ResearchEventSpec> {
    vec![
        ResearchEventSpec {
            sequence_number: 1,
            role: "researcher",
            event_type: "research_context_read",
            message: format!(
                "Read research context for {} with goal {}.",
                context.project.title,
                input.research_goal.trim()
            ),
            payload_json: json!({
                "brand_name": context.brand.name,
                "project_title": context.project.title,
                "product_name": product_name(context),
                "market_focus": input.market_focus.trim(),
                "platform_focus": normalized_platform_focus(input)
            }),
        },
        ResearchEventSpec {
            sequence_number: 2,
            role: "researcher",
            event_type: "sources_collected",
            message: format!("Collected {} source material(s).", sources.len()),
            payload_json: json!({
                "source_count": sources.len(),
                "sources": sources
            }),
        },
        ResearchEventSpec {
            sequence_number: 3,
            role: "researcher",
            event_type: "findings_drafted",
            message: format!("Drafted {} source-backed finding(s).", findings.len()),
            payload_json: json!({
                "finding_count": findings.len(),
                "findings": findings
            }),
        },
        ResearchEventSpec {
            sequence_number: 4,
            role: "reviewer",
            event_type: "citations_checked",
            message: "Checked that every finding includes a source index.".to_string(),
            payload_json: json!({
                "all_findings_source_backed": findings
                    .iter()
                    .all(|finding| finding.source_index > 0 && finding.source_index <= sources.len())
            }),
        },
        ResearchEventSpec {
            sequence_number: 5,
            role: "planner",
            event_type: "report_saved",
            message: format!("Saved research report {}.", report.id),
            payload_json: json!({
                "report_id": report.id,
                "summary": report.summary
            }),
        },
    ]
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

fn audience_or_market(context: &AgentProjectContext, input: &ResearchReportInput) -> String {
    context
        .latest_product_understanding
        .as_ref()
        .and_then(|understanding| {
            let audience = understanding.audience.trim();
            if audience.is_empty() {
                None
            } else {
                Some(audience.to_string())
            }
        })
        .or_else(|| {
            let market = input.market_focus.trim();
            if market.is_empty() {
                None
            } else {
                Some(market.to_string())
            }
        })
        .unwrap_or_else(|| "the target audience".to_string())
}

fn normalized_platform_focus(input: &ResearchReportInput) -> Vec<String> {
    input
        .platform_focus
        .iter()
        .map(|platform| platform.trim().to_string())
        .filter(|platform| !platform.is_empty())
        .collect()
}

fn platform_focus_label(input: &ResearchReportInput) -> String {
    let platforms = normalized_platform_focus(input);
    if platforms.is_empty() {
        "selected platforms".to_string()
    } else {
        platforms.join(" and ")
    }
}

fn first_sentence(value: &str) -> String {
    value
        .split(['.', '!', '?'])
        .map(str::trim)
        .find(|part| !part.is_empty())
        .unwrap_or(value.trim())
        .to_string()
}
