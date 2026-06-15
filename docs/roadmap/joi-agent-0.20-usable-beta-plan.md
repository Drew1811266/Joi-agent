# Joi Agent 0.20 Usable Beta Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the 0.11-0.19 feature set into a usable beta workflow that can take a real fashion ad project from saved project context to storyboard, prompt packages, quality review, delivery report, memory review, snapshot, and export readiness.

**Architecture:** 0.20 adds a beta workflow orchestration layer over the existing Joi aggregates instead of replacing the existing workspaces. A new backend service assesses project readiness, runs a controlled end-to-end beta pass for missing deterministic steps, records an overall Agent run, and exposes status/results through Tauri commands. The frontend adds a compact Beta Workflow panel on the Overview workspace so users can see missing steps, run the beta pass, and continue manual review/export from the existing tabs.

**Tech Stack:** Tauri 2 commands, Rust, rusqlite, serde/serde_json, React 19, TypeScript, Vitest, Testing Library, Joi repository services, existing 0.12 understanding, 0.14 research, 0.15 memory, 0.16 storyboard, 0.17 prompt adapter, 0.18 delivery, and 0.19 quality review modules.

---

## Product Outcome

After 0.20, a user can:

- Open a selected project and see whether it is beta-ready.
- See a concrete checklist for the full fashion ad workflow:
  - project setup
  - reference materials
  - product and creative understanding
  - research
  - storyboard
  - video prompt packages
  - image prompt packages
  - quality review
  - delivery report
  - accepted memory
  - snapshot and export readiness
- Run a controlled `Beta Workflow` pass that fills missing deterministic outputs when enough saved context exists.
- Keep manual authority over memory acceptance and delivery export path.
- Produce an end-to-end benchmark package from a real womenswear launch ad scenario without calling private test helpers or direct database commands.

0.20 does not add external model API execution, cloud sync, team permissions, native file picker UX, PDF/PPT export, or autonomous publishing.

## Scope

### In Scope

- Backend beta workflow status assessment.
- Backend beta workflow runner that orchestrates existing services.
- Tauri commands:
  - `joi_get_beta_workflow_status`
  - `joi_run_beta_workflow`
- Frontend API/types for beta workflow.
- Overview workspace Beta panel.
- App state refresh after beta run.
- End-to-end backend benchmark test.
- Frontend invoke test for the Beta panel.
- Browser smoke for layout and status rendering.
- Smoke report and 0.20 acceptance checklist.

### Out Of Scope

- No external generation model calls.
- No automatic acceptance of proposed memory.
- No automatic export without a user-provided export directory.
- No import/export schema rewrite.
- No large visual redesign.
- No new general-purpose Agent framework integration.

## Existing Code Context

Use these current files and patterns:

- `src-tauri/src/understanding.rs`
  - `generate_brief_understanding`
  - `BriefUnderstandingInput`
- `src-tauri/src/research.rs`
  - `generate_research_report`
  - `ResearchReportInput`
  - `ResearchSourceInput`
- `src-tauri/src/storyboard.rs`
  - `generate_storyboard`
  - `StoryboardGenerationInput`
- `src-tauri/src/prompt_adapter.rs`
  - `generate_prompt_packages`
  - `PromptGenerationInput`
  - adapter ids: `jimeng_video`, `grok_video`, `banana_2_image`, `jimeng_image`, `gpt_image_2`
- `src-tauri/src/quality_review.rs`
  - `generate_quality_review`
  - `QualityReviewGenerationInput`
- `src-tauri/src/delivery_report.rs`
  - `generate_delivery_report`
  - `preview_delivery_package`
  - `DeliveryReportGenerationInput`
- `src-tauri/src/memory_curation.rs`
  - `curate_memory_candidates`
  - `MemoryCurationInput`
- `src-tauri/src/snapshots.rs`
  - `ProjectSnapshotService::save_snapshot`
- `src-tauri/src/commands.rs`
  - public helper functions wrap repository/service calls.
- `src/components/ProjectWorkspace.tsx`
  - Overview tab currently contains Brand Setup, Project Setup, and Workflow Map.
- `src/App.tsx`
  - owns project state, refreshes all aggregate lists, and passes handlers to workspaces.

## Data Contract

### Beta Workflow Step

Rust and TypeScript must use the same serialized shape:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BetaWorkflowStep {
    pub id: String,
    pub title: String,
    pub status: String,
    pub source_count: usize,
    pub target_tab: String,
    pub action_label: String,
    pub message: String,
}
```

Allowed `status` values:

- `complete`
- `warning`
- `action_required`

### Beta Workflow Status Result

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BetaWorkflowStatusResult {
    pub project_id: String,
    pub ready: bool,
    pub score: i64,
    pub steps: Vec<BetaWorkflowStep>,
    pub next_action: String,
    pub warnings: Vec<String>,
}
```

Score rule:

```rust
let score = steps.iter().map(|step| match step.status.as_str() {
    "complete" => 10,
    "warning" => 6,
    _ => 0,
}).sum::<i64>();
```

`ready` is true only when these required steps are `complete`:

- `project_setup`
- `understanding`
- `storyboard`
- `video_prompts`
- `image_prompts`
- `quality_review`
- `delivery_report`
- `accepted_memory`
- `snapshot`

`reference_materials` and `research` may be `warning` for beta readiness because not every user starts with usable external sources, but the benchmark smoke must include at least one reference source.

### Beta Workflow Run Input

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BetaWorkflowRunInput {
    pub project_id: String,
    pub user_direction: String,
    pub image_brief: String,
    pub reference_sources: Vec<ResearchSourceInput>,
    pub memory_feedback: String,
    pub save_snapshot: bool,
}
```

### Beta Workflow Run Result

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetaWorkflowRunResult {
    pub status: BetaWorkflowStatusResult,
    pub generated_steps: Vec<String>,
    pub skipped_steps: Vec<String>,
    pub delivery_report_id: Option<String>,
    pub package_preview: Option<DeliveryPackagePreview>,
    pub snapshot_id: Option<String>,
    pub agent_run: AgentRun,
    pub agent_events: Vec<AgentRunEvent>,
}
```

## Beta Runner Behavior

The runner must be deterministic and conservative:

1. Read project, brand, assets, understandings, creative directions, research, storyboards, prompts, reviews, delivery reports, memory, and versions through the repository.
2. Generate product understanding only if none exists.
3. Generate research only if no research exists and `reference_sources` is non-empty.
4. Generate storyboard only if no storyboard with shots exists.
5. Generate video prompts only when a storyboard has shots and video prompts are missing.
6. Generate image prompts only when image prompts are missing. Use `image_brief` if provided; otherwise derive a default from project title, brand description, and latest product understanding.
7. Generate quality review only if none exists.
8. Generate memory candidates only if there is no accepted memory and either `memory_feedback` is non-empty or research reports exist. Do not auto-accept memory.
9. Generate delivery report only if none exists.
10. Save a snapshot only when `save_snapshot` is true.
11. Create one overarching Agent run with events summarizing generated and skipped steps.
12. Return a fresh status result after all actions.

Default understanding input may be derived only from saved project and brand fields:

```rust
BriefUnderstandingInput {
    project_id: project.id.clone(),
    brief_text: project.advertising_goal.clone(),
    product_name: project.title.clone(),
    category: "fashion collection".to_string(),
    audience: "short-form fashion ad viewers".to_string(),
    target_platforms: vec!["jimeng_video".to_string(), "grok_video".to_string()],
    selling_points_text: brand.description.clone(),
    visual_direction: if brand.description.trim().is_empty() {
        "Clean fashion advertising visuals with clear garment visibility.".to_string()
    } else {
        brand.description.clone()
    },
    constraints_text: "Keep garment shape, fabric texture, and brand styling consistent.".to_string(),
    reference_asset_ids: assets.iter().map(|asset| asset.id.clone()).collect(),
}
```

