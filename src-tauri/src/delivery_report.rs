use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::JoiResult;
use crate::models::{
    AgentRun, AgentRunEvent, Asset, Brand, CreativeDirection, DeliveryReport, ProductUnderstanding,
    Project, ProjectVersion, PromptPackage, ResearchReport,
};
use crate::project_package::slugify_project_title;
use crate::repositories::{
    AgentRunCreate, AgentRunEventCreate, DeliveryReportCreate, Repository, StoryboardWithShots,
};

const DELIVERY_REPORT_ROLES: [&str; 3] = ["planner", "reviewer", "memory_curator"];
const SECTIONS_FORMAT_VERSION: &str = "joi.delivery_report_sections.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeliveryReportGenerationInput {
    pub project_id: String,
    pub user_direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeliveryReportSectionStatus {
    pub id: String,
    pub title: String,
    pub status: String,
    pub source_count: usize,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryReportGenerationResult {
    pub report: DeliveryReport,
    pub sections: Vec<DeliveryReportSectionStatus>,
    pub package_preview: DeliveryPackagePreview,
    pub agent_run: AgentRun,
    pub agent_events: Vec<AgentRunEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeliveryPackagePreview {
    pub project_json_file_name: String,
    pub assets_folder_name: String,
    pub delivery_report_file_name: String,
    pub included_assets_count: usize,
    pub included_prompt_packages_count: usize,
    pub included_storyboards_count: usize,
    pub warnings: Vec<String>,
}

struct DeliveryContext {
    project: Project,
    brand: Brand,
    assets: Vec<Asset>,
    research_reports: Vec<ResearchReport>,
    product_understandings: Vec<ProductUnderstanding>,
    creative_directions: Vec<CreativeDirection>,
    storyboards: Vec<StoryboardWithShots>,
    prompt_packages: Vec<PromptPackage>,
    project_versions: Vec<ProjectVersion>,
}

struct DeliveryEventSpec {
    sequence_number: i64,
    role: &'static str,
    event_type: &'static str,
    message: String,
    payload_json: Value,
}

pub fn generate_delivery_report(
    repo: &Repository<'_>,
    input: DeliveryReportGenerationInput,
    hermes_version: String,
) -> JoiResult<DeliveryReportGenerationResult> {
    let context = read_delivery_context(repo, &input.project_id)?;
    let mut package_preview = build_package_preview(&context);
    let sections = build_section_statuses(&context);
    package_preview.warnings = package_warnings(&sections, &package_preview);
    let markdown = compose_delivery_markdown(&context, &sections, &package_preview);
    let report_title = format!("{} Delivery Report", context.project.title);

    let report = repo.create_delivery_report(DeliveryReportCreate {
        project_id: input.project_id.clone(),
        title: report_title.clone(),
        markdown: markdown.clone(),
        sections_json: sections_json(&sections, &package_preview),
        is_final_candidate: false,
    })?;

    let agent_run = repo.create_agent_run(AgentRunCreate {
        project_id: input.project_id.clone(),
        user_goal: delivery_goal(&context.project, &input),
        status: "completed".to_string(),
        runtime_kind: "hermes_core".to_string(),
        runtime_mode: "local_delivery_report_bridge".to_string(),
        runtime_version: hermes_version,
        roles_json: json!(DELIVERY_REPORT_ROLES),
        plan_json: build_plan_json(&input, &sections, &package_preview),
        result_summary: format!("Generated delivery report '{}'.", report_title),
    })?;

    let agent_events = create_delivery_events(repo, &agent_run.id, &context, &sections, &report)?;

    Ok(DeliveryReportGenerationResult {
        report,
        sections,
        package_preview,
        agent_run,
        agent_events,
    })
}

fn read_delivery_context(repo: &Repository<'_>, project_id: &str) -> JoiResult<DeliveryContext> {
    let project = repo.get_project(project_id)?;
    let brand = repo.get_brand(&project.brand_id)?;
    Ok(DeliveryContext {
        assets: repo.list_assets(project_id)?,
        research_reports: repo.list_research_reports(project_id)?,
        product_understandings: repo.list_product_understandings(project_id)?,
        creative_directions: repo.list_creative_directions(project_id)?,
        storyboards: repo.list_storyboards_with_typed_shots(project_id)?,
        prompt_packages: repo.list_prompt_packages(project_id)?,
        project_versions: repo.list_project_versions(project_id)?,
        project,
        brand,
    })
}

fn delivery_goal(project: &Project, input: &DeliveryReportGenerationInput) -> String {
    let direction = input.user_direction.trim();
    if direction.is_empty() {
        format!("Generate a delivery report for {}.", project.title)
    } else {
        format!(
            "Generate a delivery report for {}. {}",
            project.title, direction
        )
    }
}

fn build_package_preview(context: &DeliveryContext) -> DeliveryPackagePreview {
    let slug = slugify_project_title(&context.project.title);
    DeliveryPackagePreview {
        project_json_file_name: format!("{slug}.joi-project.json"),
        assets_folder_name: format!("{slug}-assets"),
        delivery_report_file_name: format!("{slug}-delivery-report.md"),
        included_assets_count: context.assets.len(),
        included_prompt_packages_count: context.prompt_packages.len(),
        included_storyboards_count: context.storyboards.len(),
        warnings: Vec::new(),
    }
}

fn build_section_statuses(context: &DeliveryContext) -> Vec<DeliveryReportSectionStatus> {
    vec![
        section_status(
            "project_brief",
            "Project Brief",
            1,
            warning_if(
                context.project.advertising_goal.trim().is_empty(),
                "No advertising goal saved yet.",
            ),
        ),
        section_status(
            "brand_understanding",
            "Brand Understanding",
            1,
            warning_if(
                context.brand.description.trim().is_empty(),
                "No brand description saved yet.",
            ),
        ),
        section_status(
            "product_understanding",
            "Product Understanding",
            context.product_understandings.len(),
            warning_if(
                context.product_understandings.is_empty(),
                "No saved product understanding yet.",
            ),
        ),
        section_status(
            "research_findings",
            "Research Findings",
            context.research_reports.len(),
            warning_if(
                context.research_reports.is_empty(),
                "No saved research report yet.",
            ),
        ),
        section_status(
            "creative_direction",
            "Creative Direction",
            context.creative_directions.len(),
            warning_if(
                context.creative_directions.is_empty(),
                "No saved creative direction yet.",
            ),
        ),
        section_status(
            "storyboard",
            "Storyboard",
            context.storyboards.len(),
            warning_if(context.storyboards.is_empty(), "No saved storyboard yet."),
        ),
        section_status(
            "prompt_packages",
            "Prompt Packages",
            context.prompt_packages.len(),
            warning_if(
                context.prompt_packages.is_empty(),
                "No saved prompt packages yet.",
            ),
        ),
        section_status(
            "assets",
            "Assets",
            context.assets.len(),
            warning_if(context.assets.is_empty(), "No project assets saved yet."),
        ),
        section_status(
            "version_notes",
            "Version Notes",
            context.project_versions.len(),
            warning_if(
                context.project_versions.is_empty(),
                "No saved project versions yet.",
            ),
        ),
    ]
}

fn section_status(
    id: &str,
    title: &str,
    source_count: usize,
    warning: &str,
) -> DeliveryReportSectionStatus {
    let status = if source_count == 0 {
        "missing"
    } else if warning.trim().is_empty() {
        "complete"
    } else {
        "partial"
    };
    DeliveryReportSectionStatus {
        id: id.to_string(),
        title: title.to_string(),
        status: status.to_string(),
        source_count,
        warning: warning.to_string(),
    }
}

fn warning_if(condition: bool, warning: &str) -> &str {
    if condition {
        warning
    } else {
        ""
    }
}

fn package_warnings(
    sections: &[DeliveryReportSectionStatus],
    preview: &DeliveryPackagePreview,
) -> Vec<String> {
    let mut warnings = sections
        .iter()
        .filter(|section| !section.warning.trim().is_empty())
        .map(|section| format!("{}: {}", section.title, section.warning))
        .collect::<Vec<_>>();
    if preview.included_assets_count == 0 && preview.included_prompt_packages_count == 0 {
        warnings.push(
            "Delivery package will contain only project JSON and the Markdown report.".to_string(),
        );
    }
    warnings
}

fn sections_json(
    sections: &[DeliveryReportSectionStatus],
    preview: &DeliveryPackagePreview,
) -> Value {
    json!({
        "format_version": SECTIONS_FORMAT_VERSION,
        "sections": sections,
        "package_preview": {
            "markdown_file_name": preview.delivery_report_file_name,
            "project_json_file_name": preview.project_json_file_name,
            "assets_folder_name": preview.assets_folder_name,
        }
    })
}

fn compose_delivery_markdown(
    context: &DeliveryContext,
    sections: &[DeliveryReportSectionStatus],
    preview: &DeliveryPackagePreview,
) -> String {
    let mut output = Vec::new();
    output.push(format!("# {} Delivery Report", context.project.title));
    output.push(compose_project_brief(context));
    output.push(compose_brand_understanding(context));
    output.push(compose_product_understanding(context));
    output.push(compose_research_findings(context));
    output.push(compose_creative_direction(context));
    output.push(compose_storyboard(context));
    output.push(compose_prompt_packages(context));
    output.push(compose_assets(context));
    output.push(compose_version_notes(context));
    output.push(compose_export_notes(preview, sections));
    output.join("\n\n")
}

fn compose_project_brief(context: &DeliveryContext) -> String {
    let mut lines = vec![
        "## Project Brief".to_string(),
        format!("- Project: {}", context.project.title),
        format!("- Duration: {} seconds", context.project.duration_seconds),
    ];
    if context.project.advertising_goal.trim().is_empty() {
        lines.push("- Goal: No advertising goal saved yet.".to_string());
    } else {
        lines.push(format!("- Goal: {}", context.project.advertising_goal));
    }
    lines.join("\n")
}

fn compose_brand_understanding(context: &DeliveryContext) -> String {
    let mut lines = vec![
        "## Brand Understanding".to_string(),
        format!("- Brand: {}", context.brand.name),
    ];
    if context.brand.description.trim().is_empty() {
        lines.push("- Description: No brand description saved yet.".to_string());
    } else {
        lines.push(format!("- Description: {}", context.brand.description));
    }
    lines.join("\n")
}

fn compose_product_understanding(context: &DeliveryContext) -> String {
    if context.product_understandings.is_empty() {
        return "## Product Understanding\nNo saved product understanding yet.".to_string();
    }

    let mut lines = vec!["## Product Understanding".to_string()];
    for understanding in &context.product_understandings {
        lines.push(format!(
            "- {} ({}) for {}",
            fallback(&understanding.product_name, "Unnamed product"),
            fallback(&understanding.category, "uncategorized"),
            fallback(&understanding.audience, "unspecified audience")
        ));
        lines.push(format!(
            "  - Selling points: {}",
            join_json_strings(&understanding.selling_points_json)
        ));
        lines.push(format!(
            "  - Constraints: {}",
            join_json_strings(&understanding.constraints_json)
        ));
    }
    lines.join("\n")
}

fn compose_research_findings(context: &DeliveryContext) -> String {
    if context.research_reports.is_empty() {
        return "## Research Findings\nNo saved research report yet.".to_string();
    }

    let mut lines = vec!["## Research Findings".to_string()];
    for report in &context.research_reports {
        lines.push(format!("- Summary: {}", report.summary));
        for finding in value_array(&report.findings_json) {
            let title = string_field(finding, "title").unwrap_or("Untitled finding");
            let insight = string_field(finding, "insight").unwrap_or("");
            let implication = string_field(finding, "creative_implication").unwrap_or("");
            lines.push(format!(
                "  - {}: {}{}",
                title,
                fallback(insight, "No insight text saved."),
                suffix_if_present(" Implication: ", implication)
            ));
        }
        for source in value_array(&report.sources_json) {
            let title = string_field(source, "title").unwrap_or("Untitled source");
            let url = string_field(source, "url").unwrap_or("");
            lines.push(format!(
                "  - Source: {}{}",
                title,
                suffix_if_present(" ", url)
            ));
        }
    }
    lines.join("\n")
}

fn compose_creative_direction(context: &DeliveryContext) -> String {
    if context.creative_directions.is_empty() {
        return "## Creative Direction\nNo saved creative direction yet.".to_string();
    }

    let mut lines = vec!["## Creative Direction".to_string()];
    for direction in &context.creative_directions {
        lines.push(format!("- Title: {}", direction.title));
        lines.push(format!(
            "  - Concept: {}",
            fallback(&direction.concept, "No concept saved.")
        ));
        lines.push(format!(
            "  - Visual style: {}",
            fallback(&direction.visual_style, "No visual style saved.")
        ));
        lines.push(format!(
            "  - Scene direction: {}",
            fallback(&direction.scene_direction, "No scene direction saved.")
        ));
    }
    lines.join("\n")
}

fn compose_storyboard(context: &DeliveryContext) -> String {
    if context.storyboards.is_empty() {
        return "## Storyboard\nNo saved storyboard yet.".to_string();
    }

    let mut lines = vec![
        "## Storyboard".to_string(),
        "| Shot | Duration | Visual | Action | Camera | Garment | Text |".to_string(),
        "| --- | ---: | --- | --- | --- | --- | --- |".to_string(),
    ];
    for item in &context.storyboards {
        for shot in &item.shots {
            lines.push(format!(
                "| {} | {}s | {} | {} | {} | {} | {} |",
                shot.shot_number,
                shot.duration_seconds,
                table_cell(&shot.description),
                table_cell(&shot.model_action),
                table_cell(&shot.camera_movement),
                table_cell(json_string(&shot.metadata_json, "garment_focus")),
                table_cell(&shot.subtitle_or_voiceover)
            ));
        }
    }
    lines.join("\n")
}

fn compose_prompt_packages(context: &DeliveryContext) -> String {
    if context.prompt_packages.is_empty() {
        return "## Prompt Packages\nNo saved prompt packages yet.".to_string();
    }

    let shot_numbers = shot_number_lookup(&context.storyboards);
    let mut lines = vec![
        "## Prompt Packages".to_string(),
        "| Platform | Modality | Source | Prompt Summary | Missing Fields |".to_string(),
        "| --- | --- | --- | --- | --- |".to_string(),
    ];
    for package in &context.prompt_packages {
        lines.push(format!(
            "| {} | {} | {} | {} | {} |",
            package.platform,
            package.modality,
            table_cell(prompt_source(package, &shot_numbers)),
            table_cell(first_prompt_line(&package.prompt_text)),
            table_cell(join_json_strings(
                &package.parameters_json["missing_fields"]
            ))
        ));
    }
    lines.join("\n")
}

fn compose_assets(context: &DeliveryContext) -> String {
    if context.assets.is_empty() {
        return "## Assets\nNo project assets saved yet.".to_string();
    }

    let mut lines = vec!["## Assets".to_string()];
    for asset in &context.assets {
        lines.push(format!(
            "- {} ({}) - {}",
            asset.display_name, asset.kind, asset.relative_path
        ));
    }
    lines.join("\n")
}

fn compose_version_notes(context: &DeliveryContext) -> String {
    if context.project_versions.is_empty() {
        return "## Version Notes\nNo saved project versions yet.".to_string();
    }

    let mut lines = vec!["## Version Notes".to_string()];
    for version in &context.project_versions {
        lines.push(format!(
            "- v{} {}: {}",
            version.version_number, version.label, version.change_reason
        ));
    }
    lines.join("\n")
}

fn compose_export_notes(
    preview: &DeliveryPackagePreview,
    _sections: &[DeliveryReportSectionStatus],
) -> String {
    let mut lines = vec![
        "## Export Notes".to_string(),
        format!("- Project JSON: {}", preview.project_json_file_name),
        format!("- Assets folder: {}", preview.assets_folder_name),
        format!("- Delivery report: {}", preview.delivery_report_file_name),
    ];
    if preview.warnings.is_empty() {
        lines.push("- Warnings: none".to_string());
    } else {
        lines.push(format!("- Warnings: {}", preview.warnings.join("; ")));
    }
    lines.join("\n")
}

fn create_delivery_events(
    repo: &Repository<'_>,
    agent_run_id: &str,
    context: &DeliveryContext,
    sections: &[DeliveryReportSectionStatus],
    report: &DeliveryReport,
) -> JoiResult<Vec<AgentRunEvent>> {
    let events = vec![
        DeliveryEventSpec {
            sequence_number: 1,
            role: "planner",
            event_type: "delivery_context_read",
            message: "Read project context for delivery report generation.".to_string(),
            payload_json: json!({
                "project_id": context.project.id,
                "assets": context.assets.len(),
                "storyboards": context.storyboards.len(),
                "prompt_packages": context.prompt_packages.len(),
            }),
        },
        DeliveryEventSpec {
            sequence_number: 2,
            role: "planner",
            event_type: "delivery_sections_resolved",
            message: "Resolved delivery report section status.".to_string(),
            payload_json: json!({"sections": sections}),
        },
        DeliveryEventSpec {
            sequence_number: 3,
            role: "reviewer",
            event_type: "delivery_report_drafted",
            message: "Drafted Markdown delivery report from saved project data.".to_string(),
            payload_json: json!({"markdown_chars": report.markdown.chars().count()}),
        },
        DeliveryEventSpec {
            sequence_number: 4,
            role: "planner",
            event_type: "delivery_package_previewed",
            message: "Prepared delivery package preview.".to_string(),
            payload_json: report.sections_json["package_preview"].clone(),
        },
        DeliveryEventSpec {
            sequence_number: 5,
            role: "memory_curator",
            event_type: "delivery_report_saved",
            message: "Saved delivery report for project handoff.".to_string(),
            payload_json: json!({"delivery_report_id": report.id}),
        },
    ];

    let mut created = Vec::new();
    for event in events {
        created.push(repo.create_agent_run_event(AgentRunEventCreate {
            agent_run_id: agent_run_id.to_string(),
            sequence_number: event.sequence_number,
            role: event.role.to_string(),
            event_type: event.event_type.to_string(),
            message: event.message,
            payload_json: event.payload_json,
        })?);
    }
    Ok(created)
}

fn build_plan_json(
    input: &DeliveryReportGenerationInput,
    sections: &[DeliveryReportSectionStatus],
    preview: &DeliveryPackagePreview,
) -> Value {
    json!({
        "stage": "0.18",
        "user_direction": input.user_direction,
        "sections": sections,
        "package_preview": preview,
    })
}

fn shot_number_lookup(storyboards: &[StoryboardWithShots]) -> HashMap<String, i64> {
    storyboards
        .iter()
        .flat_map(|item| item.shots.iter())
        .map(|shot| (shot.id.clone(), shot.shot_number))
        .collect()
}

fn prompt_source(package: &PromptPackage, shot_numbers: &HashMap<String, i64>) -> String {
    package
        .shot_id
        .as_ref()
        .and_then(|shot_id| shot_numbers.get(shot_id))
        .map(|shot_number| format!("Shot {}", shot_number))
        .unwrap_or_else(|| "Image brief".to_string())
}

fn first_prompt_line(prompt: &str) -> &str {
    prompt
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(str::trim)
        .unwrap_or("")
}

fn value_array(value: &Value) -> &[Value] {
    value.as_array().map(Vec::as_slice).unwrap_or(&[])
}

fn string_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn json_string<'a>(value: &'a Value, key: &str) -> &'a str {
    string_field(value, key).unwrap_or("")
}

fn join_json_strings(value: &Value) -> String {
    let values = value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .filter(|item| !item.trim().is_empty())
                .map(str::trim)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}

fn fallback<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.trim().is_empty() {
        fallback
    } else {
        value.trim()
    }
}

fn suffix_if_present(prefix: &str, value: &str) -> String {
    if value.trim().is_empty() {
        String::new()
    } else {
        format!("{prefix}{}", value.trim())
    }
}

fn table_cell(value: impl AsRef<str>) -> String {
    let sanitized = value
        .as_ref()
        .replace('|', "\\|")
        .replace('\n', " ")
        .trim()
        .to_string();
    if sanitized.is_empty() {
        "none".to_string()
    } else {
        sanitized
    }
}
