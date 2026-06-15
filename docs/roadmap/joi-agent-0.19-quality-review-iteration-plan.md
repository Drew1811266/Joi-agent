# Joi Agent 0.19 Quality Review And Iteration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a persisted Review workspace where Joi can inspect storyboard quality, prompt completeness, brand/product consistency, and apply selected revision suggestions back into shots or prompt packages.

**Architecture:** 0.19 adds a `quality_reviews` aggregate, a deterministic local review service, and command/UI bindings that sit between the 0.17 prompt package workflow and the 0.18 delivery report workflow. The review engine reads saved project context from the Joi repository, writes structured checklist and suggestion JSON, logs Agent runs through the existing Hermes Core bridge model, and applies only explicit user-accepted edits to existing shot or prompt package records.

**Tech Stack:** Tauri 2 commands, Rust, rusqlite, serde/serde_json, chrono, React 19, TypeScript, Vitest, Testing Library, Joi local repository, Joi 0.13 Agent run model, Joi 0.16 storyboard records, Joi 0.17 prompt package records, Joi 0.18 delivery package flow.

---

## Product Outcome

After 0.19, a user can:

- Open a `Review` workspace tab between `Prompts` and `Delivery`.
- Generate a structured quality review for the selected project.
- See a review score, checklist, evidence, severity, and grouped suggestions.
- Detect these concrete issues:
  - storyboard total duration does not match the storyboard/project target
  - repeated or near-repeated shots reduce story progression
  - shots do not visibly surface the product category or selling points
  - prompts are missing required adapter fields
  - prompts do not carry enough brand/product context
- Accept a supported suggestion and have Joi update the target shot or prompt package.
- Refresh project state after applying a suggestion.
- Save review results into project snapshots.
- See review generation and suggestion application in the Agent run log.

0.19 does not publish content, call external image/video generation models, replace human approval, or attempt subjective aesthetic scoring beyond explicit checklist rules.

## Scope

### In Scope

- `quality_reviews` persistence:
  - project binding
  - summary
  - score
  - checklist JSON
  - suggestion JSON
  - timestamps
- Review generation:
  - storyboard duration checks
  - shot repetition checks
  - garment and selling-point visibility checks
  - brand consistency checks
  - prompt completeness checks using existing prompt adapter metadata
  - structured checklist output
  - supported revision suggestions
  - Agent run/events
- Suggestion application:
  - apply shot `description` suggestions through `Repository::update_shot`
  - apply prompt `prompt_text` suggestions through `Repository::update_prompt_package`
  - mark applied suggestion status inside `quality_reviews.suggestions_json`
  - reject locked targets with validation errors
  - reject unsupported target fields with validation errors
- Snapshot integration:
  - `ProjectSnapshotService::build_snapshot` includes `quality_reviews`
- Frontend:
  - Review tab in the workflow rail
  - Review workspace
  - generate review control
  - score/checklist presentation
  - suggestion list with apply buttons
  - activity log integration
- Tests and smoke report.

### Out Of Scope

- No automatic publishing or upload to ad platforms.
- No direct calls to Jimeng, Grok, Banana, GPT Image, or other generation APIs.
- No subjective style ranking beyond deterministic review rules.
- No multi-user approval workflow.
- No destructive auto-application of all suggestions.
- No rewriting locked shots or locked prompt packages.

## Existing Code Context

Use these current pieces:

- `src-tauri/src/db.rs`
  - single `SCHEMA` string creates tables and indexes.
  - add `quality_reviews` near `delivery_reports`.
- `src-tauri/src/models.rs`
  - add `QualityReview` next to `DeliveryReport`.
- `src-tauri/src/repositories.rs`
  - existing CRUD patterns for `DeliveryReport`, `PromptPackage`, `Shot`, `AgentRun`, and events.
  - add `QualityReviewCreate`, `QualityReviewUpdate`, `create_quality_review`, `get_quality_review`, `list_quality_reviews`, `update_quality_review_suggestions`.
- `src-tauri/src/storyboard.rs`
  - `StoryboardShotView::from_shot` gives derived `visual_description`, `garment_focus`, and `transition`.
  - `regenerate_shot` shows the established Agent run/event pattern.
- `src-tauri/src/prompt_adapter.rs`
  - `PromptPackageView`, `prompt_package_view`, `PromptCompletenessCheck`, and `missing_fields` already expose adapter completeness data.
- `src-tauri/src/snapshots.rs`
  - `ProjectSnapshotService::build_snapshot` includes all current aggregates.
  - add `quality_reviews`.
- `src-tauri/src/commands.rs`
  - command functions expose services and repository operations.
  - add review command structs and public command helpers.
- `src-tauri/src/lib.rs`
  - register new command handlers.
- `src/components/BrandProjectRail.tsx`
  - workflow tab list currently includes `Delivery`; insert `Review` before it.
- `src/components/ProjectWorkspace.tsx`
  - add props and render `ReviewWorkspace`.
- `src/App.tsx`
  - refresh project state currently loads storyboards, prompts, delivery reports, versions, and agent runs.
  - add quality reviews and review action handlers.
- `src/api/joiApi.ts`
  - add invoke wrappers.
- `src/types/joi.ts`
  - add review types mirroring Rust serde output.

## Data Contract

### Quality Review Table

Create `quality_reviews`:

```sql
CREATE TABLE IF NOT EXISTS quality_reviews (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  summary TEXT NOT NULL,
  score INTEGER NOT NULL DEFAULT 0,
  checklist_json TEXT NOT NULL DEFAULT '[]',
  suggestions_json TEXT NOT NULL DEFAULT '[]',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
  CHECK (score BETWEEN 0 AND 100)
);

CREATE INDEX IF NOT EXISTS idx_quality_reviews_project_id ON quality_reviews(project_id);
```

### Rust Model

Add to `src-tauri/src/models.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReview {
    pub id: String,
    pub project_id: String,
    pub summary: String,
    pub score: i64,
    pub checklist_json: Value,
    pub suggestions_json: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Review Checklist JSON

Persist `checklist_json` as:

```json
[
  {
    "id": "duration-storyboard-1",
    "category": "storyboard_duration",
    "title": "Storyboard duration matches target",
    "status": "failed",
    "severity": "high",
    "source_type": "storyboard",
    "source_id": "storyboard-1",
    "message": "Storyboard totals 18s while project target is 15s.",
    "evidence": ["Story target: 15s", "Shot total: 18s"],
    "suggestion_ids": []
  }
]
```

Allowed `status` values:

- `passed`
- `warning`
- `failed`

Allowed `severity` values:

- `info`
- `medium`
- `high`

Allowed `category` values:

- `storyboard_duration`
- `shot_repetition`
- `garment_visibility`
- `brand_consistency`
- `prompt_completeness`
- `prompt_context`

### Review Suggestions JSON

Persist `suggestions_json` as:

```json
[
  {
    "id": "suggest-shot-shot-1-description",
    "target_type": "shot",
    "target_id": "shot-1",
    "field": "description",
    "current_value": "Model walks forward.",
    "suggested_value": "Model walks forward while the trench silhouette and water-resistant cotton texture stay visible.",
    "rationale": "The shot should make the garment benefit visible instead of only describing movement.",
    "status": "pending",
    "check_ids": ["garment-shot-1"]
  }
]
```

Allowed `status` values:

- `pending`
- `applied`
- `rejected`

Supported apply targets in 0.19:

- `target_type = "shot"`, `field = "description"`
- `target_type = "prompt_package"`, `field = "prompt_text"`

Unsupported apply targets must return `JoiError::Validation` with a concrete message:

```text
review suggestion target is not supported: <target_type>.<field>
```

## Review Rule Contract

The first 0.19 implementation is deterministic and local. It should not call an external LLM. This keeps tests stable and matches the current local bridge style.

### Rule 1: Duration Consistency

For every storyboard:

- Sum `shots.duration_seconds`.
- Compare to `storyboard.duration_seconds`.
- Compare to `project.duration_seconds`.
- If either differs, create a failed `storyboard_duration` check.
- Duration checks do not create auto-apply suggestions in 0.19 because changing duration safely requires a timeline balancing UI.

### Rule 2: Shot Repetition

For each storyboard:

- Normalize `description`, `model_action`, `camera_movement`, and `scene` by lowercasing, trimming, removing punctuation, and collapsing whitespace.
- If two shots share the same normalized `description` or share at least three of the four normalized fields, create a `shot_repetition` warning for the later shot.
- Create a shot description suggestion that adds a product detail or scene progression.

### Rule 3: Garment Visibility

Build product terms from:

- latest `ProductUnderstanding.product_name`
- latest `ProductUnderstanding.category`
- latest `ProductUnderstanding.selling_points_json`
- `Brand.description`

For each shot:

- Combine `description`, `model_action`, `scene`, `lighting`, and `metadata_json.garment_focus`.
- If no product/category/selling point term appears in that text, create a `garment_visibility` failed check.
- Create a shot description suggestion that appends a concise garment visibility clause.

### Rule 4: Brand Consistency

Build brand terms from:

- `Brand.name`
- `Brand.description`
- latest `CreativeDirection.tone`
- latest `CreativeDirection.visual_style`

For storyboards and prompt packages:

- If the project has brand or creative direction text and no brand/style term appears in a shot or prompt, create a `brand_consistency` warning.
- For prompt packages, create a `prompt_text` suggestion that appends a brand/style line.
- For shots, create a `description` suggestion only if the shot does not already have a garment visibility suggestion.

### Rule 5: Prompt Completeness

For each prompt package:

- Convert package to `PromptPackageView` with `prompt_package_view`.
- For each `missing_fields` value, create one `prompt_completeness` failed check.
- Create one `prompt_text` suggestion that appends the missing fields in a compact provider-neutral form:

```text
Include: <field-1>, <field-2>.
```

### Rule 6: Score

Compute score as:

```rust
let penalty = checks.iter().map(|check| match (check.status.as_str(), check.severity.as_str()) {
    ("failed", "high") => 18,
    ("failed", _) => 12,
    ("warning", "high") => 10,
    ("warning", _) => 6,
    _ => 0,
}).sum::<i64>();
let score = (100 - penalty).clamp(0, 100);
```

The persisted `summary` should be deterministic:

```text
Quality review scored <score>/100 with <failed_count> failed check(s), <warning_count> warning(s), and <pending_suggestions> pending suggestion(s).
```

## Implementation Tasks

### Task 1: Quality Review Data Model And Repository

**Files:**

- Modify: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/repositories.rs`
- Test: `src-tauri/tests/db_migration.rs`
- Test: `src-tauri/tests/structured_content_repository.rs`