Default image brief:

```text
Full-body fashion model photo for <project title>, clean studio lighting, visible garment texture, brand-consistent styling.
```

Default memory feedback:

```text
Capture reusable brand and production preferences from the completed beta workflow.
```

## Implementation Tasks

### Task 1: Backend Beta Status Service

**Files:**

- Create: `src-tauri/src/beta_workflow.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/beta_workflow.rs`

- [ ] **Step 1: Write failing status test**

Create `src-tauri/tests/beta_workflow.rs` with a first test:

```rust
mod common;

use common::TestDb;
use joi_agent_lib::beta_workflow::assess_beta_workflow;
use joi_agent_lib::models::MemoryScope;
use joi_agent_lib::repositories::{
    MemoryEntryCreate, ProductUnderstandingCreate, Repository,
};
use serde_json::json;

#[test]
fn beta_status_reports_missing_and_complete_steps() {
    let db = TestDb::new();
    let repo = Repository::new(db.database.connection());
    let brand = repo
        .create_brand("Atelier Joi", "Premium womenswear with soft studio light.")
        .expect("brand");
    let project = repo
        .create_project(
            &brand.id,
            "Spring Outerwear Launch",
            "Launch a 15 second ad for the spring trench collection.",
            15,
        )
        .expect("project");

    let initial = assess_beta_workflow(&repo, &project.id).expect("initial status");
    assert!(!initial.ready);
    assert_eq!(initial.steps[0].id, "project_setup");
    assert_eq!(initial.steps[0].status, "complete");
    assert!(initial
        .steps
        .iter()
        .any(|step| step.id == "understanding" && step.status == "action_required"));

    repo.create_product_understanding(ProductUnderstandingCreate {
        project_id: project.id.clone(),
        product_name: "Spring trench collection".to_string(),
        category: "outerwear".to_string(),
        audience: "urban womenswear shoppers".to_string(),
        selling_points: vec!["water-resistant cotton".to_string()],
        constraints: vec!["keep fabric texture visible".to_string()],
        notes: "Generated beta fixture understanding.".to_string(),
    })
    .expect("understanding");
    repo.create_memory_entry(MemoryEntryCreate {
        scope: MemoryScope::Project.as_str().to_string(),
        brand_id: Some(brand.id.clone()),
        project_id: Some(project.id.clone()),
        content: "Always keep tactile fabric proof in the opening shot.".to_string(),
        source: "benchmark".to_string(),
        source_entity_type: Some("manual".to_string()),
        source_entity_id: None,
    })
    .expect("memory");

    let updated = assess_beta_workflow(&repo, &project.id).expect("updated status");
    assert!(updated.score > initial.score);
    assert!(updated
        .steps
        .iter()
        .any(|step| step.id == "accepted_memory" && step.status == "complete"));
}
```

- [ ] **Step 2: Run test and verify RED**

Run:

```powershell
cargo test --test beta_workflow beta_status_reports_missing_and_complete_steps
```

Expected: FAIL because `beta_workflow` module and `assess_beta_workflow` do not exist.

- [ ] **Step 3: Add module registration**

In `src-tauri/src/lib.rs`, add:

```rust
pub mod beta_workflow;
```

- [ ] **Step 4: Implement status types and assessment**

Create `src-tauri/src/beta_workflow.rs` with:

```rust
use serde::{Deserialize, Serialize};

use crate::delivery_report::DeliveryPackagePreview;
use crate::error::JoiResult;
use crate::models::{AgentRun, AgentRunEvent, ProjectVersion};
use crate::repositories::{Repository, StoryboardWithShots};
use crate::research::ResearchSourceInput;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BetaWorkflowStep {
    pub id: String,
    pub title: String,
    pub status: String,
    pub source_count: usize,
    pub target_tab: String,
    pub action_label: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BetaWorkflowStatusResult {
    pub project_id: String,
    pub ready: bool,
    pub score: i64,
    pub steps: Vec<BetaWorkflowStep>,
    pub next_action: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BetaWorkflowRunInput {
    pub project_id: String,
    pub user_direction: String,
    pub image_brief: String,
    pub reference_sources: Vec<ResearchSourceInput>,
    pub memory_feedback: String,
    pub save_snapshot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetaWorkflowRunResult {
    pub status: BetaWorkflowStatusResult,
    pub generated_steps: Vec<String>,
    pub skipped_steps: Vec<String>,
    pub delivery_report_id: Option<String>,
    pub package_preview: Option<DeliveryPackagePreview>,
    pub snapshot_id: Option<String>,
    pub agent_run: AgentRun,
    pub agent_events: Vec<AgentRunEvent>,
}

pub fn assess_beta_workflow(
    repo: &Repository<'_>,
    project_id: &str,
) -> JoiResult<BetaWorkflowStatusResult> {
    let project = repo.get_project(project_id)?;
    let brand = repo.get_brand(&project.brand_id)?;
    let assets = repo.list_assets(project_id)?;
    let understandings = repo.list_product_understandings(project_id)?;
    let directions = repo.list_creative_directions(project_id)?;
    let research = repo.list_research_reports(project_id)?;
    let storyboards = repo.list_storyboards_with_typed_shots(project_id)?;
    let prompts = repo.list_prompt_packages(project_id)?;
    let reviews = repo.list_quality_reviews(project_id)?;
    let delivery_reports = repo.list_delivery_reports(project_id)?;
    let memory = repo.list_memory_entries_for_project(project_id)?;
    let versions = repo.list_project_versions(project_id)?;

    let has_storyboard_shots = storyboards.iter().any(|item| !item.shots.is_empty());
    let video_prompt_count = prompts
        .iter()
        .filter(|package| package.modality == "video")
        .count();
    let image_prompt_count = prompts
        .iter()
        .filter(|package| package.modality == "image")
        .count();
    let accepted_memory_count = memory
        .iter()
        .filter(|entry| entry.status == "accepted")
        .count();

    let steps = vec![
        step(
            "project_setup",
            "Project setup",
            "Overview",
            "Edit project",
            1,
            !brand.name.trim().is_empty()
                && !project.title.trim().is_empty()
                && !project.advertising_goal.trim().is_empty()
                && project.duration_seconds > 0,
            "Brand and project context are saved.",
            "Brand, project title, goal, and duration are required.",
        ),
        optional_step(
            "reference_materials",
            "Reference materials",
            "Brief",
            "Add reference",
            assets.len(),
            assets.len() > 0,
            "Reference materials are available.",
            "Add at least one reference image, video, or link for the benchmark.",
        ),
        step(
            "understanding",
            "Product understanding",
            "Brief",
            "Generate understanding",
            understandings.len() + directions.len(),
            !understandings.is_empty() && !directions.is_empty(),
            "Product understanding and creative direction are saved.",
            "Generate product understanding and creative direction.",
        ),
        optional_step(
            "research",
            "Research report",
            "Research",
            "Generate research",
            research.len(),
            !research.is_empty(),
            "Research report is saved.",
            "Generate a source-backed research report.",
        ),
        step(
            "storyboard",
            "Storyboard",
            "Storyboard",
            "Generate storyboard",
            shot_count(&storyboards),
            has_storyboard_shots,
            "Storyboard shots are available.",
            "Generate a 15-30 second storyboard.",
        ),
        step(
            "video_prompts",
            "Video prompts",
            "Prompts",
            "Generate video prompts",
            video_prompt_count,
            video_prompt_count >= 2,
            "Jimeng and Grok video prompts are available.",
            "Generate Jimeng and Grok video prompts.",
        ),
        step(
            "image_prompts",
            "Image prompts",
            "Prompts",
            "Generate image prompts",
            image_prompt_count,
            image_prompt_count >= 3,
            "Banana 2, Jimeng Image, and GPT Image 2 prompts are available.",
            "Generate image prompt packages.",
        ),
        step(
            "quality_review",
            "Quality review",
            "Review",
            "Generate review",
            reviews.len(),
            !reviews.is_empty(),
            "Quality review is saved.",
            "Generate a quality review and apply selected suggestions.",
        ),
        step(
            "delivery_report",
            "Delivery report",
            "Delivery",
            "Generate report",
            delivery_reports.len(),
            !delivery_reports.is_empty(),
            "Delivery report is saved.",
            "Generate a delivery report.",
        ),
        step(
            "accepted_memory",
            "Accepted memory",
            "Memory",
            "Review memory",
            accepted_memory_count,
            accepted_memory_count > 0,
            "Accepted project memory is available.",
            "Accept at least one memory candidate or add project memory.",
        ),
        step(
            "snapshot",
            "Snapshot",
            "Versions",
            "Save snapshot",
            versions.len(),
            !versions.is_empty(),
            "At least one project snapshot is saved.",
            "Save a project snapshot.",
        ),
    ];

    let required_ids = [
        "project_setup",
        "understanding",
        "storyboard",
        "video_prompts",
        "image_prompts",
        "quality_review",
        "delivery_report",
        "accepted_memory",
        "snapshot",
    ];
    let ready = required_ids.iter().all(|id| {
        steps
            .iter()
            .any(|step| step.id == *id && step.status == "complete")
    });
    let score = steps
        .iter()
        .map(|step| match step.status.as_str() {
            "complete" => 10,
            "warning" => 6,
            _ => 0,
        })
        .sum::<i64>();
    let next_action = steps
        .iter()
        .find(|step| step.status == "action_required")
        .map(|step| step.action_label.clone())
        .unwrap_or_else(|| "Review beta package".to_string());
    let warnings = steps
        .iter()
        .filter(|step| step.status == "warning")
        .map(|step| step.message.clone())
        .collect();

    Ok(BetaWorkflowStatusResult {
        project_id: project_id.to_string(),
        ready,
        score,
        steps,
        next_action,
        warnings,
    })
}

fn step(
    id: &str,
    title: &str,
    target_tab: &str,
    action_label: &str,
    source_count: usize,
    complete: bool,
    complete_message: &str,
    missing_message: &str,
) -> BetaWorkflowStep {
    BetaWorkflowStep {
        id: id.to_string(),
        title: title.to_string(),
        status: if complete { "complete" } else { "action_required" }.to_string(),
        source_count,
        target_tab: target_tab.to_string(),
        action_label: action_label.to_string(),
        message: if complete { complete_message } else { missing_message }.to_string(),
    }
}

fn optional_step(
    id: &str,
    title: &str,
    target_tab: &str,
    action_label: &str,
    source_count: usize,
    complete: bool,
    complete_message: &str,
    missing_message: &str,
) -> BetaWorkflowStep {
    BetaWorkflowStep {
        id: id.to_string(),
        title: title.to_string(),
        status: if complete { "complete" } else { "warning" }.to_string(),
        source_count,
        target_tab: target_tab.to_string(),
        action_label: action_label.to_string(),
        message: if complete { complete_message } else { missing_message }.to_string(),
    }
}

fn shot_count(storyboards: &[StoryboardWithShots]) -> usize {
    storyboards.iter().map(|item| item.shots.len()).sum()
}
```

