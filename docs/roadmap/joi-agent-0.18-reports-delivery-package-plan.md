# Joi Agent 0.18 Reports And Delivery Package Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the structured Joi project record into an editable delivery report and an exportable delivery package that includes the current `.joi-project.json`, assets, and Markdown report.

**Architecture:** 0.18 adds a persisted `delivery_reports` aggregate and a local deterministic report generator. The generator consumes the existing snapshot-shaped project context, including 0.17 prompt packages, then produces Markdown plus structured section metadata. The existing project package exporter is extended to include a selected delivery report as a Markdown file and package manifest metadata.

**Tech Stack:** Tauri 2 commands, Rust, rusqlite, serde/serde_json, chrono, React 19, TypeScript, Vitest, Joi local repository, Joi 0.13 Agent run model, Joi 0.17 prompt packages, existing `.joi-project.json` export service.

---

## Product Outcome

After 0.18, a user can:

- Open a `Delivery` workspace tab for a selected project.
- Generate a complete Markdown delivery report from the current project context.
- See report sections for brief, brand, product, research, creative direction, storyboard, prompt packages, assets, and version notes.
- Edit and save the generated Markdown report.
- Preview delivery package contents before export.
- Export a package that includes:
  - `{project-slug}.joi-project.json`
  - `{project-slug}-assets/`
  - `{project-slug}-delivery-report.md`
- See delivery report generation logged as an Agent run/event sequence.

0.18 does not generate PDF, PPTX, cloud share links, provider-native upload packages, or team review permissions.

## Scope

### In Scope

- Delivery report persistence:
  - report title
  - Markdown body
  - structured section metadata
  - final candidate flag
  - timestamps
- Delivery report generator:
  - deterministic Markdown composition from saved project context
  - project brief summary
  - brand summary
  - product understanding
  - research findings and sources
  - creative direction
  - storyboard table
  - prompt package table grouped by modality/platform
  - asset list
  - version notes
  - missing-section warnings
  - Agent run/events
- Command layer:
  - `joi_generate_delivery_report`
  - `joi_list_delivery_reports`
  - `joi_update_delivery_report`
  - `joi_preview_delivery_package`
  - extend `joi_export_project` with optional delivery report export
- Frontend Delivery workspace:
  - generate report button
  - editable Markdown textarea
  - report list
  - section coverage indicators
  - package preview
  - export directory field
  - export package button
- Tests and smoke report.

### Out Of Scope

- No PDF export.
- No PowerPoint export.
- No advanced visual page layout.
- No external storage upload.
- No team permissions or approval workflow.
- No provider-specific handoff formats beyond Markdown and Joi project package JSON.

## Existing Code Context

Use these current pieces:

- `src-tauri/src/snapshots.rs`
  - `ProjectSnapshotService::build_snapshot(project_id)` already returns a full project context JSON.
  - Snapshot currently includes brand, project, assets, research reports, product understandings, creative directions, storyboards, prompt packages, and memory entries.
- `src-tauri/src/project_package.rs`
  - `ProjectPackageService::export_project` already writes `{slug}.joi-project.json` and copies managed assets.
  - `ProjectExportResult` currently returns `project_json_path` and `assets_dir`.
- `src-tauri/src/repositories.rs`
  - repository already has list methods for all context aggregates.
  - add delivery report CRUD here.
- `src-tauri/src/commands.rs`
  - project export command already exists.
  - add delivery report commands and optional export input fields.
- `src/components/BrandProjectRail.tsx`
  - workflow tab list currently omits `Delivery`.
- `src/components/ProjectWorkspace.tsx`
  - workflow map already displays `Delivery`.
  - add `DeliveryWorkspace` rendering.
- `src/App.tsx`
  - project refresh currently loads assets, versions, memory, understandings, creative directions, reports, storyboards, prompt packages, and agent runs.
  - add delivery reports and package preview state.

## Data Contract

### Delivery Report Table

Create `delivery_reports`:

```sql
CREATE TABLE IF NOT EXISTS delivery_reports (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  markdown TEXT NOT NULL,
  sections_json TEXT NOT NULL,
  is_final_candidate INTEGER NOT NULL DEFAULT 0 CHECK (is_final_candidate IN (0, 1)),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_delivery_reports_project_id
  ON delivery_reports(project_id);
```