- [ ] **Step 1: Write failing migration test**

Add this test to `src-tauri/tests/db_migration.rs`:

```rust
#[test]
fn migration_creates_quality_reviews_table() {
    let db = TestDb::new();
    let tables = db.database.table_names().expect("table names");
    assert!(tables.contains(&"quality_reviews".to_string()));

    let mut statement = db
        .database
        .connection()
        .prepare("PRAGMA index_list(quality_reviews)")
        .expect("index list");
    let indexes = statement
        .query_map([], |row| row.get::<_, String>(1))
        .expect("index rows")
        .collect::<Result<Vec<_>, _>>()
        .expect("indexes");
    assert!(indexes.contains(&"idx_quality_reviews_project_id".to_string()));
}
```

- [ ] **Step 2: Run migration test and verify RED**

Run:

```powershell
cargo test migration_creates_quality_reviews_table
```

Expected:

```text
FAILED migration_creates_quality_reviews_table
```

The failure must be because `quality_reviews` or `idx_quality_reviews_project_id` does not exist.

- [ ] **Step 3: Write failing repository test**

Add this test to `src-tauri/tests/structured_content_repository.rs`:

```rust
#[test]
fn repository_creates_lists_and_updates_quality_review_suggestions() {
    let db = TestDb::new();
    let repo = Repository::new(db.database.connection());
    let (_brand, project) = create_brand_and_project(&repo);

    let review = repo
        .create_quality_review(QualityReviewCreate {
            project_id: project.id.clone(),
            summary: "Quality review scored 82/100 with 1 failed check(s), 2 warning(s), and 1 pending suggestion(s).".to_string(),
            score: 82,
            checklist_json: json!([
                {
                    "id": "duration-storyboard-1",
                    "category": "storyboard_duration",
                    "title": "Storyboard duration matches target",
                    "status": "failed",
                    "severity": "high",
                    "source_type": "storyboard",
                    "source_id": "storyboard-1",
                    "message": "Storyboard totals 18s while project target is 15s.",
                    "evidence": ["Story target: 15s", "Shot total: 18s"],
                    "suggestion_ids": []
                }
            ]),
            suggestions_json: json!([
                {
                    "id": "suggest-shot-shot-1-description",
                    "target_type": "shot",
                    "target_id": "shot-1",
                    "field": "description",
                    "current_value": "Model walks forward.",
                    "suggested_value": "Model walks forward while the trench texture stays visible.",
                    "rationale": "Expose the product benefit.",
                    "status": "pending",
                    "check_ids": ["garment-shot-1"]
                }
            ]),
        })
        .expect("quality review");

    assert_eq!(review.project_id, project.id);
    assert_eq!(review.score, 82);
    assert_eq!(review.checklist_json[0]["category"], "storyboard_duration");

    let listed = repo
        .list_quality_reviews(&project.id)
        .expect("quality reviews");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, review.id);

    let updated = repo
        .update_quality_review_suggestions(
            &review.id,
            json!([
                {
                    "id": "suggest-shot-shot-1-description",
                    "target_type": "shot",
                    "target_id": "shot-1",
                    "field": "description",
                    "current_value": "Model walks forward.",
                    "suggested_value": "Model walks forward while the trench texture stays visible.",
                    "rationale": "Expose the product benefit.",
                    "status": "applied",
                    "check_ids": ["garment-shot-1"]
                }
            ]),
        )
        .expect("updated suggestions");

    assert_eq!(updated.suggestions_json[0]["status"], "applied");
    assert!(updated.updated_at >= review.updated_at);
}
```

Add imports at the top of the file:

```rust
use joi_agent_lib::repositories::QualityReviewCreate;
use serde_json::json;
```

If `json` is already imported, keep only one import.

- [ ] **Step 4: Run repository test and verify RED**

Run:

```powershell
cargo test repository_creates_lists_and_updates_quality_review_suggestions
```

Expected:

```text
FAILED repository_creates_lists_and_updates_quality_review_suggestions
```

The failure must be because `QualityReviewCreate`, `create_quality_review`, `list_quality_reviews`, or `update_quality_review_suggestions` does not exist.

- [ ] **Step 5: Add table and model**

In `src-tauri/src/db.rs`, insert the `quality_reviews` table after `delivery_reports` and insert `idx_quality_reviews_project_id` after `idx_delivery_reports_project_id`.

In `src-tauri/src/models.rs`, insert the `QualityReview` struct after `DeliveryReport`.

- [ ] **Step 6: Add repository types**

In `src-tauri/src/repositories.rs`, import `QualityReview` from `models`, then add:

```rust
#[derive(Debug, Clone)]
pub struct QualityReviewCreate {
    pub project_id: String,
    pub summary: String,
    pub score: i64,
    pub checklist_json: Value,
    pub suggestions_json: Value,
}
```

- [ ] **Step 7: Add repository methods**

Inside `impl<'a> Repository<'a>`, add:

```rust
pub fn create_quality_review(&self, input: QualityReviewCreate) -> JoiResult<QualityReview> {
    self.get_project(&input.project_id)?;
    validate_required_text("Quality review summary", &input.summary)?;
    if !(0..=100).contains(&input.score) {
        return Err(JoiError::Validation(
            "Quality review score must be between 0 and 100".to_string(),
        ));
    }

    let now = Utc::now();
    let review = QualityReview {
        id: new_id(),
        project_id: input.project_id,
        summary: input.summary.trim().to_string(),
        score: input.score,
        checklist_json: input.checklist_json,
        suggestions_json: input.suggestions_json,
        created_at: now,
        updated_at: now,
    };

    self.connection.execute(
        "INSERT INTO quality_reviews (
            id, project_id, summary, score, checklist_json, suggestions_json, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            review.id,
            review.project_id,
            review.summary,
            review.score,
            review.checklist_json.to_string(),
            review.suggestions_json.to_string(),
            review.created_at.to_rfc3339(),
            review.updated_at.to_rfc3339()
        ],
    )?;

    Ok(review)
}

pub fn get_quality_review(&self, id: &str) -> JoiResult<QualityReview> {
    self.connection
        .query_row(
            "SELECT id, project_id, summary, score, checklist_json, suggestions_json, created_at, updated_at
             FROM quality_reviews WHERE id = ?1",
            params![id],
            map_quality_review,
        )
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => JoiError::NotFound(format!("quality review {}", id)),
            other => other.into(),
        })
}

pub fn list_quality_reviews(&self, project_id: &str) -> JoiResult<Vec<QualityReview>> {
    let mut statement = self.connection.prepare(
        "SELECT id, project_id, summary, score, checklist_json, suggestions_json, created_at, updated_at
         FROM quality_reviews WHERE project_id = ?1 ORDER BY created_at ASC",
    )?;
    let rows = statement.query_map(params![project_id], map_quality_review)?;
    collect_rows(rows)
}

pub fn update_quality_review_suggestions(
    &self,
    id: &str,
    suggestions_json: Value,
) -> JoiResult<QualityReview> {
    let now = Utc::now();
    let affected = self.connection.execute(
        "UPDATE quality_reviews
         SET suggestions_json = ?1,
             updated_at = ?2
         WHERE id = ?3",
        params![suggestions_json.to_string(), now.to_rfc3339(), id],
    )?;
    if affected == 0 {
        return Err(JoiError::NotFound(format!("quality review {}", id)));
    }
    self.get_quality_review(id)
}
```

- [ ] **Step 8: Add row mapper**

Near the other mapper functions in `src-tauri/src/repositories.rs`, add:

```rust
fn map_quality_review(row: &rusqlite::Row<'_>) -> rusqlite::Result<QualityReview> {
    Ok(QualityReview {
        id: row.get(0)?,
        project_id: row.get(1)?,
        summary: row.get(2)?,
        score: row.get(3)?,
        checklist_json: json_from_column(row, 4)?,
        suggestions_json: json_from_column(row, 5)?,
        created_at: datetime_from_column(row, 6)?,
        updated_at: datetime_from_column(row, 7)?,
    })
}
```

- [ ] **Step 9: Run data tests and verify PASS**

Run:

```powershell
cargo test migration_creates_quality_reviews_table
cargo test repository_creates_lists_and_updates_quality_review_suggestions
```