- [ ] **Step 5: Run status test and verify GREEN**

Run:

```powershell
cargo test --test beta_workflow beta_status_reports_missing_and_complete_steps
```

Expected: PASS.

- [ ] **Step 6: Commit Task 1**

Run:

```powershell
git add src-tauri/src/beta_workflow.rs src-tauri/src/lib.rs src-tauri/tests/beta_workflow.rs
git commit -m "feat: add beta workflow status assessment"
```

### Task 2: Backend Beta Workflow Runner And Commands

**Files:**

- Modify: `src-tauri/src/beta_workflow.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/beta_workflow.rs`
- Test: `src-tauri/tests/commands.rs`

- [ ] **Step 1: Write failing beta run test**

Append to `src-tauri/tests/beta_workflow.rs`:

```rust
use joi_agent_lib::beta_workflow::{run_beta_workflow, BetaWorkflowRunInput};
use joi_agent_lib::research::ResearchSourceInput;

#[test]
fn beta_run_generates_end_to_end_project_outputs() {
    let db = TestDb::new();
    let repo = Repository::new(db.database.connection());
    let brand = repo
        .create_brand("Atelier Joi", "Premium womenswear, tactile fabric proof, clean warm studio lighting.")
        .expect("brand");
    let project = repo
        .create_project(
            &brand.id,
            "Spring Outerwear Launch",
            "Create a 15 second launch ad for a spring trench collection.",
            15,
        )
        .expect("project");
    repo.create_memory_entry(MemoryEntryCreate {
        scope: MemoryScope::Project.as_str().to_string(),
        brand_id: Some(brand.id.clone()),
        project_id: Some(project.id.clone()),
        content: "Open with a tactile fabric proof before model movement.".to_string(),
        source: "benchmark".to_string(),
        source_entity_type: Some("manual".to_string()),
        source_entity_id: None,
    })
    .expect("memory");

    let result = run_beta_workflow(
        &repo,
        BetaWorkflowRunInput {
            project_id: project.id.clone(),
            user_direction: "Complete the beta benchmark with premium but practical fashion ad outputs.".to_string(),
            image_brief: "Full-body ecommerce model photo, warm clean studio, visible trench texture.".to_string(),
            reference_sources: vec![ResearchSourceInput {
                title: "Benchmark reference note".to_string(),
                url: "https://example.com/atelier-joi-reference".to_string(),
                source_type: "reference".to_string(),
                excerpt: "Texture close-ups and restrained studio movement support premium outerwear positioning.".to_string(),
            }],
            memory_feedback: "Keep tactile proof and restrained styling as reusable brand preferences.".to_string(),
            save_snapshot: true,
        },
        "0.20.0-test".to_string(),
    )
    .expect("beta run");

    assert!(result.generated_steps.contains(&"understanding".to_string()));
    assert!(result.generated_steps.contains(&"storyboard".to_string()));
    assert!(result.generated_steps.contains(&"video_prompts".to_string()));
    assert!(result.generated_steps.contains(&"image_prompts".to_string()));
    assert!(result.generated_steps.contains(&"quality_review".to_string()));
    assert!(result.generated_steps.contains(&"delivery_report".to_string()));
    assert!(result.snapshot_id.is_some());
    assert!(result.delivery_report_id.is_some());
    assert!(result.package_preview.is_some());
    assert!(result.status.ready);
}
```

- [ ] **Step 2: Run beta run test and verify RED**

Run:

```powershell
cargo test --test beta_workflow beta_run_generates_end_to_end_project_outputs
```

Expected: FAIL because `run_beta_workflow` is not implemented.

- [ ] **Step 3: Implement runner**

In `src-tauri/src/beta_workflow.rs`, add imports:

```rust
use serde_json::json;

use crate::delivery_report::{
    generate_delivery_report, preview_delivery_package, DeliveryReportGenerationInput,
};
use crate::memory_curation::{curate_memory_candidates, MemoryCurationInput};
use crate::prompt_adapter::{generate_prompt_packages, PromptGenerationInput};
use crate::quality_review::{generate_quality_review, QualityReviewGenerationInput};
use crate::repositories::{AgentRunCreate, AgentRunEventCreate};
use crate::snapshots::{ProjectSnapshotService, SaveSnapshotInput};
use crate::storyboard::{generate_storyboard, StoryboardGenerationInput};
use crate::understanding::{generate_brief_understanding, BriefUnderstandingInput};
use crate::research::{generate_research_report, ResearchReportInput};
```

Add the runner:

```rust
pub fn run_beta_workflow(
    repo: &Repository<'_>,
    input: BetaWorkflowRunInput,
    hermes_version: String,
) -> JoiResult<BetaWorkflowRunResult> {
    let project = repo.get_project(&input.project_id)?;
    let brand = repo.get_brand(&project.brand_id)?;
    let assets = repo.list_assets(&input.project_id)?;
    let mut generated_steps = Vec::new();
    let mut skipped_steps = Vec::new();

    if repo.list_product_understandings(&input.project_id)?.is_empty()
        || repo.list_creative_directions(&input.project_id)?.is_empty()
    {
        generate_brief_understanding(
            repo,
            BriefUnderstandingInput {
                project_id: input.project_id.clone(),
                brief_text: project.advertising_goal.clone(),
                product_name: project.title.clone(),
                category: "fashion collection".to_string(),
                audience: "short-form fashion ad viewers".to_string(),
                target_platforms: vec!["jimeng_video".to_string(), "grok_video".to_string()],
                selling_points_text: brand.description.clone(),
                visual_direction: default_visual_direction(&brand.description),
                constraints_text:
                    "Keep garment shape, fabric texture, and brand styling consistent.".to_string(),
                reference_asset_ids: assets.iter().map(|asset| asset.id.clone()).collect(),
            },
        )?;
        generated_steps.push("understanding".to_string());
    } else {
        skipped_steps.push("understanding".to_string());
    }

    if repo.list_research_reports(&input.project_id)?.is_empty()
        && !input.reference_sources.is_empty()
    {
        generate_research_report(
            repo,
            ResearchReportInput {
                project_id: input.project_id.clone(),
                research_goal: "Build source-backed fashion ad direction for beta workflow.".to_string(),
                market_focus: "fashion advertising".to_string(),
                platform_focus: vec!["jimeng_video".to_string(), "grok_video".to_string()],
                source_materials: input.reference_sources.clone(),
            },
            hermes_version.clone(),
        )?;
        generated_steps.push("research".to_string());
    } else {
        skipped_steps.push("research".to_string());
    }

    let storyboards = repo.list_storyboards_with_typed_shots(&input.project_id)?;
    if !storyboards.iter().any(|item| !item.shots.is_empty()) {
        generate_storyboard(
            repo,
            StoryboardGenerationInput {
                project_id: input.project_id.clone(),
                user_direction: input.user_direction.clone(),
                preferred_duration_seconds: Some(project.duration_seconds),
                preferred_shot_count: Some(5),
            },
            hermes_version.clone(),
        )?;
        generated_steps.push("storyboard".to_string());
    } else {
        skipped_steps.push("storyboard".to_string());
    }

    let current_storyboards = repo.list_storyboards_with_typed_shots(&input.project_id)?;
    let shot_ids = current_storyboards
        .last()
        .map(|item| item.shots.iter().map(|shot| shot.id.clone()).collect::<Vec<_>>())
        .unwrap_or_default();
    let prompt_packages = repo.list_prompt_packages(&input.project_id)?;
    if !shot_ids.is_empty()
        && prompt_packages
            .iter()
            .filter(|package| package.modality == "video")
            .count()
            < 2
    {
        generate_prompt_packages(
            repo,
            PromptGenerationInput {
                project_id: input.project_id.clone(),
                shot_ids: shot_ids.clone(),
                image_brief: String::new(),
                target_platforms: vec!["jimeng_video".to_string(), "grok_video".to_string()],
                user_direction: input.user_direction.clone(),
            },
            hermes_version.clone(),
        )?;
        generated_steps.push("video_prompts".to_string());
    } else {
        skipped_steps.push("video_prompts".to_string());
    }

    let prompt_packages = repo.list_prompt_packages(&input.project_id)?;
    if prompt_packages
        .iter()
        .filter(|package| package.modality == "image")
        .count()
        < 3
    {
        generate_prompt_packages(
            repo,
            PromptGenerationInput {
                project_id: input.project_id.clone(),
                shot_ids: Vec::new(),
                image_brief: default_image_brief(&project.title, &input.image_brief),
                target_platforms: vec![
                    "banana_2_image".to_string(),
                    "jimeng_image".to_string(),
                    "gpt_image_2".to_string(),
                ],
                user_direction: input.user_direction.clone(),
            },
            hermes_version.clone(),
        )?;
        generated_steps.push("image_prompts".to_string());
    } else {
        skipped_steps.push("image_prompts".to_string());
    }

    if repo.list_quality_reviews(&input.project_id)?.is_empty() {
        generate_quality_review(
            repo,
            QualityReviewGenerationInput {
                project_id: input.project_id.clone(),
                user_direction: "Review beta workflow outputs before delivery.".to_string(),
            },
            hermes_version.clone(),
        )?;
        generated_steps.push("quality_review".to_string());
    } else {
        skipped_steps.push("quality_review".to_string());
    }

    if repo
        .list_memory_entries_for_project(&input.project_id)?
        .iter()
        .all(|entry| entry.status != "accepted")
    {
        let feedback_text = if input.memory_feedback.trim().is_empty() {
            "Capture reusable brand and production preferences from the completed beta workflow."
                .to_string()
        } else {
            input.memory_feedback.clone()
        };
        curate_memory_candidates(
            repo,
            MemoryCurationInput {
                project_id: input.project_id.clone(),
                feedback_text,
                include_research_reports: true,
            },
            hermes_version.clone(),
        )?;
        generated_steps.push("memory_candidates".to_string());
    } else {
        skipped_steps.push("memory_candidates".to_string());
    }

    let mut delivery_report_id = None;
    let delivery_reports = repo.list_delivery_reports(&input.project_id)?;
    if delivery_reports.is_empty() {
        let report = generate_delivery_report(
            repo,
            DeliveryReportGenerationInput {
                project_id: input.project_id.clone(),
                user_direction: "Prepare beta delivery package summary.".to_string(),
            },
            hermes_version.clone(),
        )?;
        delivery_report_id = Some(report.report.id);
        generated_steps.push("delivery_report".to_string());
    } else {
        delivery_report_id = delivery_reports.last().map(|report| report.id.clone());
        skipped_steps.push("delivery_report".to_string());
    }

    let package_preview = Some(preview_delivery_package(
        repo,
        &input.project_id,
        delivery_report_id.as_deref(),
    )?);

    let snapshot_id = if input.save_snapshot {
        let snapshot = ProjectSnapshotService::new(repo.connection()).save_snapshot(SaveSnapshotInput {
            project_id: input.project_id.clone(),
            label: "0.20 beta workflow snapshot".to_string(),
            change_reason: "Saved after beta workflow run.".to_string(),
            changed_entities: generated_steps.clone(),
            created_by: "joi-beta-workflow".to_string(),
            is_final_candidate: true,
        })?;
        generated_steps.push("snapshot".to_string());
        Some(snapshot.id)
    } else {
        skipped_steps.push("snapshot".to_string());
        None
    };

    let agent_run = repo.create_agent_run(AgentRunCreate {
        project_id: input.project_id.clone(),
        user_goal: "Run Joi 0.20 usable beta workflow.".to_string(),
        status: "completed".to_string(),
        runtime_kind: "hermes_core".to_string(),
        runtime_mode: "local_beta_workflow_bridge".to_string(),
        runtime_version: hermes_version,
        roles_json: json!(["planner", "storyboard_writer", "prompt_adapter", "reviewer", "memory_curator"]),
        plan_json: json!({
            "generated_steps": generated_steps,
            "skipped_steps": skipped_steps,
            "save_snapshot": input.save_snapshot
        }),
        result_summary: format!(
            "Completed beta workflow with {} generated step(s) and {} skipped step(s).",
            generated_steps.len(),
            skipped_steps.len()
        ),
    })?;
    let agent_events = create_beta_events(repo, &agent_run.id, &generated_steps, &skipped_steps)?;
    let status = assess_beta_workflow(repo, &input.project_id)?;

    Ok(BetaWorkflowRunResult {
        status,
        generated_steps,
        skipped_steps,
        delivery_report_id,
        package_preview,
        snapshot_id,
        agent_run,
        agent_events,
    })
}
```

