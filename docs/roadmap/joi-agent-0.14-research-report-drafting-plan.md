# Joi Agent 0.14 Research And Report Drafting Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let Joi start a project research task, collect source/citation metadata, draft a structured fashion advertising research report, persist it to the project, and record the work as Agent run events.

**Architecture:** 0.14 builds on the 0.13 Agent runtime. Joi keeps research data in the existing `research_reports` table, adds a `research` service for deterministic report drafting, and exposes commands/UI for a Research workspace. The default 0.14 source collector is user-assisted: users provide URLs, titles, excerpts, and notes; the code stores citation metadata and writes findings from that material while keeping an adapter boundary for later live web search/fetch.

**Tech Stack:** Tauri 2 commands, Rust, rusqlite, serde/serde_json, chrono, React 19, TypeScript, Vitest, Joi 0.13 `agent_runs` and `agent_run_events`.

---

## Product Outcome

After 0.14, a user can:

- Select a project with 0.12 brief/product understanding.
- Open the Research tab.
- Enter a research goal such as `Find fashion ad references for a 15s trench coat launch film`.
- Add source materials with title, URL, source type, and excerpt/notes.
- Generate a structured research report.
- See findings, sources, rationale, and creative implications.
- Save the report to the project.
- See a corresponding Agent run in the Agent panel with researcher events.
- Reopen the project and see saved research reports.

0.14 does not ship autonomous web crawling. It creates the product surface and data contract for research reports and citations. Later stages can replace the user-assisted source collector with live search/fetch behind the same input/output model.

## Scope

### In Scope

- Research report command layer:
  - `joi_generate_research_report`
  - `joi_list_research_reports`
- Research service:
  - reads Joi project context
  - normalizes source inputs
  - drafts findings
  - drafts creative implications
  - writes a report to `research_reports`
  - writes Agent run/events using 0.13 tables
- Repository upgrade:
  - allow `ResearchReportCreate` to persist `findings_json` and `sources_json`
- Frontend Research workspace:
  - research goal input
  - market/platform focus fields
  - source material form
  - generate report button
  - saved report list
  - latest report panel
- Tests and smoke report.

### Out Of Scope

- No autonomous browser automation inside the desktop app.
- No background long-running research jobs.
- No paid search API integration.
- No source scraping of arbitrary pages by default.
- No LLM call required for report drafting.
- No replacement of 0.13 Agent run model.

## Data Contract

### `ResearchReportCreate`

Change the existing repository create struct from:

```rust
pub struct ResearchReportCreate {
    pub project_id: String,
    pub summary: String,
}
```

to:

```rust
pub struct ResearchReportCreate {
    pub project_id: String,
    pub summary: String,
    pub findings_json: serde_json::Value,
    pub sources_json: serde_json::Value,
}
```

The existing `research_reports` table already has:

```sql
summary TEXT NOT NULL DEFAULT '',
findings_json TEXT NOT NULL DEFAULT '[]',
sources_json TEXT NOT NULL DEFAULT '[]',
```

No migration is required for the table shape.

### `ResearchSourceInput`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchSourceInput {
    pub title: String,
    pub url: String,
    pub source_type: String,
    pub excerpt: String,
}
```

Rules:

- `title` is required.
- `source_type` defaults in the frontend to `reference`.
- `url` may be empty only when the source is a manually captured offline note.
- `excerpt` is required because 0.14 does not fetch page content.

### `ResearchReportInput`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchReportInput {
    pub project_id: String,
    pub research_goal: String,
    pub market_focus: String,
    pub platform_focus: Vec<String>,
    pub source_materials: Vec<ResearchSourceInput>,
}
```

Validation:

- `project_id` must exist.
- `research_goal` must be non-empty after trim.
- `source_materials` must contain at least one source.
- Each source must have `title` and `excerpt`.

### `ResearchFinding`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchFinding {
    pub title: String,
    pub insight: String,
    pub evidence: String,
    pub source_index: usize,
    pub creative_implication: String,
}
```

### `ResearchSourceCitation`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchSourceCitation {
    pub index: usize,
    pub title: String,
    pub url: String,
    pub source_type: String,
    pub excerpt: String,
}
```

### `ResearchReportResult`

```rust
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
```

## Report JSON Shape

`findings_json` stores:

```json
[
  {
    "title": "Fabric texture should become a proof point",
    "insight": "Source excerpt emphasizes close tactile details.",
    "evidence": "Quoted or paraphrased source excerpt.",
    "source_index": 1,
    "creative_implication": "Use close-up fabric insert before the model walk."
  }
]
```

`sources_json` stores:

```json
[
  {
    "index": 1,
    "title": "Reference campaign note",
    "url": "https://example.com/reference",
    "source_type": "reference",
    "excerpt": "Short user-provided excerpt."
  }
]
```

`summary` should be readable as a report abstract:

```text
Research for Spring Drop Film: 2 source-backed findings for urban commuters, focused on jimeng_video and grok_video.
```