Expected:

```text
test result: ok
```

- [ ] **Step 10: Commit Task 1**

Run:

```powershell
git add src-tauri/src/db.rs src-tauri/src/models.rs src-tauri/src/repositories.rs src-tauri/tests/db_migration.rs src-tauri/tests/structured_content_repository.rs
git commit -m "feat: add quality review repository"
```

### Task 2: Quality Review Generator Service

**Files:**

- Create: `src-tauri/src/quality_review.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/quality_review.rs`

- [ ] **Step 1: Write failing generator test**

Create `src-tauri/tests/quality_review.rs`:

```rust
mod helpers;

use joi_agent_lib::quality_review::{
    generate_quality_review, QualityReviewGenerationInput, QualityReviewSuggestionStatus,
};
use joi_agent_lib::repositories::{
    CreativeDirectionCreate, ProductUnderstandingCreate, PromptPackageCreate, Repository,
    ShotPlanCreate, StoryboardCreate,
};
use serde_json::json;

use helpers::TestDb;

#[test]
fn generate_quality_review_detects_storyboard_prompt_and_product_issues() {
    let db = TestDb::new();
    let repo = Repository::new(db.database.connection());
    let (brand, project) = helpers::create_brand_and_project(&repo);

    repo.create_product_understanding(ProductUnderstandingCreate {
        project_id: project.id.clone(),
        product_name: "Lightweight Trench".to_string(),
        category: "outerwear".to_string(),
        audience: "urban commuters".to_string(),
        selling_points: vec!["water-resistant cotton".to_string()],
        constraints: vec!["avoid winter styling".to_string()],
        notes: "Focus on material proof.".to_string(),
    })
    .expect("understanding");
    repo.create_creative_direction(CreativeDirectionCreate {
        project_id: project.id.clone(),
        title: "Clean Studio".to_string(),
        concept: "studio walk with fabric inserts".to_string(),
        tone: "premium".to_string(),
        visual_style: "clean warm studio".to_string(),
        scene_direction: "warm studio".to_string(),
        rationale: "Matches brand setup.".to_string(),
    })
    .expect("direction");

    let storyboard = repo
        .create_storyboard(StoryboardCreate {
            project_id: project.id.clone(),
            title: "Repeated storyboard".to_string(),
            duration_seconds: 15,
        })
        .expect("storyboard");

    for shot_number in 1..=2 {
        repo.create_shot_plan(ShotPlanCreate {
            storyboard_id: storyboard.id.clone(),
            shot_number,
            duration_seconds: 9,
            visual_description: "Model walks forward in a studio.".to_string(),
            model_action: "Model walks forward.".to_string(),
            garment_focus: "movement".to_string(),
            camera_movement: "slow push-in".to_string(),
            scene: "studio".to_string(),
            lighting: "soft light".to_string(),
            transition: "cut".to_string(),
            subtitle_or_text: "New season energy".to_string(),
            rationale: "Creates opening motion.".to_string(),
            source_memory_ids: vec![],
            source_research_report_ids: vec![],
            generation_context: json!({}),
        })
        .expect("shot");
    }

    repo.create_prompt_package(PromptPackageCreate {
        project_id: project.id.clone(),
        shot_id: None,
        platform: "gpt_image_2".to_string(),
        modality: "image".to_string(),
        prompt_text: "Create a model photo.".to_string(),
        negative_prompt: "avoid distorted garment".to_string(),
        parameters_json: json!({
            "format_version": "joi.prompt_package_parameters.v1",
            "adapter_profile_id": "gpt_image_2",
            "adapter_display_name": "GPT Image 2",
            "required_fields": ["subject", "scene", "garment", "material", "lighting", "style"],
            "missing_fields": ["garment", "material", "lighting", "style"]
        }),
    })
    .expect("prompt");

    let result = generate_quality_review(
        &repo,
        QualityReviewGenerationInput {
            project_id: project.id.clone(),
            user_direction: "Review before delivery.".to_string(),
        },
        "0.19.0".to_string(),
    )
    .expect("quality review");

    assert_eq!(result.review.project_id, project.id);
    assert!(result.review.score < 100);
    assert!(result.checks.iter().any(|check| check.category == "storyboard_duration"));
    assert!(result.checks.iter().any(|check| check.category == "shot_repetition"));
    assert!(result.checks.iter().any(|check| check.category == "garment_visibility"));
    assert!(result.checks.iter().any(|check| check.category == "prompt_completeness"));
    assert!(result
        .suggestions
        .iter()
        .any(|suggestion| suggestion.target_type == "shot" && suggestion.field == "description"));
    assert!(result
        .suggestions
        .iter()
        .any(|suggestion| suggestion.target_type == "prompt_package" && suggestion.field == "prompt_text"));
    assert!(result
        .suggestions
        .iter()
        .all(|suggestion| suggestion.status == QualityReviewSuggestionStatus::Pending.as_str()));
    assert_eq!(result.agent_run.runtime_mode, "local_quality_review_bridge");
    assert!(!result.agent_events.is_empty());
    assert_eq!(brand.id, project.brand_id);
}
```

- [ ] **Step 2: Run generator test and verify RED**

Run:

```powershell
cargo test generate_quality_review_detects_storyboard_prompt_and_product_issues
```

Expected:

```text
FAILED generate_quality_review_detects_storyboard_prompt_and_product_issues
```

The failure must be because `quality_review` module and generation types do not exist.

- [ ] **Step 3: Create service types**

Create `src-tauri/src/quality_review.rs` with these public types:

```rust
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::agent_context::build_project_context;
use crate::error::{JoiError, JoiResult};
use crate::models::{AgentRun, AgentRunEvent, PromptPackage, QualityReview, Shot};
use crate::prompt_adapter::prompt_package_view;
use crate::repositories::{AgentRunCreate, AgentRunEventCreate, QualityReviewCreate, Repository};

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
```

Add `pub mod quality_review;` to `src-tauri/src/lib.rs`.

- [ ] **Step 4: Implement generation entrypoint**

Add this public function to `quality_review.rs`:

```rust
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
    let brand_terms = build_brand_terms(&context.brand.name, &context.brand.description, &creative_directions);

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
```

- [ ] **Step 5: Implement deterministic helper functions**

Add helper functions in the same file:

```rust
fn build_product_terms(brand_description: &str, product_understandings: &[crate::models::ProductUnderstanding]) -> Vec<String> {
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
    creative_directions: &[crate::models::CreativeDirection],
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
        .split(|character: char| character == ',' || character == ';' || character == '/' || character == '，' || character == '；')
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
```

Also add:

```rust
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
```

- [ ] **Step 6: Implement storyboard review**

Add:

```rust
fn review_storyboards(
    project_id: &str,
    project_duration_seconds: i64,
    storyboards: &[crate::repositories::StoryboardWithShots],
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
        if shot_total != storyboard.storyboard.duration_seconds || shot_total != project_duration_seconds {
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
            review_shot_visibility(project_id, &storyboard.shots[index], product_terms, checks, suggestions);
            review_shot_brand(&storyboard.shots[index], brand_terms, suggestions);
            if index > 0 {
                review_shot_repetition(&storyboard.shots[index - 1], &storyboard.shots[index], checks, suggestions);
            }
        }
    }
}
```

Add supporting shot functions:

```rust
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
        message: format!("Shot {} does not clearly mention the garment or selling point.", shot.shot_number),
        evidence: vec![shot.description.clone()],
        suggestion_ids: vec![],
    });
    let suggested_value = append_sentence(
        &shot.description,
        "Keep the garment silhouette and key material benefit clearly visible in frame.",
    );
    suggestions.push(QualityReviewSuggestion {
        id: format!("suggest-shot-{}-description", shot.id),
        target_type: "shot".to_string(),
        target_id: shot.id.clone(),
        field: "description".to_string(),
        current_value: shot.description.clone(),
        suggested_value,
        rationale: format!("Shot should surface a visible garment cue for project {}.", project_id),
        status: QualityReviewSuggestionStatus::Pending.as_str().to_string(),
        check_ids: vec![check_id],
    });
}
```

```rust
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
        message: format!("Shot {} repeats the previous shot too closely.", current.shot_number),
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
```

```rust
fn review_shot_brand(
    shot: &Shot,
    brand_terms: &[String],
    suggestions: &mut Vec<QualityReviewSuggestion>,
) {
    if brand_terms.is_empty() {
        return;
    }
    let shot_text = format!("{} {} {} {}", shot.description, shot.scene, shot.lighting, shot.rationale);
    if text_contains_any_term(&shot_text, brand_terms) {
        return;
    }
    if suggestions
        .iter()
        .any(|suggestion| suggestion.target_type == "shot" && suggestion.target_id == shot.id && suggestion.field == "description")
    {
        return;
    }
    suggestions.push(QualityReviewSuggestion {
        id: format!("suggest-shot-{}-brand-description", shot.id),
        target_type: "shot".to_string(),
        target_id: shot.id.clone(),
        field: "description".to_string(),
        current_value: shot.description.clone(),
        suggested_value: append_sentence(&shot.description, "Maintain the brand's established visual tone in the shot."),
        rationale: "Shot should stay aligned with saved brand and creative direction.".to_string(),
        status: QualityReviewSuggestionStatus::Pending.as_str().to_string(),
        check_ids: vec![],
    });
}
```

- [ ] **Step 7: Implement prompt review**

Add:

```rust
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
                rationale: "Prompt adapters need complete fields for reliable model handoff.".to_string(),
                status: QualityReviewSuggestionStatus::Pending.as_str().to_string(),
                check_ids: vec![check_id],
            });
        }

        let context_terms = product_terms
            .iter()
            .chain(brand_terms.iter())
            .cloned()
            .collect::<Vec<_>>();
        if !context_terms.is_empty() && !text_contains_any_term(&package.prompt_text, &context_terms) {
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
            if !suggestions.iter().any(|suggestion| {
                suggestion.target_type == "prompt_package"
                    && suggestion.target_id == package.id
                    && suggestion.field == "prompt_text"
            }) {
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
                    rationale: "Prompt should carry the same brand/product context as the project brief.".to_string(),
                    status: QualityReviewSuggestionStatus::Pending.as_str().to_string(),
                    check_ids: vec![check_id],
                });
            }
        }
    }
}
```

- [ ] **Step 8: Implement scoring and event helpers**

Add:

```rust
fn link_suggestions_to_checks(checks: &mut [QualityReviewCheck], suggestions: &[QualityReviewSuggestion]) {
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
        .map(|check| match (check.status.as_str(), check.severity.as_str()) {
            ("failed", "high") => 18,
            ("failed", _) => 12,
            ("warning", "high") => 10,
            ("warning", _) => 6,
            _ => 0,
        })
        .sum::<i64>();
    (100 - penalty).clamp(0, 100)
}

fn review_summary(score: i64, checks: &[QualityReviewCheck], suggestions: &[QualityReviewSuggestion]) -> String {
    let failed_count = checks.iter().filter(|check| check.status == "failed").count();
    let warning_count = checks.iter().filter(|check| check.status == "warning").count();
    let pending_suggestions = suggestions
        .iter()
        .filter(|suggestion| suggestion.status == QualityReviewSuggestionStatus::Pending.as_str())
        .count();
    format!(
        "Quality review scored {}/100 with {} failed check(s), {} warning(s), and {} pending suggestion(s).",
        score, failed_count, warning_count, pending_suggestions
    )
}

fn quality_review_goal(project_title: &str, input: &QualityReviewGenerationInput) -> String {
    if input.user_direction.trim().is_empty() {
        format!("Review content quality for {}.", project_title)
    } else {
        format!("Review content quality for {}: {}", project_title, input.user_direction.trim())
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
            format!("Read project context for quality review."),
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
```

Add text helpers:

```rust
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
```

- [ ] **Step 9: Run generator test and verify PASS**

Run:

```powershell
cargo test generate_quality_review_detects_storyboard_prompt_and_product_issues
```

Expected:

```text
test result: ok
```

- [ ] **Step 10: Commit Task 2**

Run:

```powershell
git add src-tauri/src/quality_review.rs src-tauri/src/lib.rs src-tauri/tests/quality_review.rs
git commit -m "feat: add quality review generator"
```

### Task 3: Apply Review Suggestions

**Files:**

- Modify: `src-tauri/src/quality_review.rs`
- Test: `src-tauri/tests/quality_review.rs`

- [ ] **Step 1: Write failing apply-suggestion test**

Append this test to `src-tauri/tests/quality_review.rs`:

```rust
#[test]
fn apply_quality_review_suggestion_updates_shot_and_marks_suggestion_applied() {
    let db = TestDb::new();
    let repo = Repository::new(db.database.connection());
    let (_brand, project) = helpers::create_brand_and_project(&repo);

    let storyboard = repo
        .create_storyboard(StoryboardCreate {
            project_id: project.id.clone(),
            title: "Shot edit storyboard".to_string(),
            duration_seconds: 15,
        })
        .expect("storyboard");
    let shot = repo
        .create_shot_plan(ShotPlanCreate {
            storyboard_id: storyboard.id,
            shot_number: 1,
            duration_seconds: 5,
            visual_description: "Model walks forward.".to_string(),
            model_action: "Model walks.".to_string(),
            garment_focus: "outerwear".to_string(),
            camera_movement: "push in".to_string(),
            scene: "studio".to_string(),
            lighting: "soft".to_string(),
            transition: "cut".to_string(),
            subtitle_or_text: "Light layer".to_string(),
            rationale: "Opening motion.".to_string(),
            source_memory_ids: vec![],
            source_research_report_ids: vec![],
            generation_context: json!({}),
        })
        .expect("shot");

    let suggestion_id = format!("suggest-shot-{}-description", shot.id);
    let review = repo
        .create_quality_review(QualityReviewCreate {
            project_id: project.id.clone(),
            summary: "Quality review scored 88/100 with 0 failed check(s), 1 warning(s), and 1 pending suggestion(s).".to_string(),
            score: 88,
            checklist_json: json!([]),
            suggestions_json: json!([
                {
                    "id": suggestion_id,
                    "target_type": "shot",
                    "target_id": shot.id,
                    "field": "description",
                    "current_value": "Model walks forward.",
                    "suggested_value": "Model walks forward while the outerwear silhouette stays visible.",
                    "rationale": "Make garment visibility explicit.",
                    "status": "pending",
                    "check_ids": []
                }
            ]),
        })
        .expect("review");

    let result = joi_agent_lib::quality_review::apply_quality_review_suggestion(
        &repo,
        joi_agent_lib::quality_review::ApplyReviewSuggestionInput {
            review_id: review.id,
            suggestion_id,
        },
        "0.19.0".to_string(),
    )
    .expect("applied");

    assert_eq!(result.applied_target_type, "shot");
    assert_eq!(result.suggestion.status, "applied");
    assert!(result
        .updated_review
        .suggestions_json[0]["status"]
        .as_str()
        .is_some_and(|status| status == "applied"));

    let updated_shot = repo.get_shot(&result.applied_target_id).expect("updated shot");
    assert_eq!(
        updated_shot.description,
        "Model walks forward while the outerwear silhouette stays visible."
    );
    assert_eq!(result.agent_run.runtime_mode, "local_quality_iteration_bridge");
}
```

- [ ] **Step 2: Write failing prompt apply test**

Append:

```rust
#[test]
fn apply_quality_review_suggestion_updates_prompt_text() {
    let db = TestDb::new();
    let repo = Repository::new(db.database.connection());
    let (_brand, project) = helpers::create_brand_and_project(&repo);
    let prompt = repo
        .create_prompt_package(PromptPackageCreate {
            project_id: project.id.clone(),
            shot_id: None,
            platform: "gpt_image_2".to_string(),
            modality: "image".to_string(),
            prompt_text: "Create a model photo.".to_string(),
            negative_prompt: "avoid distorted garment".to_string(),
            parameters_json: json!({
                "format_version": "joi.prompt_package_parameters.v1",
                "adapter_profile_id": "gpt_image_2",
                "missing_fields": ["garment"]
            }),
        })
        .expect("prompt");

    let suggestion_id = format!("suggest-prompt-{}-missing-fields", prompt.id);
    let review = repo
        .create_quality_review(QualityReviewCreate {
            project_id: project.id,
            summary: "Quality review scored 82/100 with 1 failed check(s), 0 warning(s), and 1 pending suggestion(s).".to_string(),
            score: 82,
            checklist_json: json!([]),
            suggestions_json: json!([
                {
                    "id": suggestion_id,
                    "target_type": "prompt_package",
                    "target_id": prompt.id,
                    "field": "prompt_text",
                    "current_value": "Create a model photo.",
                    "suggested_value": "Create a model photo. Include: garment.",
                    "rationale": "Complete provider fields.",
                    "status": "pending",
                    "check_ids": []
                }
            ]),
        })
        .expect("review");

    let result = joi_agent_lib::quality_review::apply_quality_review_suggestion(
        &repo,
        joi_agent_lib::quality_review::ApplyReviewSuggestionInput {
            review_id: review.id,
            suggestion_id,
        },
        "0.19.0".to_string(),
    )
    .expect("applied");

    let updated_prompt = repo
        .get_prompt_package(&result.applied_target_id)
        .expect("updated prompt");
    assert_eq!(updated_prompt.prompt_text, "Create a model photo. Include: garment.");
    assert_eq!(updated_prompt.negative_prompt, "avoid distorted garment");
    assert_eq!(updated_prompt.parameters_json["missing_fields"], json!(["garment"]));
}
```

- [ ] **Step 3: Run apply tests and verify RED**

Run:

```powershell
cargo test apply_quality_review_suggestion_updates
```

Expected:

```text
FAILED apply_quality_review_suggestion_updates_shot_and_marks_suggestion_applied
FAILED apply_quality_review_suggestion_updates_prompt_text
```

The failure must be because apply types and function do not exist.

- [ ] **Step 4: Add apply types**

In `quality_review.rs`, add:

```rust
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
```

- [ ] **Step 5: Implement suggestion parsing**

Add:

```rust
fn suggestions_from_review(review: &QualityReview) -> JoiResult<Vec<QualityReviewSuggestion>> {
    serde_json::from_value(review.suggestions_json.clone()).map_err(|err| {
        JoiError::Validation(format!("quality review suggestions are malformed: {err}"))
    })
}

fn suggestion_to_value(suggestions: &[QualityReviewSuggestion]) -> Value {
    json!(suggestions)
}
```

- [ ] **Step 6: Implement apply entrypoint**

Add:

```rust
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
        .ok_or_else(|| JoiError::NotFound(format!("quality review suggestion {}", input.suggestion_id)))?;
    if suggestions[index].status != QualityReviewSuggestionStatus::Pending.as_str() {
        return Err(JoiError::Validation(format!(
            "quality review suggestion {} is not pending",
            input.suggestion_id
        )));
    }

    apply_supported_target(repo, &suggestions[index])?;
    suggestions[index].status = QualityReviewSuggestionStatus::Applied.as_str().to_string();
    let updated_review =
        repo.update_quality_review_suggestions(&review.id, suggestion_to_value(&suggestions))?;
    let applied = suggestions[index].clone();

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
        applied_target_type: suggestions[index].target_type.clone(),
        applied_target_id: suggestions[index].target_id.clone(),
        suggestion: suggestions[index].clone(),
        agent_run,
        agent_events,
    })
}
```

- [ ] **Step 7: Implement target application**

Add:

```rust
fn apply_supported_target(repo: &Repository<'_>, suggestion: &QualityReviewSuggestion) -> JoiResult<()> {
    match (suggestion.target_type.as_str(), suggestion.field.as_str()) {
        ("shot", "description") => apply_shot_description(repo, suggestion),
        ("prompt_package", "prompt_text") => apply_prompt_text(repo, suggestion),
        (target_type, field) => Err(JoiError::Validation(format!(
            "review suggestion target is not supported: {target_type}.{field}"
        ))),
    }
}

fn apply_shot_description(repo: &Repository<'_>, suggestion: &QualityReviewSuggestion) -> JoiResult<()> {
    let shot = repo.get_shot(&suggestion.target_id)?;
    if shot.is_locked {
        return Err(JoiError::Validation(
            "Locked shots cannot be updated from review suggestions".to_string(),
        ));
    }
    repo.update_shot(crate::repositories::ShotUpdate {
        id: shot.id,
        duration_seconds: shot.duration_seconds,
        visual_description: suggestion.suggested_value.clone(),
        model_action: shot.model_action,
        garment_focus: shot
            .metadata_json
            .get("garment_focus")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        camera_movement: shot.camera_movement,
        scene: shot.scene,
        lighting: shot.lighting,
        transition: shot
            .metadata_json
            .get("transition")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
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
    repo.update_prompt_package(crate::repositories::PromptPackageUpdate {
        id: package.id,
        prompt_text: suggestion.suggested_value.clone(),
        negative_prompt: package.negative_prompt,
        parameters_json: package.parameters_json,
        is_locked: package.is_locked,
    })?;
    Ok(())
}
```

- [ ] **Step 8: Run apply tests and verify PASS**

Run:

```powershell
cargo test apply_quality_review_suggestion_updates
```

Expected:

```text
test result: ok
```

- [ ] **Step 9: Commit Task 3**

Run:

```powershell
git add src-tauri/src/quality_review.rs src-tauri/tests/quality_review.rs
git commit -m "feat: apply quality review suggestions"
```

### Task 4: Commands And Snapshot Integration

**Files:**

- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/snapshots.rs`
- Test: `src-tauri/tests/commands.rs`
- Test: `src-tauri/tests/project_snapshots.rs`

- [ ] **Step 1: Write failing command test**

Add to `src-tauri/tests/commands.rs`:

```rust
#[test]
fn quality_review_commands_generate_list_and_apply_suggestions() {
    let state = test_app_state();
    let brand = create_brand(
        &state,
        BrandInput {
            name: "Atelier Joi".to_string(),
            description: "Premium studio outerwear".to_string(),
        },
    )
    .expect("brand");
    let project = create_project(
        &state,
        ProjectInput {
            brand_id: brand.id,
            title: "Spring Drop Film".to_string(),
            advertising_goal: "Launch awareness".to_string(),
            duration_seconds: 15,
        },
    )
    .expect("project");

    generate_brief_understanding(
        &state,
        BriefUnderstandingInput {
            project_id: project.id.clone(),
            brief_text: "15s outerwear ad".to_string(),
            product_name: "Lightweight Trench".to_string(),
            category: "outerwear".to_string(),
            audience: "urban commuters".to_string(),
            target_platforms: vec!["gpt_image_2".to_string()],
            selling_points: vec!["water-resistant cotton".to_string()],
            visual_direction: "clean warm studio".to_string(),
            constraints: vec![],
        },
    )
    .expect("understanding");

    let storyboard = generate_storyboard(
        &state,
        StoryboardGenerationInput {
            project_id: project.id.clone(),
            user_direction: "Generate a studio sequence.".to_string(),
            preferred_duration_seconds: Some(15),
            preferred_shot_count: Some(5),
        },
    )
    .expect("storyboard");
    generate_prompt_packages(
        &state,
        PromptGenerationInput {
            project_id: project.id.clone(),
            shot_ids: vec![storyboard.shots[0].shot.id.clone()],
            image_brief: "Ecommerce model photo".to_string(),
            target_platforms: vec!["gpt_image_2".to_string()],
            user_direction: "Image prompt".to_string(),
        },
    )
    .expect("prompts");

    let review = generate_quality_review(
        &state,
        QualityReviewGenerationInput {
            project_id: project.id.clone(),
            user_direction: "Check before delivery.".to_string(),
        },
    )
    .expect("review");
    assert_eq!(review.review.project_id, project.id);

    let listed = list_quality_reviews(&state, project.id).expect("listed reviews");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, review.review.id);
}
```

Add imports at the top of `commands.rs` test:

```rust
use joi_agent_lib::quality_review::QualityReviewGenerationInput;
```

Also add `generate_quality_review` and `list_quality_reviews` to the imported command helpers.

- [ ] **Step 2: Write failing snapshot test**

Add to `src-tauri/tests/project_snapshots.rs`:

```rust
#[test]
fn snapshot_includes_quality_reviews() {
    let db = TestDb::new();
    let repo = Repository::new(db.database.connection());
    let (_brand, project) = create_brand_and_project(&repo);
    let review = repo
        .create_quality_review(QualityReviewCreate {
            project_id: project.id.clone(),
            summary: "Quality review scored 100/100 with 0 failed check(s), 0 warning(s), and 0 pending suggestion(s).".to_string(),
            score: 100,
            checklist_json: json!([]),
            suggestions_json: json!([]),
        })
        .expect("review");

    let service = ProjectSnapshotService::new(db.database.connection());
    let snapshot = service.build_snapshot(&project.id).expect("snapshot");

    assert_eq!(snapshot["quality_reviews"][0]["id"], review.id);
    assert_eq!(snapshot["quality_reviews"][0]["score"], 100);
}
```

Add imports if missing:

```rust
use joi_agent_lib::repositories::QualityReviewCreate;
use serde_json::json;
```

- [ ] **Step 3: Run command and snapshot tests and verify RED**

Run:

```powershell
cargo test quality_review_commands_generate_list_and_apply_suggestions
cargo test snapshot_includes_quality_reviews
```

Expected:

```text
FAILED quality_review_commands_generate_list_and_apply_suggestions
FAILED snapshot_includes_quality_reviews
```

The command test should fail because command helpers do not exist. The snapshot test should fail because the snapshot omits `quality_reviews`.

- [ ] **Step 4: Add command imports and public command wrappers**

In `src-tauri/src/commands.rs`, add imports:

```rust
use crate::quality_review::{
    apply_quality_review_suggestion as apply_quality_review_suggestion_service,
    generate_quality_review as generate_quality_review_service,
    ApplyReviewSuggestionInput, ApplyReviewSuggestionResult, QualityReviewGenerationInput,
    QualityReviewGenerationResult,
};
```

Add `QualityReview` to the `models` import list.

Add Tauri commands:

```rust
#[tauri::command(rename_all = "snake_case")]
pub fn joi_generate_quality_review(
    state: State<'_, AppState>,
    input: QualityReviewGenerationInput,
) -> JoiResult<QualityReviewGenerationResult> {
    generate_quality_review(state.inner(), input)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_list_quality_reviews(
    state: State<'_, AppState>,
    project_id: String,
) -> JoiResult<Vec<QualityReview>> {
    list_quality_reviews(state.inner(), project_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_apply_quality_review_suggestion(
    state: State<'_, AppState>,
    input: ApplyReviewSuggestionInput,
) -> JoiResult<ApplyReviewSuggestionResult> {
    apply_quality_review_suggestion(state.inner(), input)
}
```

Add helper functions:

```rust
pub fn generate_quality_review(
    state: &AppState,
    input: QualityReviewGenerationInput,
) -> JoiResult<QualityReviewGenerationResult> {
    let runtime_status = get_agent_runtime_status(state)?;
    let db = lock_db(state)?;
    generate_quality_review_service(
        &Repository::new(db.connection()),
        input,
        runtime_status.hermes_version,
    )
}

pub fn list_quality_reviews(state: &AppState, project_id: String) -> JoiResult<Vec<QualityReview>> {
    let db = lock_db(state)?;
    Repository::new(db.connection()).list_quality_reviews(&project_id)
}

pub fn apply_quality_review_suggestion(
    state: &AppState,
    input: ApplyReviewSuggestionInput,
) -> JoiResult<ApplyReviewSuggestionResult> {
    let runtime_status = get_agent_runtime_status(state)?;
    let db = lock_db(state)?;
    apply_quality_review_suggestion_service(
        &Repository::new(db.connection()),
        input,
        runtime_status.hermes_version,
    )
}
```

- [ ] **Step 5: Register commands**

In `src-tauri/src/lib.rs`, add to `tauri::generate_handler!`:

```rust
commands::joi_generate_quality_review,
commands::joi_list_quality_reviews,
commands::joi_apply_quality_review_suggestion,
```

Place them after prompt commands and before delivery commands so the command order matches the product workflow.

- [ ] **Step 6: Add snapshot field**

In `src-tauri/src/snapshots.rs`, add:

```rust
"quality_reviews": repo.list_quality_reviews(project_id)?,
```

Place it after `"prompt_packages"` and before `"delivery_reports"`.

- [ ] **Step 7: Run command and snapshot tests and verify PASS**

Run:

```powershell
cargo test quality_review_commands_generate_list_and_apply_suggestions
cargo test snapshot_includes_quality_reviews
```

Expected:

```text
test result: ok
```

- [ ] **Step 8: Commit Task 4**

Run:

```powershell
git add src-tauri/src/commands.rs src-tauri/src/lib.rs src-tauri/src/snapshots.rs src-tauri/tests/commands.rs src-tauri/tests/project_snapshots.rs
git commit -m "feat: expose quality review commands"
```

### Task 5: Frontend Review Workspace

**Files:**

- Modify: `src/types/joi.ts`
- Modify: `src/api/joiApi.ts`
- Create: `src/components/ReviewWorkspace.tsx`
- Modify: `src/components/BrandProjectRail.tsx`
- Modify: `src/components/ProjectWorkspace.tsx`
- Modify: `src/App.tsx`
- Modify: `src/styles.css`
- Test: `src/App.test.tsx`

- [ ] **Step 1: Write failing frontend test**

In `src/App.test.tsx`, add mock helpers:

```ts
const mockQualityReviewResult = {
  review: {
    id: "quality-review-1",
    project_id: "project-1",
    summary: "Quality review scored 82/100 with 1 failed check(s), 1 warning(s), and 2 pending suggestion(s).",
    score: 82,
    checklist_json: [
      {
        id: "prompt-completeness-prompt-gpt_image_2",
        category: "prompt_completeness",
        title: "Prompt contains required adapter fields",
        status: "failed",
        severity: "high",
        source_type: "prompt_package",
        source_id: "prompt-gpt_image_2",
        message: "GPT Image 2 is missing required field(s): garment.",
        evidence: ["garment"],
        suggestion_ids: ["suggest-prompt-prompt-gpt_image_2-missing-fields"],
      },
    ],
    suggestions_json: [
      {
        id: "suggest-prompt-prompt-gpt_image_2-missing-fields",
        target_type: "prompt_package",
        target_id: "prompt-gpt_image_2",
        field: "prompt_text",
        current_value: "Create a model photo.",
        suggested_value: "Create a model photo. Include: garment.",
        rationale: "Complete provider fields.",
        status: "pending",
        check_ids: ["prompt-completeness-prompt-gpt_image_2"],
      },
    ],
    created_at: "2026-06-15T00:00:00Z",
    updated_at: "2026-06-15T00:00:00Z",
  },
  checks: [
    {
      id: "prompt-completeness-prompt-gpt_image_2",
      category: "prompt_completeness",
      title: "Prompt contains required adapter fields",
      status: "failed",
      severity: "high",
      source_type: "prompt_package",
      source_id: "prompt-gpt_image_2",
      message: "GPT Image 2 is missing required field(s): garment.",
      evidence: ["garment"],
      suggestion_ids: ["suggest-prompt-prompt-gpt_image_2-missing-fields"],
    },
  ],
  suggestions: [
    {
      id: "suggest-prompt-prompt-gpt_image_2-missing-fields",
      target_type: "prompt_package",
      target_id: "prompt-gpt_image_2",
      field: "prompt_text",
      current_value: "Create a model photo.",
      suggested_value: "Create a model photo. Include: garment.",
      rationale: "Complete provider fields.",
      status: "pending",
      check_ids: ["prompt-completeness-prompt-gpt_image_2"],
    },
  ],
  agent_run: {
    id: "run-review",
    project_id: "project-1",
    user_goal: "Review content quality for Spring Drop Film.",
    status: "completed",
    runtime_kind: "hermes_core",
    runtime_mode: "local_quality_review_bridge",
    runtime_version: "0.19.0",
    roles_json: ["reviewer"],
    plan_json: [],
    result_summary: "Quality review scored 82/100.",
    created_at: "2026-06-15T00:00:00Z",
    updated_at: "2026-06-15T00:00:00Z",
  },
  agent_events: [],
};
```

Add mock cases:

```ts
case "joi_list_quality_reviews":
  return Promise.resolve([]);
case "joi_generate_quality_review":
  return Promise.resolve(mockQualityReviewResult);
case "joi_apply_quality_review_suggestion":
  return Promise.resolve({
    updated_review: {
      ...mockQualityReviewResult.review,
      suggestions_json: [
        {
          ...mockQualityReviewResult.suggestions[0],
          status: "applied",
        },
      ],
    },
    suggestion: {
      ...mockQualityReviewResult.suggestions[0],
      status: "applied",
    },
    applied_target_type: "prompt_package",
    applied_target_id: "prompt-gpt_image_2",
    agent_run: {
      ...mockQualityReviewResult.agent_run,
      id: "run-review-apply",
      runtime_mode: "local_quality_iteration_bridge",
    },
    agent_events: [],
  });
```

Add test:

```ts
test("generates and applies a quality review suggestion", async () => {
  render(<App />);

  await screen.findByText("Spring Drop Film");
  fireEvent.click(screen.getByRole("button", { name: "Review" }));

  fireEvent.change(screen.getByLabelText("Review direction"), {
    target: { value: "Check prompt completeness before delivery." },
  });
  fireEvent.click(screen.getByRole("button", { name: "Generate Review" }));

  await screen.findByText("82/100");
  expect(screen.getByText("Prompt contains required adapter fields")).toBeInTheDocument();
  expect(screen.getByText("Create a model photo. Include: garment.")).toBeInTheDocument();

  fireEvent.click(screen.getByRole("button", { name: "Apply Suggestion" }));
  await waitFor(() => {
    expect(invokeMock).toHaveBeenCalledWith(
      "joi_apply_quality_review_suggestion",
      expect.objectContaining({
        input: {
          review_id: "quality-review-1",
          suggestion_id: "suggest-prompt-prompt-gpt_image_2-missing-fields",
        },
      }),
    );
  });
});
```

- [ ] **Step 2: Run frontend test and verify RED**

Run:

```powershell
npm test -- --run src/App.test.tsx -t "quality review"
```

Expected:

```text
FAIL src/App.test.tsx
```

The failure must be because the Review tab/workspace/API does not exist.

- [ ] **Step 3: Add TypeScript types**

In `src/types/joi.ts`, add:

```ts
export type QualityReview = {
  id: string;
  project_id: string;
  summary: string;
  score: number;
  checklist_json: QualityReviewCheck[];
  suggestions_json: QualityReviewSuggestion[];
  created_at: string;
  updated_at: string;
};

export type QualityReviewGenerationInput = {
  project_id: string;
  user_direction: string;
};

export type QualityReviewCheck = {
  id: string;
  category: string;
  title: string;
  status: string;
  severity: string;
  source_type: string;
  source_id: string;
  message: string;
  evidence: string[];
  suggestion_ids: string[];
};

export type QualityReviewSuggestion = {
  id: string;
  target_type: string;
  target_id: string;
  field: string;
  current_value: string;
  suggested_value: string;
  rationale: string;
  status: string;
  check_ids: string[];
};

export type QualityReviewGenerationResult = {
  review: QualityReview;
  checks: QualityReviewCheck[];
  suggestions: QualityReviewSuggestion[];
  agent_run: AgentRun;
  agent_events: AgentRunEvent[];
};

export type ApplyReviewSuggestionInput = {
  review_id: string;
  suggestion_id: string;
};

export type ApplyReviewSuggestionResult = {
  updated_review: QualityReview;
  suggestion: QualityReviewSuggestion;
  applied_target_type: string;
  applied_target_id: string;
  agent_run: AgentRun;
  agent_events: AgentRunEvent[];
};
```

- [ ] **Step 4: Add API wrappers**

In `src/api/joiApi.ts`, import the new types and add:

```ts
export function generateQualityReview(
  input: QualityReviewGenerationInput,
): Promise<QualityReviewGenerationResult> {
  return invoke<QualityReviewGenerationResult>("joi_generate_quality_review", { input });
}

export function listQualityReviews(projectId: string): Promise<QualityReview[]> {
  return invoke<QualityReview[]>("joi_list_quality_reviews", { project_id: projectId });
}

export function applyQualityReviewSuggestion(
  input: ApplyReviewSuggestionInput,
): Promise<ApplyReviewSuggestionResult> {
  return invoke<ApplyReviewSuggestionResult>("joi_apply_quality_review_suggestion", { input });
}
```

- [ ] **Step 5: Create Review workspace component**

Create `src/components/ReviewWorkspace.tsx`:

```tsx
import type { FormEvent } from "react";

import type {
  Project,
  QualityReview,
  QualityReviewCheck,
  QualityReviewSuggestion,
} from "../types/joi";

export type ReviewDraft = {
  user_direction: string;
};

type ReviewWorkspaceProps = {
  applyingSuggestionId: string | null;
  generatingQualityReview: boolean;
  latestChecks: QualityReviewCheck[];
  latestSuggestions: QualityReviewSuggestion[];
  onApplySuggestion: (reviewId: string, suggestionId: string) => void;
  onReviewDraftChange: (field: keyof ReviewDraft, value: string) => void;
  onSubmitReview: () => void;
  qualityReviews: QualityReview[];
  reviewDraft: ReviewDraft;
  selectedProject: Project | null;
};

export function ReviewWorkspace({
  applyingSuggestionId,
  generatingQualityReview,
  latestChecks,
  latestSuggestions,
  onApplySuggestion,
  onReviewDraftChange,
  onSubmitReview,
  qualityReviews,
  reviewDraft,
  selectedProject,
}: ReviewWorkspaceProps) {
  const latestReview = qualityReviews[qualityReviews.length - 1] ?? null;
  const checks = latestChecks.length > 0 ? latestChecks : normalizeChecks(latestReview);
  const suggestions =
    latestSuggestions.length > 0 ? latestSuggestions : normalizeSuggestions(latestReview);

  return (
    <div className="review-layout">
      <section className="workspace-panel wide">
        <div className="section-heading">
          <h2>Quality Review</h2>
          <span className="muted">
            {latestReview ? `${latestReview.score}/100` : "No review yet"}
          </span>
        </div>
        <form className="review-generator-form" onSubmit={submit(onSubmitReview)}>
          <label className="wide-field">
            Review direction
            <textarea
              disabled={!selectedProject || generatingQualityReview}
              onChange={(event) => onReviewDraftChange("user_direction", event.target.value)}
              placeholder="Check prompt completeness before delivery."
              rows={3}
              value={reviewDraft.user_direction}
            />
          </label>
          <button disabled={!selectedProject || generatingQualityReview} type="submit">
            {generatingQualityReview ? "Generating" : "Generate Review"}
          </button>
        </form>
      </section>

      <section className="workspace-panel wide">
        <div className="section-heading">
          <h2>Review Checklist</h2>
          <span className="muted">{checks.length} check(s)</span>
        </div>
        {latestReview ? <strong className="review-score">{latestReview.score}/100</strong> : null}
        {checks.length > 0 ? (
          <div className="review-check-list">
            {checks.map((check) => (
              <article className={`review-check ${check.status}`} key={check.id}>
                <div>
                  <strong>{check.title}</strong>
                  <span>{check.message}</span>
                </div>
                <small>{check.category} · {check.severity}</small>
              </article>
            ))}
          </div>
        ) : (
          <p className="muted">Generate a review after storyboard and prompt packages are ready.</p>
        )}
      </section>

      <section className="workspace-panel wide">
        <div className="section-heading">
          <h2>Revision Suggestions</h2>
          <span className="muted">{suggestions.length} suggestion(s)</span>
        </div>
        {suggestions.length > 0 && latestReview ? (
          <div className="review-suggestion-list">
            {suggestions.map((suggestion) => (
              <article className="review-suggestion" key={suggestion.id}>
                <div className="review-suggestion-copy">
                  <strong>{targetLabel(suggestion)}</strong>
                  <p>{suggestion.suggested_value}</p>
                  <small>{suggestion.rationale}</small>
                </div>
                <button
                  disabled={suggestion.status !== "pending" || applyingSuggestionId === suggestion.id}
                  onClick={() => onApplySuggestion(latestReview.id, suggestion.id)}
                  type="button"
                >
                  {suggestion.status === "applied"
                    ? "Applied"
                    : applyingSuggestionId === suggestion.id
                      ? "Applying"
                      : "Apply Suggestion"}
                </button>
              </article>
            ))}
          </div>
        ) : (
          <p className="muted">Accepted suggestions will update the target shot or prompt package.</p>
        )}
      </section>
    </div>
  );
}

function normalizeChecks(review: QualityReview | null): QualityReviewCheck[] {
  return Array.isArray(review?.checklist_json) ? review.checklist_json : [];
}

function normalizeSuggestions(review: QualityReview | null): QualityReviewSuggestion[] {
  return Array.isArray(review?.suggestions_json) ? review.suggestions_json : [];
}

function targetLabel(suggestion: QualityReviewSuggestion): string {
  return `${suggestion.target_type} · ${suggestion.field}`;
}

function submit(handler: () => void) {
  return (event: FormEvent) => {
    event.preventDefault();
    handler();
  };
}
```

- [ ] **Step 6: Add Review tab**

In `src/components/BrandProjectRail.tsx`, insert `Review` before `Delivery` in the workflow tabs:

```ts
const workflowTabs = [
  "Overview",
  "Brief",
  "Research",
  "Storyboard",
  "Prompts",
  "Review",
  "Delivery",
  "Assets",
  "Memory",
  "Versions",
];
```

If the file uses an inline array instead of `workflowTabs`, update the inline array with the same order.

- [ ] **Step 7: Wire ProjectWorkspace**

In `src/components/ProjectWorkspace.tsx`:

Import:

```ts
import { ReviewWorkspace, type ReviewDraft } from "./ReviewWorkspace";
```

Add types to the existing imports:

```ts
QualityReview,
QualityReviewCheck,
QualityReviewSuggestion,
```

Add props:

```ts
applyingSuggestionId: string | null;
generatingQualityReview: boolean;
latestReviewChecks: QualityReviewCheck[];
latestReviewSuggestions: QualityReviewSuggestion[];
onApplyReviewSuggestion: (reviewId: string, suggestionId: string) => void;
onReviewDraftChange: (field: keyof ReviewDraft, value: string) => void;
onSubmitQualityReview: () => void;
qualityReviews: QualityReview[];
reviewDraft: ReviewDraft;
```

Destructure those props.

Add render block between `Prompts` and `Delivery`:

```tsx
{activeTab === "Review" ? (
  <ReviewWorkspace
    applyingSuggestionId={applyingSuggestionId}
    generatingQualityReview={generatingQualityReview}
    latestChecks={latestReviewChecks}
    latestSuggestions={latestReviewSuggestions}
    onApplySuggestion={onApplyReviewSuggestion}
    onReviewDraftChange={onReviewDraftChange}
    onSubmitReview={onSubmitQualityReview}
    qualityReviews={qualityReviews}
    reviewDraft={reviewDraft}
    selectedProject={selectedProject}
  />
) : null}
```

Update fallback tab allow-list:

```ts
["Overview", "Brief", "Research", "Storyboard", "Prompts", "Review", "Delivery", "Assets", "Memory", "Versions"]
```

Update workflow map:

```ts
["Brief", "Research", "Creative Direction", "Storyboard", "Prompts", "Review", "Delivery"]
```

- [ ] **Step 8: Wire App state and handlers**

In `src/App.tsx`, import API functions:

```ts
applyQualityReviewSuggestion,
generateQualityReview,
listQualityReviews,
```

Import type:

```ts
import type { ReviewDraft } from "./components/ReviewWorkspace";
```

Import review types:

```ts
QualityReview,
QualityReviewCheck,
QualityReviewSuggestion,
```

Add empty draft:

```ts
const emptyReviewDraft: ReviewDraft = {
  user_direction: "",
};
```

Add state:

```ts
const [applyingSuggestionId, setApplyingSuggestionId] = useState<string | null>(null);
const [generatingQualityReview, setGeneratingQualityReview] = useState(false);
const [latestReviewChecks, setLatestReviewChecks] = useState<QualityReviewCheck[]>([]);
const [latestReviewSuggestions, setLatestReviewSuggestions] = useState<QualityReviewSuggestion[]>([]);
const [qualityReviews, setQualityReviews] = useState<QualityReview[]>([]);
const [reviewDraft, setReviewDraft] = useState<ReviewDraft>(emptyReviewDraft);
```

Reset these when project changes or a new project/brand starts:

```ts
setLatestReviewChecks([]);
setLatestReviewSuggestions([]);
setQualityReviews([]);
setReviewDraft(emptyReviewDraft);
```

In `refreshProjectState`, add `qualityReviewList` before delivery reports:

```ts
qualityReviewList,
```

Add `listQualityReviews(projectId)` to the Promise list after `listPromptPackages(projectId)`.

Set state:

```ts
setQualityReviews(qualityReviewList);
```

Add handler:

```ts
function updateReviewDraft(field: keyof ReviewDraft, value: string) {
  setReviewDraft((draft) => ({ ...draft, [field]: value }));
}
```

Add submit handler:

```ts
async function submitQualityReview() {
  if (!selectedProject) {
    setError("Select a project before generating a quality review.");
    return;
  }
  try {
    setGeneratingQualityReview(true);
    setError(null);
    const result = await generateQualityReview({
      project_id: selectedProject.id,
      user_direction: reviewDraft.user_direction,
    });
    setLatestReviewChecks(result.checks);
    setLatestReviewSuggestions(result.suggestions);
    await refreshProjectState(selectedProject.id);
    setAgentRuns((runs) => [
      { run: result.agent_run, events: result.agent_events },
      ...runs.filter((item) => item.run.id !== result.agent_run.id),
    ]);
    setActivityLog((entries) => [...entries, `Generated quality review ${result.review.score}/100.`]);
  } catch (submitError) {
    setError(formatError(submitError));
  } finally {
    setGeneratingQualityReview(false);
  }
}
```

Add apply handler:

```ts
async function applyReviewSuggestion(reviewId: string, suggestionId: string) {
  if (!selectedProject) {
    setError("Select a project before applying a review suggestion.");
    return;
  }
  try {
    setApplyingSuggestionId(suggestionId);
    setError(null);
    const result = await applyQualityReviewSuggestion({
      review_id: reviewId,
      suggestion_id: suggestionId,
    });
    setLatestReviewSuggestions((suggestions) =>
      suggestions.map((suggestion) =>
        suggestion.id === suggestionId ? result.suggestion : suggestion,
      ),
    );
    await refreshProjectState(selectedProject.id);
    setAgentRuns((runs) => [
      { run: result.agent_run, events: result.agent_events },
      ...runs.filter((item) => item.run.id !== result.agent_run.id),
    ]);
    setActivityLog((entries) => [
      ...entries,
      `Applied review suggestion to ${result.applied_target_type}.`,
    ]);
  } catch (submitError) {
    setError(formatError(submitError));
  } finally {
    setApplyingSuggestionId(null);
  }
}
```

Pass props to `ProjectWorkspace`:

```tsx
applyingSuggestionId={applyingSuggestionId}
generatingQualityReview={generatingQualityReview}
latestReviewChecks={latestReviewChecks}
latestReviewSuggestions={latestReviewSuggestions}
onApplyReviewSuggestion={applyReviewSuggestion}
onReviewDraftChange={updateReviewDraft}
onSubmitQualityReview={submitQualityReview}
qualityReviews={qualityReviews}
reviewDraft={reviewDraft}
```

- [ ] **Step 9: Add CSS**

In `src/styles.css`, add:

```css
.review-layout {
  display: grid;
  gap: 16px;
}

.review-generator-form {
  display: grid;
  gap: 14px;
}

.review-score {
  display: inline-flex;
  width: fit-content;
  min-width: 72px;
  justify-content: center;
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 8px 10px;
  background: var(--surface-subtle);
  font-size: 1rem;
}

.review-check-list,
.review-suggestion-list {
  display: grid;
  gap: 10px;
}

.review-check,
.review-suggestion {
  display: grid;
  gap: 10px;
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 12px;
  background: var(--surface);
}

.review-check.failed {
  border-color: #c46a45;
}

.review-check.warning {
  border-color: #aa8b38;
}

.review-check div,
.review-suggestion-copy {
  display: grid;
  gap: 4px;
}

.review-suggestion {
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: start;
}

.review-suggestion p {
  margin: 0;
}

@media (max-width: 760px) {
  .review-suggestion {
    grid-template-columns: 1fr;
  }
}
```

If the project uses different custom properties, replace `var(--surface)` and `var(--surface-subtle)` with the nearest existing surface variables from `src/styles.css`.

- [ ] **Step 10: Run frontend test and verify PASS**

Run:

```powershell
npm test -- --run src/App.test.tsx -t "quality review"
```

Expected:

```text
PASS src/App.test.tsx
```

- [ ] **Step 11: Run full frontend tests and build**

Run:

```powershell
npm test
npm run build
```

Expected:

```text
Test Files  1 passed
vite build completed
```

- [ ] **Step 12: Commit Task 5**

Run:

```powershell
git add src/types/joi.ts src/api/joiApi.ts src/components/ReviewWorkspace.tsx src/components/BrandProjectRail.tsx src/components/ProjectWorkspace.tsx src/App.tsx src/styles.css src/App.test.tsx
git commit -m "feat: add quality review workspace"
```

### Task 6: Smoke, Review, Merge, Push

**Files:**

- Create: `docs/superpowers/reports/joi-0.19-quality-review-iteration-smoke-test.md`
- Modify only if needed from test failures.

- [ ] **Step 1: Run full backend tests**

Run:

```powershell
cargo test
```

Expected:

```text
test result: ok
```

Known acceptable warning if still present:

```text
warning: field `temp_dir` is never read
```

- [ ] **Step 2: Run full frontend tests and build**

Run:

```powershell
npm test
npm run build
```

Expected:

```text
PASS
vite build completed
```

- [ ] **Step 3: Start local app**

Run:

```powershell
npm run dev -- --host 127.0.0.1
```

Keep the dev server running until browser smoke is complete.

- [ ] **Step 4: Browser smoke on desktop**

Use the Browser plugin on the local Vite URL.

Smoke path:

1. Open the app.
2. Confirm left rail includes `Review` between `Prompts` and `Delivery`.
3. Select or create a project.
4. Open `Review`.
5. Confirm `Quality Review`, `Review Checklist`, and `Revision Suggestions` sections render.
6. Generate a review if local state has storyboard and prompts.
7. Confirm no horizontal overflow and no console errors.

- [ ] **Step 5: Browser smoke on mobile viewport**

Use a narrow viewport.

Smoke path:

1. Open `Review`.
2. Confirm score/checklist/suggestion content stays inside its containers.
3. Confirm suggestion action buttons wrap under text instead of overflowing.
4. Confirm no console errors.

- [ ] **Step 6: Write smoke report**

Create `docs/superpowers/reports/joi-0.19-quality-review-iteration-smoke-test.md`:

```markdown
# Joi Agent 0.19 Quality Review And Iteration Smoke Test

## Automated Verification

- `cargo test`: PASS
- `npm test`: PASS
- `npm run build`: PASS

## Browser Verification

Desktop:

- Review tab appears between Prompts and Delivery.
- Review workspace renders generation, checklist, and suggestion areas.
- No horizontal overflow.
- Console has no errors.

Mobile:

- Review workspace remains readable in a narrow viewport.
- Suggestion action button stacks without text overflow.
- Console has no errors.

## Acceptance Checklist

- Joi can identify storyboard repetition, duration mismatch, and insufficient garment/selling-point coverage.
- Joi can identify missing prompt fields.
- User can accept a supported suggestion and update the target shot or prompt package.
- Quality reviews are included in project snapshots.
- Review generation and suggestion application create Agent runs/events.

## Known Limits

- 0.19 review rules are deterministic and local.
- Duration issues are reported but not auto-balanced.
- Only shot description and prompt text suggestions are auto-applicable.
```

- [ ] **Step 7: Commit smoke report**

Run:

```powershell
git add docs/superpowers/reports/joi-0.19-quality-review-iteration-smoke-test.md
git commit -m "test: add Joi 0.19 quality review smoke report"
```

- [ ] **Step 8: Review branch log**

Run:

```powershell
git log --oneline --decorate -6
git status --short
```

Expected:

```text
git status --short
```

prints no tracked file changes.

- [ ] **Step 9: Merge to main**

From the main workspace:

```powershell
git switch main
git merge --no-ff codex/joi-0.19-quality-review
```

Expected:

```text
Merge made by the 'ort' strategy.
```

- [ ] **Step 10: Verify main after merge**

Run:

```powershell
npm test
npm run build
cargo test
```

Expected:

```text
npm test PASS
npm run build PASS
cargo test PASS
```

- [ ] **Step 11: Push main to GitHub**

Run:

```powershell
git push origin main
```

Expected:

```text
main -> main
```

- [ ] **Step 12: Clean 0.19 worktree**

After the push succeeds:

```powershell
git worktree remove .worktrees\joi-0.19-quality-review
git branch -d codex/joi-0.19-quality-review
```

Expected:

```text
Deleted branch codex/joi-0.19-quality-review
```

## 0.19 Acceptance Review

Before calling 0.19 complete, verify:

- `quality_reviews` table exists with project index.
- Repository can create, list, get, and update review suggestions.
- `generate_quality_review` creates checks for duration, repetition, garment visibility, prompt completeness, and prompt context when fixture data triggers them.
- `apply_quality_review_suggestion` updates supported shot and prompt targets.
- Unsupported suggestion targets return `JoiError::Validation`.
- Locked shots and locked prompt packages are not modified.
- `ProjectSnapshotService::build_snapshot` includes `quality_reviews`.
- Tauri command handlers are registered.
- Review tab renders between `Prompts` and `Delivery`.
- Browser smoke passes on desktop and mobile.
- 0.19 smoke report exists.
- Main branch is pushed to GitHub.

## Execution Policy For This Goal

This plan is part of the active Codex goal:

```text
以 0.2 roadmap 作为长期目标，按 0.11 到 0.20 的 0.01 小阶段推进 Joi Agent 开发；每个小阶段先编写详细实施文档，再开发、验收、合并到 main 并推送 GitHub。
```

Execution should continue automatically after this plan is saved:

1. Commit this plan on `main`.
2. Push `main`.
3. Create implementation branch/worktree `codex/joi-0.19-quality-review`.
4. Execute tasks with TDD.
5. Verify and smoke test.
6. Merge to `main`.
7. Push to GitHub.
8. Clean the 0.19 worktree.
9. Move to 0.20 planning only after 0.19 acceptance passes.

## Self-Review

- Spec coverage: The 0.19 roadmap requires storyboard review, prompt review, brand consistency review, duration consistency, shot repetition, garment visibility, platform prompt completeness, structured checklist, apply suggested revision, and snapshot inclusion. Tasks 1 through 5 map directly to those requirements, and Task 6 verifies them.
- Placeholder scan: This plan uses concrete table names, structs, function names, commands, expected failures, and expected pass criteria. It contains no unresolved placeholder markers, no deferred implementation marker, and no unspecified test step.
- Type consistency: `QualityReview`, `QualityReviewCheck`, `QualityReviewSuggestion`, `QualityReviewGenerationInput`, `QualityReviewGenerationResult`, `ApplyReviewSuggestionInput`, and `ApplyReviewSuggestionResult` are named consistently across Rust, TypeScript, commands, and tests.