Add helpers in the same file:

```rust
fn default_visual_direction(brand_description: &str) -> String {
    if brand_description.trim().is_empty() {
        "Clean fashion advertising visuals with clear garment visibility.".to_string()
    } else {
        brand_description.trim().to_string()
    }
}

fn default_image_brief(project_title: &str, image_brief: &str) -> String {
    if image_brief.trim().is_empty() {
        format!(
            "Full-body fashion model photo for {}, clean studio lighting, visible garment texture, brand-consistent styling.",
            project_title
        )
    } else {
        image_brief.trim().to_string()
    }
}

fn create_beta_events(
    repo: &Repository<'_>,
    agent_run_id: &str,
    generated_steps: &[String],
    skipped_steps: &[String],
) -> JoiResult<Vec<AgentRunEvent>> {
    let events = [
        (
            1,
            "planner",
            "beta_context_assessed",
            "Assessed project readiness for beta workflow.",
            json!({ "generated_step_count": generated_steps.len(), "skipped_step_count": skipped_steps.len() }),
        ),
        (
            2,
            "planner",
            "beta_steps_generated",
            "Generated missing beta workflow outputs.",
            json!({ "generated_steps": generated_steps }),
        ),
        (
            3,
            "reviewer",
            "beta_steps_skipped",
            "Skipped outputs that were already present or needed manual input.",
            json!({ "skipped_steps": skipped_steps }),
        ),
    ];

    events
        .into_iter()
        .map(|(sequence_number, role, event_type, message, payload_json)| {
            repo.create_agent_run_event(AgentRunEventCreate {
                agent_run_id: agent_run_id.to_string(),
                sequence_number,
                role: role.to_string(),
                event_type: event_type.to_string(),
                message: message.to_string(),
                payload_json,
            })
        })
        .collect()
}
```

If `Repository::connection()` is private or unavailable, add a small public accessor:

```rust
pub fn connection(&self) -> &Connection {
    self.connection
}
```

- [ ] **Step 4: Add command wrappers**

In `src-tauri/src/commands.rs`, import:

```rust
use crate::beta_workflow::{
    assess_beta_workflow, run_beta_workflow, BetaWorkflowRunInput, BetaWorkflowRunResult,
    BetaWorkflowStatusResult,
};
```

Add Tauri commands:

```rust
#[tauri::command(rename_all = "snake_case")]
pub fn joi_get_beta_workflow_status(
    state: State<'_, AppState>,
    project_id: String,
) -> JoiResult<BetaWorkflowStatusResult> {
    get_beta_workflow_status(state.inner(), project_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_run_beta_workflow(
    state: State<'_, AppState>,
    input: BetaWorkflowRunInput,
) -> JoiResult<BetaWorkflowRunResult> {
    run_beta_workflow_command(state.inner(), input)
}
```

Add helper functions near the other public helpers:

```rust
pub fn get_beta_workflow_status(
    state: &AppState,
    project_id: String,
) -> JoiResult<BetaWorkflowStatusResult> {
    let db = state.db.lock().expect("database mutex poisoned");
    assess_beta_workflow(&Repository::new(db.connection()), &project_id)
}

pub fn run_beta_workflow_command(
    state: &AppState,
    input: BetaWorkflowRunInput,
) -> JoiResult<BetaWorkflowRunResult> {
    let db = state.db.lock().expect("database mutex poisoned");
    run_beta_workflow(
        &Repository::new(db.connection()),
        input,
        crate::hermes_bridge::hermes_version(),
    )
}
```

In `src-tauri/src/lib.rs`, register:

```rust
commands::joi_get_beta_workflow_status,
commands::joi_run_beta_workflow,
```

- [ ] **Step 5: Add command test**

In `src-tauri/tests/commands.rs`, import:

```rust
use joi_agent_lib::beta_workflow::BetaWorkflowRunInput;
use joi_agent_lib::commands::{get_beta_workflow_status, run_beta_workflow_command};
```

Add a test:

```rust
#[test]
fn beta_workflow_commands_report_status_and_run() {
    let (_app, state) = test_state();
    let brand = create_brand(
        &state,
        BrandInput {
            name: "Atelier Joi".to_string(),
            description: "Premium womenswear, tactile textures, warm studio.".to_string(),
        },
    )
    .expect("brand");
    let project = create_project(
        &state,
        ProjectInput {
            brand_id: brand.id.clone(),
            title: "Spring Outerwear Launch".to_string(),
            advertising_goal: "Create a 15 second launch ad.".to_string(),
            duration_seconds: 15,
        },
    )
    .expect("project");
    create_memory_entry(
        &state,
        MemoryEntryInput {
            scope: "project".to_string(),
            brand_id: Some(brand.id),
            project_id: Some(project.id.clone()),
            content: "Open with tactile proof.".to_string(),
            source: "benchmark".to_string(),
        },
    )
    .expect("memory");

    let before = get_beta_workflow_status(&state, project.id.clone()).expect("before status");
    assert!(!before.ready);

    let result = run_beta_workflow_command(
        &state,
        BetaWorkflowRunInput {
            project_id: project.id.clone(),
            user_direction: "Complete the beta workflow.".to_string(),
            image_brief: "Full-body model photo, warm studio, visible trench texture.".to_string(),
            reference_sources: vec![ResearchSourceInput {
                title: "Reference note".to_string(),
                url: "https://example.com/reference".to_string(),
                source_type: "reference".to_string(),
                excerpt: "Texture close-ups support premium positioning.".to_string(),
            }],
            memory_feedback: "Keep tactile proof.".to_string(),
            save_snapshot: true,
        },
    )
    .expect("beta run");

    assert!(result.status.ready);
    assert!(result.delivery_report_id.is_some());
    assert!(result.snapshot_id.is_some());
}
```

- [ ] **Step 6: Run backend tests**

Run:

```powershell
cargo test --test beta_workflow
cargo test --test commands beta_workflow_commands_report_status_and_run
cargo test
```

Expected: PASS. Existing `TestApp.temp_dir` warning is acceptable.

- [ ] **Step 7: Commit Task 2**

Run:

```powershell
git add src-tauri/src/beta_workflow.rs src-tauri/src/commands.rs src-tauri/src/lib.rs src-tauri/tests/beta_workflow.rs src-tauri/tests/commands.rs
git commit -m "feat: run beta workflow orchestration"
```

### Task 3: Frontend Beta Workflow Panel

**Files:**

- Modify: `src/types/joi.ts`
- Modify: `src/api/joiApi.ts`
- Create: `src/components/BetaWorkflowPanel.tsx`
- Modify: `src/components/ProjectWorkspace.tsx`
- Modify: `src/App.tsx`
- Modify: `src/styles.css`
- Test: `src/App.test.tsx`

- [ ] **Step 1: Write failing frontend test**

In `src/App.test.tsx`, add a mock response for:

```ts
case "joi_get_beta_workflow_status":
  return Promise.resolve(mockBetaWorkflowStatus());
case "joi_run_beta_workflow":
  return Promise.resolve(mockBetaWorkflowRunResult());
```

Add helpers:

```ts
function mockBetaWorkflowStatus() {
  return {
    project_id: "project-1",
    ready: false,
    score: 62,
    next_action: "Generate storyboard",
    warnings: ["Generate a source-backed research report."],
    steps: [
      {
        id: "project_setup",
        title: "Project setup",
        status: "complete",
        source_count: 1,
        target_tab: "Overview",
        action_label: "Edit project",
        message: "Brand and project context are saved.",
      },
      {
        id: "storyboard",
        title: "Storyboard",
        status: "action_required",
        source_count: 0,
        target_tab: "Storyboard",
        action_label: "Generate storyboard",
        message: "Generate a 15-30 second storyboard.",
      },
    ],
  };
}

function mockBetaWorkflowRunResult() {
  return {
    status: {
      ...mockBetaWorkflowStatus(),
      ready: true,
      score: 96,
      next_action: "Review beta package",
      steps: mockBetaWorkflowStatus().steps.map((step) => ({ ...step, status: "complete" })),
      warnings: [],
    },
    generated_steps: ["storyboard", "video_prompts", "image_prompts", "quality_review", "delivery_report", "snapshot"],
    skipped_steps: ["research"],
    delivery_report_id: "delivery-report-1",
    package_preview: mockDeliveryReportResult().package_preview,
    snapshot_id: "version-beta",
    agent_run: {
      id: "run-beta",
      project_id: "project-1",
      user_goal: "Run Joi 0.20 usable beta workflow.",
      status: "completed",
      runtime_kind: "hermes_core",
      runtime_mode: "local_beta_workflow_bridge",
      runtime_version: "0.20.0",
      roles_json: ["planner"],
      plan_json: [],
      result_summary: "Completed beta workflow.",
      created_at: "2026-06-15T00:00:00Z",
      updated_at: "2026-06-15T00:00:00Z",
    },
    agent_events: [],
  };
}
```

Add test:

```ts
test("shows and runs the beta workflow from overview", async () => {
  render(<App />);

  await screen.findByRole("heading", { name: "Spring Drop Film" });
  expect(await screen.findByRole("heading", { name: "Beta Workflow" })).toBeInTheDocument();
  expect(screen.getByText("Generate a 15-30 second storyboard.")).toBeInTheDocument();

  fireEvent.change(screen.getByLabelText("Beta direction"), {
    target: { value: "Complete the beta benchmark." },
  });
  fireEvent.change(screen.getByLabelText("Beta image brief"), {
    target: { value: "Full-body model photo, warm studio." },
  });
  fireEvent.click(screen.getByRole("button", { name: /run beta workflow/i }));

  await waitFor(() => {
    expect(invokeMock).toHaveBeenCalledWith("joi_run_beta_workflow", {
      input: expect.objectContaining({
        project_id: "project-1",
        user_direction: "Complete the beta benchmark.",
        image_brief: "Full-body model photo, warm studio.",
        save_snapshot: true,
      }),
    });
  });
  expect(await screen.findByText("Beta ready")).toBeInTheDocument();
});
```

- [ ] **Step 2: Run frontend test and verify RED**

Run:

```powershell
npm test -- --run src/App.test.tsx -t "beta workflow"
```

Expected: FAIL because beta workflow types/API/UI do not exist.

- [ ] **Step 3: Add frontend types**

In `src/types/joi.ts`, add:

```ts
export type BetaWorkflowStep = {
  id: string;
  title: string;
  status: "complete" | "warning" | "action_required";
  source_count: number;
  target_tab: string;
  action_label: string;
  message: string;
};

export type BetaWorkflowStatusResult = {
  project_id: string;
  ready: boolean;
  score: number;
  steps: BetaWorkflowStep[];
  next_action: string;
  warnings: string[];
};

export type BetaWorkflowReferenceSource = {
  title: string;
  url: string;
  source_type: string;
  excerpt: string;
};

export type BetaWorkflowRunInput = {
  project_id: string;
  user_direction: string;
  image_brief: string;
  reference_sources: BetaWorkflowReferenceSource[];
  memory_feedback: string;
  save_snapshot: boolean;
};

export type BetaWorkflowRunResult = {
  status: BetaWorkflowStatusResult;
  generated_steps: string[];
  skipped_steps: string[];
  delivery_report_id: string | null;
  package_preview: DeliveryPackagePreview | null;
  snapshot_id: string | null;
  agent_run: AgentRun;
  agent_events: AgentRunEvent[];
};
```

- [ ] **Step 4: Add API wrappers**

In `src/api/joiApi.ts`, import the types and add:

```ts
export function getBetaWorkflowStatus(projectId: string): Promise<BetaWorkflowStatusResult> {
  return invoke<BetaWorkflowStatusResult>("joi_get_beta_workflow_status", { project_id: projectId });
}

export function runBetaWorkflow(input: BetaWorkflowRunInput): Promise<BetaWorkflowRunResult> {
  return invoke<BetaWorkflowRunResult>("joi_run_beta_workflow", { input });
}
```

- [ ] **Step 5: Create BetaWorkflowPanel**

Create `src/components/BetaWorkflowPanel.tsx`:

```tsx
import type { FormEvent } from "react";

import type { BetaWorkflowStatusResult, Project } from "../types/joi";

export type BetaWorkflowDraft = {
  user_direction: string;
  image_brief: string;
  reference_title: string;
  reference_url: string;
  reference_excerpt: string;
  memory_feedback: string;
  save_snapshot: boolean;
};

type BetaWorkflowPanelProps = {
  betaDraft: BetaWorkflowDraft;
  betaStatus: BetaWorkflowStatusResult | null;
  onBetaDraftChange: (field: keyof BetaWorkflowDraft, value: string | boolean) => void;
  onRunBetaWorkflow: () => void;
  runningBetaWorkflow: boolean;
  selectedProject: Project | null;
};

export function BetaWorkflowPanel({
  betaDraft,
  betaStatus,
  onBetaDraftChange,
  onRunBetaWorkflow,
  runningBetaWorkflow,
  selectedProject,
}: BetaWorkflowPanelProps) {
  return (
    <section className="workspace-panel wide">
      <div className="section-heading">
        <h2>Beta Workflow</h2>
        <span className={betaStatus?.ready ? "status-pill complete" : "status-pill"}>
          {betaStatus?.ready ? "Beta ready" : `${betaStatus?.score ?? 0}/110`}
        </span>
      </div>
      <form className="beta-workflow-form" onSubmit={submit(onRunBetaWorkflow)}>
        <label>
          Beta direction
          <textarea
            disabled={!selectedProject || runningBetaWorkflow}
            onChange={(event) => onBetaDraftChange("user_direction", event.target.value)}
            rows={3}
            value={betaDraft.user_direction}
          />
        </label>
        <label>
          Beta image brief
          <textarea
            disabled={!selectedProject || runningBetaWorkflow}
            onChange={(event) => onBetaDraftChange("image_brief", event.target.value)}
            rows={3}
            value={betaDraft.image_brief}
          />
        </label>
        <label>
          Reference title
          <input
            disabled={!selectedProject || runningBetaWorkflow}
            onChange={(event) => onBetaDraftChange("reference_title", event.target.value)}
            value={betaDraft.reference_title}
          />
        </label>
        <label>
          Reference URL
          <input
            disabled={!selectedProject || runningBetaWorkflow}
            onChange={(event) => onBetaDraftChange("reference_url", event.target.value)}
            value={betaDraft.reference_url}
          />
        </label>
        <label className="wide-field">
          Reference excerpt
          <textarea
            disabled={!selectedProject || runningBetaWorkflow}
            onChange={(event) => onBetaDraftChange("reference_excerpt", event.target.value)}
            rows={3}
            value={betaDraft.reference_excerpt}
          />
        </label>
        <label className="wide-field">
          Memory feedback
          <textarea
            disabled={!selectedProject || runningBetaWorkflow}
            onChange={(event) => onBetaDraftChange("memory_feedback", event.target.value)}
            rows={2}
            value={betaDraft.memory_feedback}
          />
        </label>
        <label className="checkbox-row">
          <input
            checked={betaDraft.save_snapshot}
            disabled={!selectedProject || runningBetaWorkflow}
            onChange={(event) => onBetaDraftChange("save_snapshot", event.target.checked)}
            type="checkbox"
          />
          Save beta snapshot
        </label>
        <button disabled={!selectedProject || runningBetaWorkflow} type="submit">
          {runningBetaWorkflow ? "Running" : "Run Beta Workflow"}
        </button>
      </form>
      {betaStatus ? (
        <div className="beta-step-list">
          {betaStatus.steps.map((step) => (
            <article className={`beta-step ${step.status}`} key={step.id}>
              <div>
                <strong>{step.title}</strong>
                <span>{step.message}</span>
              </div>
              <small>{step.source_count} source(s) · {step.target_tab}</small>
            </article>
          ))}
        </div>
      ) : (
        <p className="muted">Select a project to inspect beta readiness.</p>
      )}
    </section>
  );
}

function submit(handler: () => void) {
  return (event: FormEvent) => {
    event.preventDefault();
    handler();
  };
}
```

- [ ] **Step 6: Wire ProjectWorkspace**

In `src/components/ProjectWorkspace.tsx`, import:

```ts
import { BetaWorkflowPanel, type BetaWorkflowDraft } from "./BetaWorkflowPanel";
```

Add props:

```ts
betaDraft: BetaWorkflowDraft;
betaStatus: BetaWorkflowStatusResult | null;
onBetaDraftChange: (field: keyof BetaWorkflowDraft, value: string | boolean) => void;
onRunBetaWorkflow: () => void;
runningBetaWorkflow: boolean;
```

Add `BetaWorkflowStatusResult` to the type imports from `../types/joi`.

Render the panel in the Overview grid after Workflow Map:

```tsx
<BetaWorkflowPanel
  betaDraft={betaDraft}
  betaStatus={betaStatus}
  onBetaDraftChange={onBetaDraftChange}
  onRunBetaWorkflow={onRunBetaWorkflow}
  runningBetaWorkflow={runningBetaWorkflow}
  selectedProject={selectedProject}
/>
```

- [ ] **Step 7: Wire App state and handler**

In `src/App.tsx`, import:

```ts
getBetaWorkflowStatus,
runBetaWorkflow,
```

Import type:

```ts
import type { BetaWorkflowDraft } from "./components/BetaWorkflowPanel";
```

Add type import:

```ts
BetaWorkflowStatusResult,
```

Add defaults:

```ts
const emptyBetaDraft: BetaWorkflowDraft = {
  user_direction: "Complete a usable beta pass for this fashion ad project.",
  image_brief: "Full-body fashion model photo, clean warm studio lighting, visible garment texture.",
  reference_title: "Benchmark reference note",
  reference_url: "https://example.com/fashion-reference",
  reference_excerpt: "Texture close-ups and restrained studio motion support premium fashion positioning.",
  memory_feedback: "Keep tactile garment proof and brand-consistent styling as reusable project preferences.",
  save_snapshot: true,
};
```

Add state:

```ts
const [betaDraft, setBetaDraft] = useState<BetaWorkflowDraft>(emptyBetaDraft);
const [betaStatus, setBetaStatus] = useState<BetaWorkflowStatusResult | null>(null);
const [runningBetaWorkflow, setRunningBetaWorkflow] = useState(false);
```

Reset `betaStatus` when selected project is cleared.

In `refreshProjectState`, after loading the aggregate lists, also call `getBetaWorkflowStatus(projectId)` and set `betaStatus`.

Add:

```ts
function updateBetaDraft(field: keyof BetaWorkflowDraft, value: string | boolean) {
  setBetaDraft((draft) => ({ ...draft, [field]: value }));
}

async function submitBetaWorkflow() {
  if (!selectedProject) {
    setError("Select a project before running the beta workflow.");
    return;
  }
  try {
    setRunningBetaWorkflow(true);
    setError(null);
    const referenceSources =
      betaDraft.reference_title.trim() && betaDraft.reference_excerpt.trim()
        ? [
            {
              title: betaDraft.reference_title,
              url: betaDraft.reference_url || "https://example.com/fashion-reference",
              source_type: "reference",
              excerpt: betaDraft.reference_excerpt,
            },
          ]
        : [];
    const result = await runBetaWorkflow({
      project_id: selectedProject.id,
      user_direction: betaDraft.user_direction,
      image_brief: betaDraft.image_brief,
      reference_sources: referenceSources,
      memory_feedback: betaDraft.memory_feedback,
      save_snapshot: betaDraft.save_snapshot,
    });
    setBetaStatus(result.status);
    setPackagePreview(result.package_preview);
    await refreshProjectState(selectedProject.id);
    setAgentRuns((runs) => [
      { run: result.agent_run, events: result.agent_events },
      ...runs.filter((item) => item.run.id !== result.agent_run.id),
    ]);
    setActivityLog((entries) => [
      ...entries,
      `Beta workflow generated ${result.generated_steps.length} step(s).`,
    ]);
  } catch (submitError) {
    setError(formatError(submitError));
  } finally {
    setRunningBetaWorkflow(false);
  }
}
```

Pass props to `ProjectWorkspace`:

```tsx
betaDraft={betaDraft}
betaStatus={betaStatus}
onBetaDraftChange={updateBetaDraft}
onRunBetaWorkflow={submitBetaWorkflow}
runningBetaWorkflow={runningBetaWorkflow}
```

- [ ] **Step 8: Add CSS**

In `src/styles.css`, add:

```css
.status-pill {
  display: inline-flex;
  min-width: 78px;
  justify-content: center;
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 6px 8px;
  color: var(--muted);
  font-size: 0.82rem;
}

.status-pill.complete {
  color: #235b3a;
  border-color: #75a486;
  background: #edf7f0;
}

.beta-workflow-form {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px;
}

.beta-workflow-form button {
  justify-self: start;
}

.beta-step-list {
  display: grid;
  gap: 10px;
  margin-top: 16px;
}

.beta-step {
  display: grid;
  gap: 8px;
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 12px;
  background: var(--surface);
}

.beta-step.complete {
  border-color: #75a486;
}

.beta-step.warning {
  border-color: #aa8b38;
}

.beta-step.action_required {
  border-color: #c46a45;
}

.beta-step div {
  display: grid;
  gap: 4px;
}

@media (max-width: 760px) {
  .beta-workflow-form {
    grid-template-columns: 1fr;
  }
}
```