Rules:

- `project_id` must exist.
- `title` must be non-empty.
- `markdown` must be non-empty.
- `sections_json` must be an object with `format_version = "joi.delivery_report_sections.v1"`.
- Only one final candidate per project is preferred but not required in 0.18. If implemented, make it repository-level behavior, not a database trigger.

### Rust Models

Add to `src-tauri/src/models.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryReport {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub markdown: String,
    pub sections_json: Value,
    pub is_final_candidate: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Repository Types

Add to `src-tauri/src/repositories.rs`:

```rust
#[derive(Debug, Clone)]
pub struct DeliveryReportCreate {
    pub project_id: String,
    pub title: String,
    pub markdown: String,
    pub sections_json: Value,
    pub is_final_candidate: bool,
}

#[derive(Debug, Clone)]
pub struct DeliveryReportUpdate {
    pub id: String,
    pub title: String,
    pub markdown: String,
    pub sections_json: Value,
    pub is_final_candidate: bool,
}
```

Repository methods:

```rust
pub fn create_delivery_report(&self, input: DeliveryReportCreate) -> JoiResult<DeliveryReport>;
pub fn get_delivery_report(&self, id: &str) -> JoiResult<DeliveryReport>;
pub fn list_delivery_reports(&self, project_id: &str) -> JoiResult<Vec<DeliveryReport>>;
pub fn update_delivery_report(&self, input: DeliveryReportUpdate) -> JoiResult<DeliveryReport>;
```

### Delivery Report Section Metadata

`sections_json` shape:

```json
{
  "format_version": "joi.delivery_report_sections.v1",
  "sections": [
    {
      "id": "project_brief",
      "title": "Project Brief",
      "status": "complete",
      "source_count": 1,
      "warning": ""
    },
    {
      "id": "prompt_packages",
      "title": "Prompt Packages",
      "status": "complete",
      "source_count": 5,
      "warning": ""
    }
  ],
  "package_preview": {
    "markdown_file_name": "spring-drop-film-delivery-report.md",
    "project_json_file_name": "spring-drop-film.joi-project.json",
    "assets_folder_name": "spring-drop-film-assets"
  }
}
```

Allowed section statuses:

- `complete`
- `partial`
- `missing`

Required section IDs:

- `project_brief`
- `brand_understanding`
- `product_understanding`
- `research_findings`
- `creative_direction`
- `storyboard`
- `prompt_packages`
- `assets`
- `version_notes`

### Delivery Report Generation Types

Create `src-tauri/src/delivery_report.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeliveryReportGenerationInput {
    pub project_id: String,
    pub user_direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
```

### Command Input Types

Add to `src-tauri/src/commands.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeliveryReportUpdateInput {
    pub id: String,
    pub title: String,
    pub markdown: String,
    pub sections_json: serde_json::Value,
    pub is_final_candidate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeliveryPackagePreviewInput {
    pub project_id: String,
    pub delivery_report_id: Option<String>,
}
```

Extend existing `ProjectExportCommandInput`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectExportCommandInput {
    pub project_id: String,
    pub export_dir: PathBuf,
    pub delivery_report_id: Option<String>,
}
```

Backwards compatibility:

- Existing JSON without `delivery_report_id` must still deserialize.
- Exporting without a report keeps the existing package behavior.

### Export Result Contract

Extend `ProjectExportResult`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectExportResult {
    pub project_json_path: PathBuf,
    pub assets_dir: PathBuf,
    pub delivery_report_path: Option<PathBuf>,
}
```

The command return should serialize `delivery_report_path` as `null` when no report is exported.

When a delivery report is exported:

- write `{slug}-delivery-report.md`
- add to package JSON:

```json
{
  "delivery_report": {
    "id": "report-1",
    "title": "Spring Drop Film Delivery Report",
    "markdown_file": "spring-drop-film-delivery-report.md"
  }
}
```

## Markdown Report Contract

Generated Markdown must use this section order:

```md
# {Project Title} Delivery Report

## Project Brief

## Brand Understanding

## Product Understanding

## Research Findings

## Creative Direction

## Storyboard

| Shot | Duration | Visual | Action | Camera | Garment | Text |

## Prompt Packages

| Platform | Modality | Source | Prompt Summary | Missing Fields |

## Assets

## Version Notes

## Export Notes
```

Rules:

- Do not invent missing source material.
- For missing sections, write a short warning line, for example:
  - `No saved research report yet.`
- Prompt package rows must include platform, modality, source (`Shot N` or `Image brief`), first prompt line, and missing fields.
- Research findings should include source title or URL when available in `sources_json`.
- Storyboard rows should be sorted by storyboard creation order then shot number.
- Markdown must be deterministic for the same saved project state.

## Implementation Tasks

### Task 1: Delivery Report Data Model And Repository

**Files:**

- Modify: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/repositories.rs`
- Test: `src-tauri/tests/db_migration.rs`
- Test: `src-tauri/tests/structured_content_repository.rs`
- Test: `src-tauri/tests/project_snapshots.rs`

- [ ] **Step 1: Write failing migration test**

Add to `src-tauri/tests/db_migration.rs`:

```rust
#[test]
fn migration_creates_delivery_reports_table() {
    let db = migrated_in_memory_database();
    let columns = table_columns(&db, "delivery_reports");

    assert!(columns.contains(&"id".to_string()));
    assert!(columns.contains(&"project_id".to_string()));
    assert!(columns.contains(&"title".to_string()));
    assert!(columns.contains(&"markdown".to_string()));
    assert!(columns.contains(&"sections_json".to_string()));
    assert!(columns.contains(&"is_final_candidate".to_string()));
}
```

Run:

```powershell
cd src-tauri
cargo test --test db_migration migration_creates_delivery_reports_table -- --nocapture
```

Expected RED:

- table does not exist.

- [ ] **Step 2: Add schema and model**

Modify `src-tauri/src/db.rs` to create `delivery_reports`.

Modify `src-tauri/src/models.rs` to add `DeliveryReport`.

- [ ] **Step 3: Write failing repository tests**

Add to `src-tauri/tests/structured_content_repository.rs`:

```rust
#[test]
fn creates_lists_and_updates_delivery_reports() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());
    let project_id = seed_project(&repo);

    let report = repo
        .create_delivery_report(DeliveryReportCreate {
            project_id: project_id.clone(),
            title: "Launch Film Delivery Report".into(),
            markdown: "# Launch Film Delivery Report".into(),
            sections_json: json!({
                "format_version": "joi.delivery_report_sections.v1",
                "sections": []
            }),
            is_final_candidate: false,
        })
        .expect("create report");

    assert_eq!(report.project_id, project_id);
    assert_eq!(report.title, "Launch Film Delivery Report");

    let reports = repo.list_delivery_reports(&project_id).expect("reports");
    assert_eq!(reports.len(), 1);

    let updated = repo
        .update_delivery_report(DeliveryReportUpdate {
            id: report.id.clone(),
            title: "Edited Report".into(),
            markdown: "# Edited Report".into(),
            sections_json: json!({
                "format_version": "joi.delivery_report_sections.v1",
                "sections": [{"id": "project_brief", "status": "complete"}]
            }),
            is_final_candidate: true,
        })
        .expect("update report");

    assert_eq!(updated.title, "Edited Report");
    assert!(updated.is_final_candidate);
}
```

Add validation test:

```rust
#[test]
fn rejects_blank_delivery_report_markdown() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());
    let project_id = seed_project(&repo);

    let result = repo.create_delivery_report(DeliveryReportCreate {
        project_id,
        title: "Report".into(),
        markdown: "   ".into(),
        sections_json: json!({"format_version": "joi.delivery_report_sections.v1"}),
        is_final_candidate: false,
    });

    assert!(result.is_err());
}
```

Run:

```powershell
cargo test --test structured_content_repository delivery_report -- --nocapture
```

Expected RED:

- repository types and methods do not exist.

- [ ] **Step 4: Implement repository**

Add CRUD methods and row mapper:

```rust
fn map_delivery_report(row: &rusqlite::Row<'_>) -> rusqlite::Result<DeliveryReport>;
```

Validation:

- title non-empty
- markdown non-empty
- sections format version present
- project exists via FK

- [ ] **Step 5: Snapshot support**

Modify `ProjectSnapshotService::build_snapshot` to include:

```json
"delivery_reports": repo.list_delivery_reports(project_id)?
```

Add assertion in `src-tauri/tests/project_snapshots.rs` that delivery reports appear in snapshots.

### Task 2: Delivery Report Generator Service

**Files:**

- Create: `src-tauri/src/delivery_report.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/delivery_report.rs`

- [ ] **Step 1: Write failing generator test**

Create `src-tauri/tests/delivery_report.rs`:

```rust
#[test]
fn generates_delivery_report_from_full_project_context() {
    let app = TestApp::new();
    let state = app_state(&app);
    let project_id = seed_full_delivery_project(&state);

    let result = generate_delivery_report(
        &state,
        DeliveryReportGenerationInput {
            project_id: project_id.clone(),
            user_direction: "Keep the handoff concise.".into(),
        },
    )
    .expect("report");

    assert_eq!(result.report.project_id, project_id);
    assert!(result.report.markdown.contains("# Launch Film Delivery Report"));
    assert!(result.report.markdown.contains("## Storyboard"));
    assert!(result.report.markdown.contains("## Prompt Packages"));
    assert!(result.report.markdown.contains("jimeng_video"));
    assert!(result.report.markdown.contains("gpt_image_2"));
    assert!(result.package_preview.delivery_report_file_name.ends_with("-delivery-report.md"));
    assert_eq!(result.agent_run.runtime_mode, "local_delivery_report_bridge");
    assert!(result.agent_events.len() >= 4);
}
```

Run:

```powershell
cargo test --test delivery_report -- --nocapture
```

Expected RED:

- delivery report service does not exist.

- [ ] **Step 2: Implement section status builder**

Add helper functions:

```rust
fn section_status(id: &str, title: &str, source_count: usize, warning: &str) -> DeliveryReportSectionStatus;
fn sections_json(sections: &[DeliveryReportSectionStatus], preview: &DeliveryPackagePreview) -> Value;
```

Status rules:

- `complete` when source count > 0 and no warning.
- `partial` when source count > 0 and warning is non-empty.
- `missing` when source count == 0.

- [ ] **Step 3: Implement Markdown composer**

Use small deterministic helpers:

```rust
fn compose_project_brief(snapshot: &Value) -> String;
fn compose_brand_understanding(snapshot: &Value) -> String;
fn compose_product_understanding(snapshot: &Value) -> String;
fn compose_research_findings(snapshot: &Value) -> String;
fn compose_creative_direction(snapshot: &Value) -> String;
fn compose_storyboard(snapshot: &Value) -> String;
fn compose_prompt_packages(snapshot: &Value) -> String;
fn compose_assets(snapshot: &Value) -> String;
fn compose_version_notes(snapshot: &Value) -> String;
```

Use structured JSON access; do not use ad hoc substring parsing for nested fields.

- [ ] **Step 4: Agent run/events**

Create an Agent run:

- `runtime_kind`: `hermes_core`
- `runtime_mode`: `local_delivery_report_bridge`
- `runtime_version`: `0.18.0`
- roles:
  - `planner`
  - `reviewer`
  - `memory_curator`
- events:
  - `delivery_context_read`
  - `delivery_sections_resolved`
  - `delivery_report_drafted`
  - `delivery_package_previewed`
  - `delivery_report_saved`

- [ ] **Step 5: Missing-context test**

Add test:

```rust
#[test]
fn generated_report_marks_missing_sections_without_failing() {
    let app = TestApp::new();
    let state = app_state(&app);
    let project_id = seed_minimal_project(&state);

    let result = generate_delivery_report(
        &state,
        DeliveryReportGenerationInput {
            project_id,
            user_direction: String::new(),
        },
    )
    .expect("report with warnings");

    assert!(result.report.markdown.contains("No saved research report yet."));
    assert!(result.sections.iter().any(|section| section.status == "missing"));
}
```

### Task 3: Commands And Export Integration

**Files:**

- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/project_package.rs`
- Test: `src-tauri/tests/commands.rs`
- Test: `src-tauri/tests/project_export.rs`

- [ ] **Step 1: Write failing command tests**

Add command input round-trip tests:

```rust
let update: DeliveryReportUpdateInput = serde_json::from_value(json!({
    "id": "report-1",
    "title": "Report",
    "markdown": "# Report",
    "sections_json": {"format_version": "joi.delivery_report_sections.v1"},
    "is_final_candidate": true
}))?;

let preview: DeliveryPackagePreviewInput = serde_json::from_value(json!({
    "project_id": "project-1",
    "delivery_report_id": "report-1"
}))?;
```

Add state helper test:

```rust
#[test]
fn state_helpers_generate_update_list_preview_and_export_delivery_report() {
    let app = TestApp::new();
    let state = app.state();
    let project_id = seed_full_delivery_project(&state);

    let generated = generate_delivery_report(&state, DeliveryReportGenerationInput {
        project_id: project_id.clone(),
        user_direction: "Prepare final handoff.".into(),
    }).expect("generate report");

    let updated = update_delivery_report(&state, DeliveryReportUpdateInput {
        id: generated.report.id.clone(),
        title: "Edited Delivery Report".into(),
        markdown: "# Edited Delivery Report".into(),
        sections_json: generated.report.sections_json.clone(),
        is_final_candidate: true,
    }).expect("update");

    assert_eq!(updated.title, "Edited Delivery Report");
    assert_eq!(list_delivery_reports(&state, project_id.clone()).unwrap().len(), 1);

    let preview = preview_delivery_package(&state, DeliveryPackagePreviewInput {
        project_id: project_id.clone(),
        delivery_report_id: Some(updated.id.clone()),
    }).expect("preview");
    assert!(preview.delivery_report_file_name.ends_with("-delivery-report.md"));
}
```

Expected RED:

- command helpers do not exist.

- [ ] **Step 2: Add Tauri commands**

Command names:

```rust
joi_generate_delivery_report
joi_list_delivery_reports
joi_update_delivery_report
joi_preview_delivery_package
```

Register in `tauri::generate_handler!`.

- [ ] **Step 3: Export project with optional report**

Modify `ProjectPackageService::export_project`:

- If `delivery_report_id` is `Some`, load the report and verify it belongs to project.
- Write `{slug}-delivery-report.md`.
- Add `delivery_report` metadata to package JSON.
- Return `delivery_report_path: Some(path)`.
- If `delivery_report_id` is `None`, preserve existing behavior and return `None`.

- [ ] **Step 4: Export tests**

Add to `src-tauri/tests/project_export.rs`:

```rust
#[test]
fn exports_delivery_report_markdown_with_project_package() {
    let app = TestApp::new();
    let db = migrated_database(&app);
    let repo = Repository::new(db.connection());
    let project_id = create_project(&repo, "Launch Film");
    let report = repo.create_delivery_report(DeliveryReportCreate {
        project_id: project_id.clone(),
        title: "Launch Film Delivery Report".into(),
        markdown: "# Launch Film Delivery Report".into(),
        sections_json: json!({"format_version": "joi.delivery_report_sections.v1", "sections": []}),
        is_final_candidate: true,
    }).expect("report");

    let export_dir = app.temp_dir.path().join("exports");
    let result = ProjectPackageService::new(db.connection(), app.temp_dir.path().join("assets"))
        .export_project(ProjectExportInput {
            project_id,
            export_dir: export_dir.clone(),
            delivery_report_id: Some(report.id.clone()),
        })
        .expect("export");

    let report_path = result.delivery_report_path.expect("report path");
    assert!(report_path.ends_with("launch-film-delivery-report.md"));
    assert_eq!(std::fs::read_to_string(report_path).unwrap(), "# Launch Film Delivery Report");
}
```

Add rejection test for report/project mismatch.

### Task 4: Frontend Delivery Workspace

**Files:**

- Modify: `src/types/joi.ts`
- Modify: `src/api/joiApi.ts`
- Modify: `src/components/BrandProjectRail.tsx`
- Modify: `src/components/ProjectWorkspace.tsx`
- Create: `src/components/DeliveryWorkspace.tsx`
- Modify: `src/App.tsx`
- Modify: `src/App.test.tsx`
- Modify: `src/styles.css`

- [ ] **Step 1: Add frontend types and API wrappers**

Add to `src/types/joi.ts`:

```ts
export type DeliveryReport = {
  id: string;
  project_id: string;
  title: string;
  markdown: string;
  sections_json: unknown;
  is_final_candidate: boolean;
  created_at: string;
  updated_at: string;
};

export type DeliveryReportGenerationInput = {
  project_id: string;
  user_direction: string;
};

export type DeliveryReportSectionStatus = {
  id: string;
  title: string;
  status: string;
  source_count: number;
  warning: string;
};

export type DeliveryPackagePreview = {
  project_json_file_name: string;
  assets_folder_name: string;
  delivery_report_file_name: string;
  included_assets_count: number;
  included_prompt_packages_count: number;
  included_storyboards_count: number;
  warnings: string[];
};

export type DeliveryReportGenerationResult = {
  report: DeliveryReport;
  sections: DeliveryReportSectionStatus[];
  package_preview: DeliveryPackagePreview;
  agent_run: AgentRun;
  agent_events: AgentRunEvent[];
};

export type DeliveryReportUpdateInput = {
  id: string;
  title: string;
  markdown: string;
  sections_json: unknown;
  is_final_candidate: boolean;
};

export type DeliveryPackagePreviewInput = {
  project_id: string;
  delivery_report_id: string | null;
};

export type ProjectExportInput = {
  project_id: string;
  export_dir: string;
  delivery_report_id: string | null;
};

export type ProjectExportResult = {
  project_json_path: string;
  assets_dir: string;
  delivery_report_path: string | null;
};
```

Add wrappers:

```ts
generateDeliveryReport(input)
listDeliveryReports(projectId)
updateDeliveryReport(input)
previewDeliveryPackage(input)
exportProject(input)
```

- [ ] **Step 2: Add failing UI test**

In `src/App.test.tsx`, add command mocks:

```ts
case "joi_generate_delivery_report":
  return Promise.resolve(mockDeliveryReportGenerationResult);
case "joi_list_delivery_reports":
  return Promise.resolve([]);
case "joi_update_delivery_report":
  return Promise.resolve({...mockDeliveryReportGenerationResult.report, markdown: args?.input?.markdown});
case "joi_preview_delivery_package":
  return Promise.resolve(mockDeliveryReportGenerationResult.package_preview);
case "joi_export_project":
  return Promise.resolve({
    project_json_path: "D:/exports/spring-drop-film.joi-project.json",
    assets_dir: "D:/exports/spring-drop-film-assets",
    delivery_report_path: "D:/exports/spring-drop-film-delivery-report.md",
  });
```

Add test:

```ts
test("generates edits previews and exports delivery reports", async () => {
  render(<App />);

  await screen.findByRole("heading", { name: "Spring Drop Film" });
  fireEvent.click(screen.getByRole("button", { name: "Delivery" }));
  fireEvent.change(screen.getByLabelText("Delivery direction"), {
    target: { value: "Make the handoff concise." },
  });
  fireEvent.click(screen.getByRole("button", { name: /generate delivery report/i }));

  await waitFor(() => {
    expect(invokeMock).toHaveBeenCalledWith("joi_generate_delivery_report", {
      input: {
        project_id: "project-1",
        user_direction: "Make the handoff concise.",
      },
    });
  });

  expect(await screen.findByText("Prompt Packages")).toBeInTheDocument();
  fireEvent.change(screen.getByLabelText("Report markdown"), {
    target: { value: "# Edited delivery report" },
  });
  fireEvent.click(screen.getByRole("button", { name: /save report/i }));

  await waitFor(() => {
    expect(invokeMock).toHaveBeenCalledWith("joi_update_delivery_report", expect.objectContaining({
      input: expect.objectContaining({ markdown: "# Edited delivery report" }),
    }));
  });

  expect(await screen.findByText("spring-drop-film-delivery-report.md")).toBeInTheDocument();
  fireEvent.change(screen.getByLabelText("Export directory"), {
    target: { value: "D:/exports" },
  });
  fireEvent.click(screen.getByRole("button", { name: /export package/i }));

  await waitFor(() => {
    expect(invokeMock).toHaveBeenCalledWith("joi_export_project", {
      input: {
        project_id: "project-1",
        export_dir: "D:/exports",
        delivery_report_id: "delivery-report-1",
      },
    });
  });
});
```

Run:

```powershell
npm test -- src/App.test.tsx
```

Expected RED:

- Delivery tab/workspace and wrappers do not exist.

- [ ] **Step 3: Build `DeliveryWorkspace.tsx`**

Export draft type:

```ts
export type DeliveryDraft = {
  user_direction: string;
  title: string;
  markdown: string;
  is_final_candidate: boolean;
  export_dir: string;
};
```

Props:

- delivery reports
- active report/result
- package preview
- selected project
- generating/saving/exporting flags
- draft change handlers
- generate/save/export handlers

Render:

- `Delivery direction`
- `Generate Delivery Report`
- report list
- section status indicators
- `Report title`
- `Report markdown`
- `Final candidate`
- `Save Report`
- package preview file list
- `Export directory`
- `Export Package`

Use labels for all controls. Keep report editor in a full-width panel.

- [ ] **Step 4: Wire App state**

Add state:

```ts
const emptyDeliveryDraft: DeliveryDraft = {
  user_direction: "",
  title: "",
  markdown: "",
  is_final_candidate: false,
  export_dir: "",
};
const [deliveryDraft, setDeliveryDraft] = useState<DeliveryDraft>(emptyDeliveryDraft);
const [deliveryReports, setDeliveryReports] = useState<DeliveryReport[]>([]);
const [deliveryResult, setDeliveryResult] = useState<DeliveryReportGenerationResult | null>(null);
const [deliveryPreview, setDeliveryPreview] = useState<DeliveryPackagePreview | null>(null);
const [generatingDeliveryReport, setGeneratingDeliveryReport] = useState(false);
const [savingDeliveryReport, setSavingDeliveryReport] = useState(false);
const [exportingProject, setExportingProject] = useState(false);
const [exportResult, setExportResult] = useState<ProjectExportResult | null>(null);
```

Project refresh:

- `listDeliveryReports(projectId)`
- `previewDeliveryPackage({ project_id: projectId, delivery_report_id: latestReportIdOrNull })`

Handlers:

- `submitDeliveryReportGeneration`
- `handleDeliveryReportSelect`
- `handleUpdateDeliveryReport`
- `handleExportDeliveryPackage`

When generating:

- set report into editor draft
- set delivery result
- refresh project state
- prepend agent run/events

When saving:

- update report list
- refresh package preview

When exporting:

- call `exportProject`
- show returned paths in activity log and UI

- [ ] **Step 5: Add Delivery tab**

Modify `BrandProjectRail.tsx`:

```ts
const workspaceTabs = [
  "Overview",
  "Brief",
  "Research",
  "Storyboard",
  "Prompts",
  "Delivery",
  "Assets",
  "Memory",
  "Versions",
];
```

Modify `ProjectWorkspace.tsx` fallback list to include `Delivery`.

- [ ] **Step 6: CSS**

Add:

```css
.delivery-layout {
  display: grid;
  gap: 14px;
}

.delivery-toolbar,
.delivery-editor-grid,
.delivery-preview-grid {
  display: grid;
  gap: 12px;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.delivery-markdown-editor {
  min-height: 420px;
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
}

.section-status-grid {
  display: grid;
  gap: 8px;
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
}
```

Mobile:

```css
@media (max-width: 1100px) {
  .delivery-toolbar,
  .delivery-editor-grid,
  .delivery-preview-grid {
    grid-template-columns: 1fr;
  }
}
```

### Task 5: Smoke, Review, Merge, Push

**Files:**

- Create: `docs/superpowers/reports/joi-0.18-reports-delivery-package-smoke-test.md`

- [ ] **Step 1: Full verification**

```powershell
npm test
npm run build
cd src-tauri
cargo test
cargo test --test delivery_report -- --nocapture
cargo test --test project_export -- --nocapture
cargo test --test commands -- --nocapture
```

- [ ] **Step 2: Browser smoke**

Start:

```powershell
npm run dev -- --host 127.0.0.1 --port 1420
```

Verify:

- Delivery tab renders.
- Generate Delivery Report button is visible.
- Report markdown editor is visible.
- Package preview shows JSON, assets folder, and Markdown report file names.
- Export directory field is visible.
- Desktop `1440x900` has no horizontal overflow.
- Mobile `390x844` has no horizontal overflow.

Normal browser limitation:

- A normal browser cannot call native Tauri commands. Command integration is covered by Rust and React tests. Browser smoke may verify shell/layout only unless a Tauri invoke mock harness is available.

- [ ] **Step 3: Smoke report**

Create `docs/superpowers/reports/joi-0.18-reports-delivery-package-smoke-test.md` with:

- verification commands run
- browser viewports checked
- package files covered
- acceptance checklist
- known limitations

- [ ] **Step 4: Commit smoke report**

```powershell
git add docs/superpowers/reports/joi-0.18-reports-delivery-package-smoke-test.md
git commit -m "test: add Joi 0.18 delivery package smoke report"
```

- [ ] **Step 5: Merge and push**

From the main workspace:

```powershell
git status --short --branch
git merge --ff-only codex/joi-0.18-delivery-package
npm test
npm run build
cd src-tauri
cargo test
git push origin main
```

- [ ] **Step 6: Clean up worktree**

```powershell
git worktree remove --force "D:\Software Project\Joi-agent\.worktrees\joi-0.18-delivery-package"
git worktree prune
git branch -d codex/joi-0.18-delivery-package
```

## Acceptance Criteria

0.18 is complete only when:

- Delivery reports are persisted in `delivery_reports`.
- Delivery reports are included in project snapshots.
- Joi can generate a Markdown report from the current project context.
- Report includes project brief, brand, product, research, creative direction, storyboard, prompt packages, assets, and versions.
- Missing source sections are shown as warnings, not silently invented.
- User can edit and save report title, markdown, and final candidate status.
- Delivery package preview shows JSON file, assets folder, Markdown report file, counts, and warnings.
- Project export can optionally include a selected delivery report Markdown file.
- Existing export behavior still works when no report is selected.
- Delivery report generation creates Agent run/events.
- Frontend Delivery tab covers generate, edit, preview, and export flow.
- Tests cover schema migration, repository, generator, commands, export integration, and frontend flow.
- Browser smoke report is written.
- Changes are merged to `main` and pushed to GitHub.

## Risks And Mitigations

### Risk: Report Generator Invents Missing Context

Mitigation:

- Compose from snapshot JSON and repository data only.
- For missing sections, write explicit warnings.
- Add missing-context test.

### Risk: Export Breaks Existing `.joi-project.json` Users

Mitigation:

- Keep `delivery_report_id` optional.
- Keep package `format_version` at `1` for additive metadata only.
- Existing export tests must still pass with `delivery_report_path = None`.

### Risk: Report Markdown Becomes Too Large For UI Editing

Mitigation:

- Use a single full-width textarea with stable dimensions.
- Keep section indicators separate from Markdown editor.
- Do not attempt a rich text editor in 0.18.

### Risk: Delivery Package Preview Drifts From Export

Mitigation:

- Generate preview from the same slug and report selection helpers used by exporter.
- Add test asserting preview file names match exported file names.

## Handoff To 0.19

0.19 quality review should consume:

- `DeliveryReport.markdown`
- `DeliveryReport.sections_json.sections`
- `DeliveryReport.is_final_candidate`
- package preview warnings
- prompt package missing fields
- storyboard duration and shot rows

0.19 should add structured review checklists that can point to specific sections, shots, and prompt packages before final delivery.

## Self-Review

- Spec coverage: This plan covers the 0.18 roadmap scope: report generation, Markdown export, `.joi-project.json` export integration, delivery package preview, frontend report editing, tests, smoke, merge, and push.
- Placeholder scan: File paths, command names, type names, table names, test names, and expected outputs are concrete.
- Type consistency: `DeliveryReport`, `DeliveryReportGenerationInput`, `DeliveryReportGenerationResult`, `DeliveryPackagePreview`, `DeliveryReportUpdateInput`, and `ProjectExportInput` are aligned across backend, command, frontend, and tests.