## Agent Events

Generating a report creates one Agent run:

- `status`: `completed`
- `runtime_kind`: `hermes_core`
- `runtime_mode`: `local_research_bridge`
- `roles_json`: `["researcher", "planner", "reviewer"]`

Expected events:

1. researcher `research_context_read`
2. researcher `sources_collected`
3. researcher `findings_drafted`
4. reviewer `citations_checked`
5. planner `report_saved`

## Implementation Tasks

### Task 1: Repository Support For Structured Research Reports

**Files:**

- Modify: `src-tauri/src/repositories.rs`
- Test: `src-tauri/tests/structured_content_repository.rs`

- [ ] **Step 1: Write failing repository test**

Add to `src-tauri/tests/structured_content_repository.rs`:

```rust
#[test]
fn stores_research_report_findings_and_sources() {
    let app = TestApp::new();
    let repo = Repository::new(app.db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".to_string(),
            description: "Premium womenswear".to_string(),
        })
        .unwrap();
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id,
            title: "Spring Drop Film".to_string(),
            advertising_goal: "Launch awareness".to_string(),
            duration_seconds: 15,
        })
        .unwrap();

    let report = repo
        .create_research_report(ResearchReportCreate {
            project_id: project.id.clone(),
            summary: "Research summary".to_string(),
            findings_json: serde_json::json!([
                {
                    "title": "Texture proof point",
                    "insight": "Fabric closeups should lead the edit",
                    "source_index": 1
                }
            ]),
            sources_json: serde_json::json!([
                {
                    "index": 1,
                    "title": "Reference note",
                    "url": "https://example.com/reference"
                }
            ]),
        })
        .unwrap();

    let reports = repo.list_research_reports(&project.id).unwrap();

    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].id, report.id);
    assert_eq!(reports[0].findings_json[0]["title"], "Texture proof point");
    assert_eq!(reports[0].sources_json[0]["title"], "Reference note");
}
```

- [ ] **Step 2: Run test and confirm RED**

Run:

```powershell
cd src-tauri
cargo test --test structured_content_repository stores_research_report_findings_and_sources -- --nocapture
```

Expected:

- Fails because `ResearchReportCreate` has no `findings_json` or `sources_json` fields.

- [ ] **Step 3: Extend repository create struct**

Modify `src-tauri/src/repositories.rs`:

```rust
#[derive(Debug, Clone)]
pub struct ResearchReportCreate {
    pub project_id: String,
    pub summary: String,
    pub findings_json: Value,
    pub sources_json: Value,
}
```

- [ ] **Step 4: Persist findings and sources**

Update `create_research_report`:

```rust
let report = ResearchReport {
    id: new_id(),
    project_id: input.project_id,
    summary: input.summary,
    findings_json: input.findings_json,
    sources_json: input.sources_json,
    created_at: now,
    updated_at: now,
};
```

Keep the existing SQL insert statement but pass:

```rust
report.findings_json.to_string(),
report.sources_json.to_string(),
```

- [ ] **Step 5: Update existing tests**

Existing tests that construct `ResearchReportCreate` must add:

```rust
findings_json: serde_json::json!([]),
sources_json: serde_json::json!([]),
```

Files likely affected:

- `src-tauri/tests/structured_content_repository.rs`
- `src-tauri/tests/project_snapshots.rs`

- [ ] **Step 6: Run repository tests**

Run:

```powershell
cd src-tauri
cargo test --test structured_content_repository -- --nocapture
cargo test --test project_snapshots -- --nocapture
```

Expected:

- Tests pass.

### Task 2: Research Report Drafting Service

**Files:**

- Create: `src-tauri/src/research.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/research.rs`

- [ ] **Step 1: Write failing service tests**

Create `src-tauri/tests/research.rs`:

```rust
mod common;

use common::TestApp;
use joi_agent_lib::db::Database;
use joi_agent_lib::repositories::{BrandCreate, ProductUnderstandingCreate, ProjectCreate, Repository};
use joi_agent_lib::research::{generate_research_report, ResearchReportInput, ResearchSourceInput};

fn migrated_repo() -> (TestApp, Database) {
    let app = TestApp::new();
    let db = Database::open(&app.db_path).expect("open database");
    db.migrate().expect("migrate");
    (app, db)
}

#[test]
fn generates_source_backed_research_report() {
    let (_app, db) = migrated_repo();
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".to_string(),
            description: "Contemporary womenswear".to_string(),
        })
        .unwrap();
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id,
            title: "Spring Drop Film".to_string(),
            advertising_goal: "Launch awareness".to_string(),
            duration_seconds: 15,
        })
        .unwrap();
    repo.create_product_understanding(ProductUnderstandingCreate {
        project_id: project.id.clone(),
        product_name: "Lightweight trench".to_string(),
        category: "outerwear".to_string(),
        audience: "urban commuters".to_string(),
        selling_points: vec!["water-resistant cotton".to_string()],
        constraints: vec!["avoid heavy winter styling".to_string()],
        notes: "Focus on fabric texture.".to_string(),
    })
    .unwrap();

    let result = generate_research_report(
        &repo,
        ResearchReportInput {
            project_id: project.id.clone(),
            research_goal: "Find visual references for a trench launch film".to_string(),
            market_focus: "urban commuter outerwear".to_string(),
            platform_focus: vec!["jimeng_video".to_string(), "grok_video".to_string()],
            source_materials: vec![ResearchSourceInput {
                title: "Reference campaign note".to_string(),
                url: "https://example.com/reference".to_string(),
                source_type: "reference".to_string(),
                excerpt: "Close fabric texture and walking movement made the product benefit clear.".to_string(),
            }],
        },
        "0.16.0".to_string(),
    )
    .unwrap();

    assert_eq!(result.report.project_id, project.id);
    assert_eq!(result.findings.len(), 1);
    assert_eq!(result.sources.len(), 1);
    assert!(result.rationale.contains("Lightweight trench"));
    assert!(result.creative_implications[0].contains("fabric"));
    assert_eq!(result.agent_events.len(), 5);
    assert_eq!(result.agent_events[1].event_type, "sources_collected");

    let reports = repo.list_research_reports(&project.id).unwrap();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].findings_json[0]["source_index"], 1);
}

#[test]
fn rejects_research_report_without_sources() {
    let (_app, db) = migrated_repo();
    let repo = Repository::new(db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".to_string(),
            description: String::new(),
        })
        .unwrap();
    let project = repo
        .create_project(ProjectCreate {
            brand_id: brand.id,
            title: "Spring Drop Film".to_string(),
            advertising_goal: "Launch awareness".to_string(),
            duration_seconds: 15,
        })
        .unwrap();

    let error = generate_research_report(
        &repo,
        ResearchReportInput {
            project_id: project.id,
            research_goal: "Find references".to_string(),
            market_focus: String::new(),
            platform_focus: Vec::new(),
            source_materials: Vec::new(),
        },
        "0.16.0".to_string(),
    )
    .expect_err("missing sources should fail");

    assert!(error.to_string().contains("at least one research source"));
}
```

- [ ] **Step 2: Run tests and confirm RED**

Run:

```powershell
cd src-tauri
cargo test --test research -- --nocapture
```

Expected:

- Fails because `research` module and functions do not exist.

- [ ] **Step 3: Implement research DTOs**

Create `src-tauri/src/research.rs` with the DTOs from the Data Contract section:

```rust
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::agent_context::build_project_context;
use crate::error::{JoiError, JoiResult};
use crate::models::{AgentRun, AgentRunEvent, ResearchReport};
use crate::repositories::{AgentRunCreate, AgentRunEventCreate, Repository, ResearchReportCreate};
use crate::validation::validate_required_text;
```

Add:

```rust
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
```

- [ ] **Step 4: Implement validation and citation normalization**

Add:

```rust
fn validate_input(input: &ResearchReportInput) -> JoiResult<()> {
    validate_required_text("Research goal", &input.research_goal)?;
    if input.source_materials.is_empty() {
        return Err(JoiError::Validation(
            "Research report requires at least one research source".to_string(),
        ));
    }
    for (index, source) in input.source_materials.iter().enumerate() {
        validate_required_text(&format!("Research source {} title", index + 1), &source.title)?;
        validate_required_text(&format!("Research source {} excerpt", index + 1), &source.excerpt)?;
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
```

- [ ] **Step 5: Implement deterministic finding writer**

Add:

```rust
fn build_findings(
    product_name: &str,
    input: &ResearchReportInput,
    sources: &[ResearchSourceCitation],
) -> Vec<ResearchFinding> {
    sources
        .iter()
        .map(|source| {
            let title = format!("{} insight from {}", product_name, source.title);
            let insight = format!(
                "For {}, the source suggests a usable fashion advertising angle: {}",
                input.research_goal,
                first_sentence(&source.excerpt)
            );
            let creative_implication = if source.excerpt.to_lowercase().contains("texture") {
                "Use tactile close-ups as visual proof before the model movement.".to_string()
            } else if source.excerpt.to_lowercase().contains("movement") {
                "Use model motion to demonstrate garment behavior in the first half of the film.".to_string()
            } else {
                "Translate the source observation into one clear shot requirement.".to_string()
            };
            ResearchFinding {
                title,
                insight,
                evidence: source.excerpt.clone(),
                source_index: source.index,
                creative_implication,
            }
        })
        .collect()
}

fn first_sentence(value: &str) -> String {
    value
        .split(['.', '。', '!', '！', '?', '？'])
        .map(str::trim)
        .find(|part| !part.is_empty())
        .unwrap_or(value.trim())
        .to_string()
}
```

- [ ] **Step 6: Implement report generation**