If `var(--surface)` is not present, use the closest existing surface token in `src/styles.css`.

- [ ] **Step 9: Run frontend tests**

Run:

```powershell
npm test -- --run src/App.test.tsx -t "beta workflow"
npm test
npm run build
```

Expected: PASS.

- [ ] **Step 10: Commit Task 3**

Run:

```powershell
git add src/types/joi.ts src/api/joiApi.ts src/components/BetaWorkflowPanel.tsx src/components/ProjectWorkspace.tsx src/App.tsx src/styles.css src/App.test.tsx
git commit -m "feat: add beta workflow panel"
```

### Task 4: 0.20 Benchmark Smoke And Report

**Files:**

- Create: `docs/superpowers/reports/joi-0.20-usable-beta-smoke-test.md`
- Modify only if verification exposes defects.

- [ ] **Step 1: Run full automated verification**

Run:

```powershell
npm test
npm run build
cargo test
```

Expected: PASS. Existing `TestApp.temp_dir` warning remains acceptable.

- [ ] **Step 2: Start local app**

Run:

```powershell
npm run dev -- --host 127.0.0.1 --port 55306
```

Keep the server running until browser smoke is complete.

- [ ] **Step 3: Browser smoke desktop**

Use the Browser plugin:

1. Open `http://127.0.0.1:55306/`.
2. Confirm Overview includes `Beta Workflow`.
3. Confirm status steps render.
4. Confirm `Run Beta Workflow` control is visible.
5. Confirm no horizontal overflow at desktop width.
6. Confirm browser console has no errors.

In a normal browser, Tauri IPC is unavailable. The smoke must verify the readable backend fallback message remains clear and does not show a raw `invoke` exception.

- [ ] **Step 4: Browser smoke mobile**

Use viewport `390 x 844`:

1. Reload the app.
2. Confirm `Beta Workflow` remains visible.
3. Confirm beta form fields stack to one column.
4. Confirm status cards do not overflow.
5. Confirm browser console has no errors.

- [ ] **Step 5: Native command smoke through tests**

The true beta run is covered by:

```powershell
cargo test --test beta_workflow beta_run_generates_end_to_end_project_outputs
cargo test --test commands beta_workflow_commands_report_status_and_run
```

These tests must prove a benchmark project can produce:

- product understanding
- creative direction
- research report from source input
- storyboard
- Jimeng and Grok video prompts
- Banana 2, Jimeng Image, and GPT Image 2 image prompts
- quality review
- delivery report
- package preview
- accepted memory participation
- snapshot

- [ ] **Step 6: Write smoke report**

Create `docs/superpowers/reports/joi-0.20-usable-beta-smoke-test.md`:

```markdown
# Joi 0.20 Usable Beta Smoke Test

Date: 2026-06-15

Branch: `codex/joi-0.20-usable-beta`

## Automated Verification

- `npm test`: PASS
- `npm run build`: PASS
- `cargo test`: PASS
- `cargo test --test beta_workflow beta_run_generates_end_to_end_project_outputs`: PASS
- `cargo test --test commands beta_workflow_commands_report_status_and_run`: PASS

Known warning:

- Rust tests still warn that `TestApp.temp_dir` is never read. This is existing test fixture ownership behavior and not a 0.20 regression.

## Browser Verification

Desktop:

- Overview renders `Beta Workflow`.
- Beta readiness status cards render.
- Beta form controls are visible.
- No horizontal overflow.
- Console has no errors.
- Browser preview shows the readable backend unavailable fallback instead of a raw `invoke` exception.

Mobile:

- `Beta Workflow` remains visible at `390 x 844`.
- Form fields stack to one column.
- Status cards remain inside the viewport.
- Console has no errors.

## Benchmark Coverage

- Brand: contemporary womenswear label.
- Product: spring outerwear collection.
- Goal: 15 second short-video launch ad.
- Reference source: source-backed benchmark note.
- Output: creative direction, storyboard, image prompts, video prompts, quality review, delivery report, package preview, accepted memory participation, and snapshot.

## Result

0.20 usable beta workflow passed automated and browser smoke verification.
```

- [ ] **Step 7: Commit smoke report**

Run:

```powershell
git add docs/superpowers/reports/joi-0.20-usable-beta-smoke-test.md
git commit -m "test: add Joi 0.20 usable beta smoke report"
```

### Task 5: Merge, Push, And Goal Completion Review

**Files:**

- No code changes expected.

- [ ] **Step 1: Review branch state**

Run:

```powershell
git status --short --branch
git log --oneline --decorate -8
```

Expected: no uncommitted tracked changes.

- [ ] **Step 2: Merge to main**

From main workspace:

```powershell
git status --short --branch
git merge --no-ff codex/joi-0.20-usable-beta -m "merge: Joi 0.20 usable beta"
```

Expected:

```text
Merge made by the 'ort' strategy.
```

- [ ] **Step 3: Verify main after merge**

Run:

```powershell
npm test
npm run build
cargo test
```

Expected: PASS.

- [ ] **Step 4: Push main**

Run:

```powershell
git push origin main
```

Expected: `main -> main`.

- [ ] **Step 5: Clean 0.20 worktree**

Run:

```powershell
git worktree remove .worktrees\joi-0.20-usable-beta
git branch -d codex/joi-0.20-usable-beta
```

Expected: local 0.20 worktree and branch are removed.

- [ ] **Step 6: 0.2 goal acceptance review**

Verify:

- 0.11 through 0.20 implementation plans exist.
- 0.11 through 0.20 smoke reports exist.
- `main` is pushed to GitHub.
- README still describes the product accurately.
- 0.20 benchmark can complete from saved project context through delivery readiness.
- The active Codex goal can be marked complete only after all above checks pass.

## Execution Policy For This Goal

This plan is part of the active Codex goal:

```text
以 0.2 roadmap 作为长期目标，按 0.11 到 0.20 的 0.01 小阶段推进 Joi Agent 开发；每个小阶段先编写详细实施文档，再开发、验收、合并到 main 并推送 GitHub。
```

Execution should continue automatically after this plan is saved:

1. Commit this plan on `main`.
2. Push `main`.
3. Create implementation branch/worktree `codex/joi-0.20-usable-beta`.
4. Execute tasks with TDD.
5. Verify and smoke test.
6. Merge to `main`.
7. Push to GitHub.
8. Clean the 0.20 worktree.
9. Mark the Codex goal complete only after 0.20 acceptance passes.

## Self-Review

- Spec coverage: The 0.20 roadmap requires a usable beta closed loop from project setup through understanding, storyboard, prompts, review, delivery report, memory, snapshot, and export readiness. Tasks 1 through 4 implement and verify that loop, and Task 5 checks the 0.2 goal before completion.
- Placeholder scan: This plan includes concrete file paths, command names, type contracts, test names, step ids, frontend labels, and verification commands. It avoids unresolved markers and deferred implementation language.
- Type consistency: `BetaWorkflowStep`, `BetaWorkflowStatusResult`, `BetaWorkflowRunInput`, and `BetaWorkflowRunResult` are named consistently across Rust, TypeScript, commands, tests, and UI wiring.