Add:

```rust
pub fn generate_research_report(
    repo: &Repository<'_>,
    input: ResearchReportInput,
    hermes_version: String,
) -> JoiResult<ResearchReportResult> {
    validate_input(&input)?;
    let context = build_project_context(repo, &input.project_id)?;
    let product_name = context
        .latest_product_understanding
        .as_ref()
        .map(|item| item.product_name.trim())
        .filter(|value| !value.is_empty())
        .unwrap_or("the product");
    let sources = normalize_sources(&input.source_materials);
    let findings = build_findings(product_name, &input, &sources);
    let creative_implications = findings
        .iter()
        .map(|finding| finding.creative_implication.clone())
        .collect::<Vec<_>>();
    let rationale = format!(
        "Research for {} uses {} source materials to support {}.",
        context.project.title,
        sources.len(),
        product_name
    );
    let summary = format!(
        "Research for {}: {} source-backed findings for {}, focused on {}.",
        context.project.title,
        findings.len(),
        product_name,
        if input.platform_focus.is_empty() {
            "general fashion advertising".to_string()
        } else {
            input.platform_focus.join(", ")
        }
    );
    let report = repo.create_research_report(ResearchReportCreate {
        project_id: input.project_id.clone(),
        summary,
        findings_json: json!(findings),
        sources_json: json!(sources),
    })?;
    let agent_run = repo.create_agent_run(AgentRunCreate {
        project_id: input.project_id,
        user_goal: input.research_goal,
        status: "completed".to_string(),
        runtime_kind: "hermes_core".to_string(),
        runtime_mode: "local_research_bridge".to_string(),
        runtime_version: hermes_version,
        roles_json: json!(["researcher", "planner", "reviewer"]),
        plan_json: json!([
            {"role":"researcher","stage":"0.14","title":"Collect source metadata"},
            {"role":"researcher","stage":"0.14","title":"Draft source-backed findings"},
            {"role":"reviewer","stage":"0.14","title":"Check citation coverage"},
            {"role":"planner","stage":"0.14","title":"Save report for creative workflow"}
        ]),
        result_summary: report.summary.clone(),
    })?;
    let agent_events = persist_research_events(repo, &agent_run.id, &context.project.title, sources.len(), findings.len(), &report.id)?;
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
```

- [ ] **Step 7: Persist research events**

Add:

```rust
fn persist_research_events(
    repo: &Repository<'_>,
    agent_run_id: &str,
    project_title: &str,
    source_count: usize,
    finding_count: usize,
    report_id: &str,
) -> JoiResult<Vec<AgentRunEvent>> {
    let specs = [
        ("researcher", "research_context_read", format!("Read research context for {}.", project_title), json!({"project_title": project_title})),
        ("researcher", "sources_collected", format!("Collected {} research sources.", source_count), json!({"source_count": source_count})),
        ("researcher", "findings_drafted", format!("Drafted {} source-backed findings.", finding_count), json!({"finding_count": finding_count})),
        ("reviewer", "citations_checked", "Checked that each finding references a source index.".to_string(), json!({"finding_count": finding_count})),
        ("planner", "report_saved", format!("Saved research report {}.", report_id), json!({"report_id": report_id})),
    ];
    let mut events = Vec::new();
    for (index, (role, event_type, message, payload_json)) in specs.into_iter().enumerate() {
        events.push(repo.create_agent_run_event(AgentRunEventCreate {
            agent_run_id: agent_run_id.to_string(),
            sequence_number: (index + 1) as i64,
            role: role.to_string(),
            event_type: event_type.to_string(),
            message,
            payload_json,
        })?);
    }
    Ok(events)
}
```

- [ ] **Step 8: Register module**

Modify `src-tauri/src/lib.rs`:

```rust
pub mod research;
```

- [ ] **Step 9: Run research tests**

Run:

```powershell
cd src-tauri
cargo test --test research -- --nocapture
```

Expected:

- Research service tests pass.

### Task 3: Tauri Commands For Research

**Files:**

- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/commands.rs`

- [ ] **Step 1: Write failing command test**

Add imports in `src-tauri/tests/commands.rs`:

```rust
use joi_agent_lib::commands::{generate_research_report, list_research_reports};
use joi_agent_lib::research::{ResearchReportInput, ResearchSourceInput};
```

Add test:

```rust
#[test]
fn state_helpers_generate_and_list_research_reports() {
    let (_app, state) = test_state();
    let brand = create_brand(
        &state,
        BrandInput {
            name: "Atelier Joi".to_string(),
            description: "Contemporary womenswear".to_string(),
        },
    )
    .unwrap();
    let project = create_project(
        &state,
        ProjectInput {
            brand_id: brand.id,
            title: "Spring Drop Film".to_string(),
            advertising_goal: "Launch awareness".to_string(),
            duration_seconds: 15,
        },
    )
    .unwrap();

    let result = generate_research_report(
        &state,
        ResearchReportInput {
            project_id: project.id.clone(),
            research_goal: "Find reference angles".to_string(),
            market_focus: "outerwear".to_string(),
            platform_focus: vec!["jimeng_video".to_string()],
            source_materials: vec![ResearchSourceInput {
                title: "Reference note".to_string(),
                url: "https://example.com/reference".to_string(),
                source_type: "reference".to_string(),
                excerpt: "Texture details support premium positioning.".to_string(),
            }],
        },
    )
    .unwrap();

    assert_eq!(result.findings.len(), 1);
    assert_eq!(result.agent_events.len(), 5);

    let reports = list_research_reports(&state, project.id).unwrap();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].id, result.report.id);
}
```

- [ ] **Step 2: Run command test and confirm RED**

Run:

```powershell
cd src-tauri
cargo test --test commands state_helpers_generate_and_list_research_reports -- --nocapture
```

Expected:

- Fails because command helpers do not exist.

- [ ] **Step 3: Add command handlers**

In `src-tauri/src/commands.rs`, import:

```rust
use crate::research::{
    generate_research_report as generate_research_report_service, ResearchReportInput,
    ResearchReportResult,
};
```

Add Tauri commands:

```rust
#[tauri::command(rename_all = "snake_case")]
pub fn joi_generate_research_report(
    state: State<'_, AppState>,
    input: ResearchReportInput,
) -> JoiResult<ResearchReportResult> {
    generate_research_report(state.inner(), input)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_list_research_reports(
    state: State<'_, AppState>,
    project_id: String,
) -> JoiResult<Vec<ResearchReport>> {
    list_research_reports(state.inner(), project_id)
}
```

Add helper functions:

```rust
pub fn generate_research_report(
    state: &AppState,
    input: ResearchReportInput,
) -> JoiResult<ResearchReportResult> {
    let runtime_status = get_agent_runtime_status(state)?;
    let db = lock_db(state)?;
    generate_research_report_service(
        &Repository::new(db.connection()),
        input,
        runtime_status.hermes_version,
    )
}

pub fn list_research_reports(state: &AppState, project_id: String) -> JoiResult<Vec<ResearchReport>> {
    let db = lock_db(state)?;
    Repository::new(db.connection()).list_research_reports(&project_id)
}
```

- [ ] **Step 4: Register commands**

In `src-tauri/src/lib.rs`, register:

```rust
commands::joi_generate_research_report,
commands::joi_list_research_reports,
```

- [ ] **Step 5: Run command tests**

Run:

```powershell
cd src-tauri
cargo test --test commands state_helpers_generate_and_list_research_reports -- --nocapture
cargo test --test commands -- --nocapture
```

Expected:

- Command tests pass.

### Task 4: Frontend Research Workspace

**Files:**

- Create: `src/components/ResearchWorkspace.tsx`
- Modify: `src/types/joi.ts`
- Modify: `src/api/joiApi.ts`
- Modify: `src/App.tsx`
- Modify: `src/components/ProjectWorkspace.tsx`
- Modify: `src/App.test.tsx`

- [ ] **Step 1: Add frontend types**

Add to `src/types/joi.ts`:

```ts
export type ResearchReport = {
  id: string;
  project_id: string;
  summary: string;
  findings_json: unknown;
  sources_json: unknown;
  created_at: string;
  updated_at: string;
};

export type ResearchSourceInput = {
  title: string;
  url: string;
  source_type: string;
  excerpt: string;
};

export type ResearchReportInput = {
  project_id: string;
  research_goal: string;
  market_focus: string;
  platform_focus: string[];
  source_materials: ResearchSourceInput[];
};

export type ResearchFinding = {
  title: string;
  insight: string;
  evidence: string;
  source_index: number;
  creative_implication: string;
};

export type ResearchSourceCitation = {
  index: number;
  title: string;
  url: string;
  source_type: string;
  excerpt: string;
};

export type ResearchReportResult = {
  report: ResearchReport;
  findings: ResearchFinding[];
  sources: ResearchSourceCitation[];
  rationale: string;
  creative_implications: string[];
  agent_run: AgentRun;
  agent_events: AgentRunEvent[];
};
```

- [ ] **Step 2: Add API wrappers**

Add to `src/api/joiApi.ts`:

```ts
export function generateResearchReport(input: ResearchReportInput): Promise<ResearchReportResult> {
  return invoke<ResearchReportResult>("joi_generate_research_report", { input });
}

export function listResearchReports(projectId: string): Promise<ResearchReport[]> {
  return invoke<ResearchReport[]>("joi_list_research_reports", { project_id: projectId });
}
```

- [ ] **Step 3: Add failing UI test**

In `src/App.test.tsx`, add mock cases:

```ts
case "joi_list_research_reports":
  return Promise.resolve([]);
case "joi_generate_research_report":
  return Promise.resolve({
    report: {
      id: "research-1",
      project_id: "project-1",
      summary: "Research for Spring Drop Film: 1 source-backed finding.",
      findings_json: [],
      sources_json: [],
      created_at: "2026-06-15T00:00:00Z",
      updated_at: "2026-06-15T00:00:00Z",
    },
    findings: [
      {
        title: "Texture proof point",
        insight: "Texture details support premium positioning.",
        evidence: "Texture details support premium positioning.",
        source_index: 1,
        creative_implication: "Use tactile close-ups as visual proof before the model movement.",
      },
    ],
    sources: [
      {
        index: 1,
        title: "Reference note",
        url: "https://example.com/reference",
        source_type: "reference",
        excerpt: "Texture details support premium positioning.",
      },
    ],
    rationale: "Research for Spring Drop Film uses 1 source materials.",
    creative_implications: ["Use tactile close-ups as visual proof before the model movement."],
    agent_run: {
      id: "run-research-1",
      project_id: "project-1",
      user_goal: "Find reference angles",
      status: "completed",
      runtime_kind: "hermes_core",
      runtime_mode: "local_research_bridge",
      runtime_version: "0.16.0",
      roles_json: ["researcher", "planner", "reviewer"],
      plan_json: [],
      result_summary: "Research for Spring Drop Film: 1 source-backed finding.",
      created_at: "2026-06-15T00:00:00Z",
      updated_at: "2026-06-15T00:00:00Z",
    },
    agent_events: [],
  });
```

Add test:

```ts
test("generates a research report from the Research workspace", async () => {
  render(<App />);

  await screen.findByRole("heading", { name: "Spring Drop Film" });
  fireEvent.click(screen.getByRole("button", { name: "Research" }));
  fireEvent.change(screen.getByLabelText("Research goal"), {
    target: { value: "Find reference angles" },
  });
  fireEvent.change(screen.getByLabelText("Market focus"), {
    target: { value: "outerwear" },
  });
  fireEvent.change(screen.getByLabelText("Platform focus"), {
    target: { value: "jimeng_video, grok_video" },
  });
  fireEvent.change(screen.getByLabelText("Source title"), {
    target: { value: "Reference note" },
  });
  fireEvent.change(screen.getByLabelText("Source URL"), {
    target: { value: "https://example.com/reference" },
  });
  fireEvent.change(screen.getByLabelText("Source excerpt"), {
    target: { value: "Texture details support premium positioning." },
  });
  fireEvent.click(screen.getByRole("button", { name: /generate research report/i }));

  await waitFor(() => {
    expect(invokeMock).toHaveBeenCalledWith("joi_generate_research_report", {
      input: {
        project_id: "project-1",
        research_goal: "Find reference angles",
        market_focus: "outerwear",
        platform_focus: ["jimeng_video", "grok_video"],
        source_materials: [
          {
            title: "Reference note",
            url: "https://example.com/reference",
            source_type: "reference",
            excerpt: "Texture details support premium positioning.",
          },
        ],
      },
    });
  });
  expect(await screen.findByText("Texture proof point")).toBeInTheDocument();
});
```

- [ ] **Step 4: Run UI test and confirm RED**

Run:

```powershell
npm test -- src/App.test.tsx
```

Expected:

- Fails because Research workspace is still an empty reserved section.

- [ ] **Step 5: Create ResearchWorkspace component**

Create `src/components/ResearchWorkspace.tsx`:

```tsx
import type {
  Project,
  ResearchReport,
  ResearchReportResult,
  ResearchSourceInput,
} from "../types/joi";

export type ResearchDraft = {
  research_goal: string;
  market_focus: string;
  platform_focus_text: string;
  source_title: string;
  source_url: string;
  source_type: string;
  source_excerpt: string;
};

type ResearchWorkspaceProps = {
  generatingResearch: boolean;
  onResearchDraftChange: (field: keyof ResearchDraft, value: string) => void;
  onSubmitResearchReport: () => void;
  researchDraft: ResearchDraft;
  researchReports: ResearchReport[];
  researchResult: ResearchReportResult | null;
  selectedProject: Project | null;
};

export function ResearchWorkspace({
  generatingResearch,
  onResearchDraftChange,
  onSubmitResearchReport,
  researchDraft,
  researchReports,
  researchResult,
  selectedProject,
}: ResearchWorkspaceProps) {
  return (
    <div className="workspace-grid">
      <section className="workspace-panel wide">
        <h2>Research Brief</h2>
        <form className="brief-form" onSubmit={submit(onSubmitResearchReport)}>
          <label className="wide-field">
            Research goal
            <textarea
              disabled={!selectedProject || generatingResearch}
              onChange={(event) => onResearchDraftChange("research_goal", event.target.value)}
              rows={3}
              value={researchDraft.research_goal}
            />
          </label>
          <label>
            Market focus
            <input
              disabled={!selectedProject || generatingResearch}
              onChange={(event) => onResearchDraftChange("market_focus", event.target.value)}
              value={researchDraft.market_focus}
            />
          </label>
          <label>
            Platform focus
            <input
              disabled={!selectedProject || generatingResearch}
              onChange={(event) => onResearchDraftChange("platform_focus_text", event.target.value)}
              placeholder="jimeng_video, grok_video"
              value={researchDraft.platform_focus_text}
            />
          </label>
          <label>
            Source title
            <input
              disabled={!selectedProject || generatingResearch}
              onChange={(event) => onResearchDraftChange("source_title", event.target.value)}
              value={researchDraft.source_title}
            />
          </label>
          <label>
            Source URL
            <input
              disabled={!selectedProject || generatingResearch}
              onChange={(event) => onResearchDraftChange("source_url", event.target.value)}
              value={researchDraft.source_url}
            />
          </label>
          <label className="wide-field">
            Source excerpt
            <textarea
              disabled={!selectedProject || generatingResearch}
              onChange={(event) => onResearchDraftChange("source_excerpt", event.target.value)}
              rows={4}
              value={researchDraft.source_excerpt}
            />
          </label>
          <button disabled={!selectedProject || generatingResearch} type="submit">
            {generatingResearch ? "Generating" : "Generate Research Report"}
          </button>
        </form>
      </section>
      <section className="workspace-panel">
        <h2>Latest Research</h2>
        {researchResult ? (
          <div className="understanding-result">
            <p>{researchResult.report.summary}</p>
            <div className="data-list">
              {researchResult.findings.map((finding) => (
                <article className="data-row" key={`${finding.source_index}-${finding.title}`}>
                  <strong>{finding.title}</strong>
                  <span>{finding.insight}</span>
                  <small>{finding.creative_implication}</small>
                </article>
              ))}
            </div>
          </div>
        ) : (
          <p className="muted">Generated research findings will appear here.</p>
        )}
      </section>
      <section className="workspace-panel">
        <h2>Saved Reports</h2>
        <div className="data-list">
          {researchReports.length === 0 ? (
            <p className="muted">No research reports yet.</p>
          ) : (
            researchReports.map((report) => (
              <article className="data-row" key={report.id}>
                <strong>{report.summary}</strong>
                <small>{report.created_at}</small>
              </article>
            ))
          )}
        </div>
      </section>
    </div>
  );
}

export function researchSourceFromDraft(draft: ResearchDraft): ResearchSourceInput {
  return {
    title: draft.source_title,
    url: draft.source_url,
    source_type: draft.source_type || "reference",
    excerpt: draft.source_excerpt,
  };
}

function submit(action: () => void) {
  return (event: React.FormEvent) => {
    event.preventDefault();
    action();
  };
}
```

- [ ] **Step 6: Wire App state**

In `src/App.tsx`, add:

```ts
const emptyResearchDraft: ResearchDraft = {
  research_goal: "",
  market_focus: "",
  platform_focus_text: "",
  source_title: "",
  source_url: "",
  source_type: "reference",
  source_excerpt: "",
};
```

State:

```ts
const [generatingResearch, setGeneratingResearch] = useState(false);
const [researchDraft, setResearchDraft] = useState<ResearchDraft>(emptyResearchDraft);
const [researchReports, setResearchReports] = useState<ResearchReport[]>([]);
const [researchResult, setResearchResult] = useState<ResearchReportResult | null>(null);
```

In `refreshProjectState`, add `listResearchReports(projectId)` and set `researchReports`.

Add submit handler:

```ts
async function submitResearchReport() {
  if (!selectedProject) {
    setError("Select a project before generating research.");
    return;
  }
  if (!researchDraft.research_goal.trim()) {
    setError("Research goal is required.");
    return;
  }
  if (!researchDraft.source_title.trim() || !researchDraft.source_excerpt.trim()) {
    setError("Source title and excerpt are required.");
    return;
  }

  try {
    setGeneratingResearch(true);
    setError(null);
    const result = await generateResearchReport({
      project_id: selectedProject.id,
      research_goal: researchDraft.research_goal,
      market_focus: researchDraft.market_focus,
      platform_focus: splitListText(researchDraft.platform_focus_text),
      source_materials: [researchSourceFromDraft(researchDraft)],
    });
    setResearchResult(result);
    await refreshProjectState(selectedProject.id);
    setActivityLog((entries) => [...entries, `Generated research report ${result.report.id}.`]);
  } catch (submitError) {
    setError(formatError(submitError));
  } finally {
    setGeneratingResearch(false);
  }
}
```

Pass props through `ProjectWorkspace`.

- [ ] **Step 7: Render Research tab**

In `src/components/ProjectWorkspace.tsx`, import `ResearchWorkspace` and render when `activeTab === "Research"`:

```tsx
{activeTab === "Research" ? (
  <ResearchWorkspace
    generatingResearch={generatingResearch}
    onResearchDraftChange={onResearchDraftChange}
    onSubmitResearchReport={onSubmitResearchReport}
    researchDraft={researchDraft}
    researchReports={researchReports}
    researchResult={researchResult}
    selectedProject={selectedProject}
  />
) : null}
```

Update the empty-state condition to exclude `"Research"`.

- [ ] **Step 8: Run frontend tests and build**

Run:

```powershell
npm test
npm run build
```

Expected:

- Tests and build pass.

### Task 5: Smoke, Commit, Merge, Push

**Files:**

- Create: `docs/superpowers/reports/joi-0.14-research-report-smoke-test.md`
- Commit all 0.14 implementation files.

- [ ] **Step 1: Run full verification**

Run:

```powershell
npm test
npm run build
cd src-tauri
cargo test
cargo test --test commands -- --nocapture
cargo test --test research -- --nocapture
```

Expected:

- All commands pass.

- [ ] **Step 2: Browser smoke**

Run:

```powershell
npm run dev -- --host 127.0.0.1 --port 1420
```

Use the in-app browser to verify:

- Research tab renders a real form, not the reserved empty state.
- Fields render:
  - Research goal
  - Market focus
  - Platform focus
  - Source title
  - Source URL
  - Source excerpt
- Saved Reports panel renders.
- Desktop layout has no horizontal overflow.
- Mobile layout has no horizontal overflow.

Normal browser limitation:

- A normal browser still cannot call Tauri `invoke`; command integration is covered by Rust and React tests.

- [ ] **Step 3: Write smoke report**

Create `docs/superpowers/reports/joi-0.14-research-report-smoke-test.md` with:

- automated commands run
- browser observations
- acceptance checklist
- any known limitations

- [ ] **Step 4: Commit 0.14 implementation**

Run:

```powershell
git status --short
git add <0.14 files>
git commit -m "feat: add Joi 0.14 research reports"
```

- [ ] **Step 5: Merge to main**

Run from repository root:

```powershell
git checkout main
git merge --ff-only codex/joi-0.14-research-report
```

- [ ] **Step 6: Verify on main**

Run:

```powershell
npm test
npm run build
cd src-tauri
cargo test
```

- [ ] **Step 7: Push**

Run:

```powershell
git push origin main
```

If HTTPS Git push fails but GitHub API and SSH are reachable, use the same temporary repo deploy key fallback used for 0.13:

- create a temporary ed25519 key in the system temp directory
- add it as a writable deploy key to `Drew1811266/Joi-agent`
- push over `ssh.github.com:443`
- delete the deploy key
- delete the local temp key
- verify remote main SHA through GitHub API

## Acceptance Criteria

0.14 is complete only when:

- User can open a real Research workspace.
- User can provide research goal, market/platform focus, and source material.
- Joi generates a structured report with findings, sources, rationale, and creative implications.
- Report is persisted to `research_reports`.
- `findings_json` and `sources_json` contain useful structured data.
- Research generation creates an Agent run.
- Agent run includes ordered researcher/reviewer/planner events.
- Saved reports reload with the project.
- Tests cover repository, research service, commands, and frontend flow.
- Browser smoke report is written.
- Changes are merged to `main` and pushed to GitHub.

## Risks And Mitigations

### Risk: Users Expect Fully Automated Web Research

Mitigation:

- Label 0.14 as source-assisted research.
- Keep source URL and excerpt metadata explicit.
- Preserve adapter boundaries so 0.15+ can add live search/fetch without changing report storage.

### Risk: Deterministic Report Writing Feels Too Generic

Mitigation:

- Use project title, product name, audience, platform focus, and source excerpts in every report.
- Keep findings source-backed and tied to source indexes.
- Use creative implications as the main product value, not generic summaries.

### Risk: Existing Snapshot Tests Break

Mitigation:

- Update all `ResearchReportCreate` call sites with empty JSON arrays.
- Verify `project_snapshots` still includes research reports.

### Risk: Research Tab Becomes Too Large

Mitigation:

- 0.14 supports one source input in the first UI.
- Backend accepts multiple source materials so the model is future-ready.
- Multi-source frontend editing can be added later without changing command contracts.

## Handoff To 0.15

0.15 should use reports from 0.14 as durable memory candidates:

- Convert stable findings and creative implications into proposed project memory.
- Let users accept/reject memory entries.
- Feed accepted research memory into storyboard and prompt generation.

## Self-Review

- Spec coverage: This plan covers the 0.14 roadmap scope: research task start, source metadata, citations, structured findings, creative implications, report persistence, and Agent event tracking.
- Placeholder scan: No task contains TBD/TODO/fill-in instructions. Commands, structs, files, tests, and expected outputs are named concretely.
- Type consistency: `ResearchReportInput`, `ResearchSourceInput`, `ResearchFinding`, `ResearchSourceCitation`, `ResearchReportResult`, `ResearchReport`, and command names are consistent across backend, frontend, and tests.
