# Joi Agent 0.16 Storyboard Generation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Joi generate, save, edit, and selectively regenerate practical 15 to 30 second fashion advertising storyboards from project context.

**Architecture:** 0.16 builds on the existing `storyboards` and `shots` tables instead of adding a migration. Existing shot columns store the core visible fields; `metadata_json` stores 0.16-specific fields such as `garment_focus`, `transition`, source memory ids, and source research report ids. The implementation adds typed repository helpers, a deterministic `storyboard` service, Tauri commands, and a Storyboard workspace in React.

**Tech Stack:** Tauri 2 commands, Rust, rusqlite, serde/serde_json, chrono, React 19, TypeScript, Vitest, Joi 0.13 Agent run model, Joi 0.14 research reports, Joi 0.15 accepted memory.

---

## Product Outcome

After 0.16, a user can:

- Open the Storyboard tab for a project.
- Generate a complete 15 to 30 second fashion ad storyboard from current project context.
- See total duration, shot count, and per-shot duration.
- See each shot with visual description, model action, garment focus, camera movement, scene, lighting, transition, text suggestion, and rationale.
- Edit a shot and save the edited details.
- Regenerate one selected shot while preserving the rest of the storyboard.
- See storyboard generation and shot regeneration recorded as Agent runs with ordered events.
- Save snapshots that include storyboards and shots through the existing snapshot path.

0.16 does not generate video files, prompt packages, or a timeline editor. It creates the structured storyboard foundation that 0.17 prompt adapters will consume.

## Scope

### In Scope

- Repository support:
  - create rich storyboard shots with all 0.16 visible fields
  - update a shot after user edits
  - get one shot by id
  - list storyboards with typed shots
- Storyboard generation service:
  - read brand, project, assets, product understanding, creative direction, accepted memory, and research reports
  - plan shot count and durations from project duration
  - generate a deterministic fashion-ad storyboard
  - store storyboard and shots
  - record Agent run/events
- Selected-shot regeneration service:
  - validate shot belongs to storyboard and project
  - preserve shot number and duration
  - replace shot content using revision note and project context
  - record Agent run/events
- Command layer:
  - `joi_generate_storyboard`
  - `joi_list_storyboards`
  - `joi_update_shot`
  - `joi_regenerate_shot`
- Frontend Storyboard workspace:
  - generate form
  - saved storyboard list
  - shot cards/table
  - inline shot editing
  - selected-shot regenerate action
- Tests and smoke report.

### Out Of Scope

- No video generation.
- No prompt adapter generation; that starts in 0.17.
- No drag-and-drop timeline.
- No database migration for `garment_focus` or `transition`.
- No external model API calls.
- No semantic storyboard review; quality review starts in 0.19.

## Data Contract

### Existing Tables

0.16 reuses:

- `storyboards`
- `shots`
- `agent_runs`
- `agent_run_events`
- `memory_entries`
- `research_reports`
- `product_understandings`
- `creative_directions`

No migration is required.

### Shot Metadata Contract

Use `Shot.metadata_json` for fields not represented as first-class columns:

```json
{
  "format_version": "joi.shot_metadata.v1",
  "garment_focus": "water-resistant cotton texture and relaxed trench silhouette",
  "transition": "match cut into movement",
  "source_memory_ids": ["memory-1"],
  "source_research_report_ids": ["report-1"],
  "generation_context": {
    "stage": "0.16",
    "source": "storyboard_generation",
    "selling_point": "water-resistant cotton"
  }
}
```

Rules:

- `format_version` must be `joi.shot_metadata.v1`.
- `garment_focus` and `transition` are always present as strings.
- `source_memory_ids` contains accepted memory ids used by the shot.
- `source_research_report_ids` contains research report ids used by the shot.
- `generation_context.stage` is always `0.16`.
- User edits preserve `source_memory_ids` and `source_research_report_ids` unless the update explicitly provides new metadata through service code.

### `ShotPlanCreate`

Add to `src-tauri/src/repositories.rs`:

```rust
#[derive(Debug, Clone)]
pub struct ShotPlanCreate {
    pub storyboard_id: String,
    pub shot_number: i64,
    pub duration_seconds: i64,
    pub visual_description: String,
    pub model_action: String,
    pub garment_focus: String,
    pub camera_movement: String,
    pub scene: String,
    pub lighting: String,
    pub transition: String,
    pub subtitle_or_text: String,
    pub rationale: String,
    pub source_memory_ids: Vec<String>,
    pub source_research_report_ids: Vec<String>,
    pub generation_context: serde_json::Value,
}
```

Mapping:

- `visual_description` -> `shots.description`
- `model_action` -> `shots.model_action`
- `camera_movement` -> `shots.camera_movement`
- `scene` -> `shots.scene`
- `lighting` -> `shots.lighting`
- `subtitle_or_text` -> `shots.subtitle_or_voiceover`
- `rationale` -> `shots.rationale`
- `garment_focus`, `transition`, `source_memory_ids`, `source_research_report_ids`, `generation_context` -> `shots.metadata_json`

Validation:

- `storyboard_id` must exist.
- `shot_number` must be positive.
- `duration_seconds` must be positive.
- `visual_description`, `model_action`, `garment_focus`, `camera_movement`, `scene`, and `rationale` are required.
- `transition`, `lighting`, and `subtitle_or_text` may be empty but should be stored as strings.

### `ShotUpdate`

Add to `src-tauri/src/repositories.rs`:

```rust
#[derive(Debug, Clone)]
pub struct ShotUpdate {
    pub id: String,
    pub duration_seconds: i64,
    pub visual_description: String,
    pub model_action: String,
    pub garment_focus: String,
    pub camera_movement: String,
    pub scene: String,
    pub lighting: String,
    pub transition: String,
    pub subtitle_or_text: String,
    pub rationale: String,
    pub is_locked: bool,
}
```

Rules:

- Missing shot id returns `JoiError::NotFound`.
- `duration_seconds` must be positive.
- Required text fields match `ShotPlanCreate`.
- Existing `source_memory_ids`, `source_research_report_ids`, and `generation_context` are preserved.
- `updated_at` is refreshed.

### `StoryboardWithShots`

Add to `src-tauri/src/repositories.rs`:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoryboardWithShots {
    pub storyboard: crate::models::Storyboard,
    pub shots: Vec<crate::models::Shot>,
}
```

Add:

```rust
pub fn list_storyboards_with_typed_shots(
    &self,
    project_id: &str,
) -> JoiResult<Vec<StoryboardWithShots>>
```

Keep the existing `list_storyboards_with_shots` JSON method for snapshots.

### `StoryboardGenerationInput`

Create in `src-tauri/src/storyboard.rs`:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct StoryboardGenerationInput {
    pub project_id: String,
    pub user_direction: String,
    pub preferred_duration_seconds: Option<i64>,
    pub preferred_shot_count: Option<i64>,
}
```

Rules:

- `project_id` must exist.
- Final duration is `preferred_duration_seconds.unwrap_or(project.duration_seconds)`.
- Duration must be between `15` and `30` seconds inclusive.
- `preferred_shot_count`, when present, must be between `3` and `10`.
- Empty `user_direction` is allowed.

### `StoryboardShotView`

Create in `src-tauri/src/storyboard.rs`:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoryboardShotView {
    pub shot: crate::models::Shot,
    pub visual_description: String,
    pub garment_focus: String,
    pub transition: String,
}
```

Rules:

- `visual_description` mirrors `shot.description`.
- `garment_focus` and `transition` are extracted from `shot.metadata_json`.
- Missing metadata fields are returned as empty strings.

### `StoryboardGenerationResult`

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoryboardGenerationResult {
    pub storyboard: crate::models::Storyboard,
    pub shots: Vec<StoryboardShotView>,
    pub total_duration_seconds: i64,
    pub agent_run: crate::models::AgentRun,
    pub agent_events: Vec<crate::models::AgentRunEvent>,
}
```

### `ShotUpdateInput`

Create in `src-tauri/src/commands.rs`:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ShotUpdateInput {
    pub id: String,
    pub duration_seconds: i64,
    pub visual_description: String,
    pub model_action: String,
    pub garment_focus: String,
    pub camera_movement: String,
    pub scene: String,
    pub lighting: String,
    pub transition: String,
    pub subtitle_or_text: String,
    pub rationale: String,
    pub is_locked: bool,
}
```

### `ShotRegenerationInput`

Create in `src-tauri/src/storyboard.rs`:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ShotRegenerationInput {
    pub project_id: String,
    pub storyboard_id: String,
    pub shot_id: String,
    pub revision_note: String,
}
```

Rules:

- `project_id`, `storyboard_id`, and `shot_id` must exist and belong together.
- Locked shots cannot be regenerated.
- Empty `revision_note` is allowed and means "make the shot more specific using current project context."
- Shot number and duration are preserved.

### `ShotRegenerationResult`

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShotRegenerationResult {
    pub shot: StoryboardShotView,
    pub agent_run: crate::models::AgentRun,
    pub agent_events: Vec<crate::models::AgentRunEvent>,
}
```

## Storyboard Generation Rules

### Duration And Shot Count

Default shot count:

- 15 seconds -> 5 shots
- 16 to 20 seconds -> 6 shots
- 21 to 25 seconds -> 7 shots
- 26 to 30 seconds -> 8 shots

If `preferred_shot_count` is provided, use it after validating `3..=10`.

Duration distribution:

```text
base = total_duration_seconds / shot_count
remainder = total_duration_seconds % shot_count
first remainder shots get base + 1 seconds
remaining shots get base seconds
```

The sum of shot durations must equal storyboard duration exactly.

### Context Priority

Use project context in this order:

1. Project title, goal, and duration.
2. Latest product understanding.
3. Latest creative direction.
4. Accepted project memory only.
5. Research report creative implications.
6. Asset names and source URIs as visual anchors.
7. User direction.

Rejected and proposed memory must not influence generation.

### Shot Arc

Generated storyboards should follow this arc:

1. Opening hook: brand mood and product entrance.
2. Product proof: material, fabric, silhouette, or detail.
3. Model movement: show garment behavior.
4. Styling or scene expansion: context of use.
5. Closing memory: final product/brand impression.

For 6 to 8 shots, insert extra detail, movement, styling, and closing shots while preserving the same arc.

### Agent Events

Storyboard generation creates one Agent run:

- `status`: `completed`
- `runtime_kind`: `hermes_core`
- `runtime_mode`: `local_storyboard_bridge`
- `roles_json`: `["planner", "storyboard_writer", "reviewer"]`

Expected events:

1. planner `storyboard_context_read`
2. planner `duration_plan_created`
3. storyboard_writer `shot_requirements_mapped`
4. storyboard_writer `shots_drafted`
5. reviewer `duration_consistency_checked`
6. storyboard_writer `storyboard_saved`

Selected-shot regeneration creates one Agent run:

- `runtime_mode`: `local_storyboard_regeneration_bridge`
- same roles as generation

Expected events:

1. planner `shot_context_read`
2. storyboard_writer `revision_instruction_applied`
3. reviewer `shot_duration_preserved`
4. storyboard_writer `shot_saved`

## Implementation Tasks

### Task 1: Repository Support For Rich Shots

**Files:**

- Modify: `src-tauri/src/repositories.rs`
- Test: `src-tauri/tests/structured_content_repository.rs`

- [ ] **Step 1: Write failing repository tests**

Add imports:

```rust
use joi_agent_lib::repositories::{
    ShotPlanCreate, ShotUpdate, StoryboardCreate,
};
```

Add test:

```rust
#[test]
fn creates_shot_plan_with_visible_storyboard_fields() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());
    let project_id = seed_project(&repo);
    let storyboard = repo
        .create_storyboard(StoryboardCreate {
            project_id,
            title: "15s spring launch film".into(),
            duration_seconds: 15,
        })
        .expect("storyboard");

    let shot = repo
        .create_shot_plan(ShotPlanCreate {
            storyboard_id: storyboard.id.clone(),
            shot_number: 1,
            duration_seconds: 3,
            visual_description: "Model enters a clean studio frame wearing the trench.".into(),
            model_action: "Model walks forward and turns slightly toward camera.".into(),
            garment_focus: "relaxed trench silhouette and water-resistant cotton".into(),
            camera_movement: "slow push-in".into(),
            scene: "minimal warm studio".into(),
            lighting: "soft side light".into(),
            transition: "cut on movement".into(),
            subtitle_or_text: "Light enough for changing weather".into(),
            rationale: "Opening shot establishes product and brand mood.".into(),
            source_memory_ids: vec!["memory-1".into()],
            source_research_report_ids: vec!["report-1".into()],
            generation_context: json!({
                "stage": "0.16",
                "source": "storyboard_generation",
                "selling_point": "water-resistant cotton"
            }),
        })
        .expect("shot plan");

    assert_eq!(shot.description, "Model enters a clean studio frame wearing the trench.");
    assert_eq!(shot.model_action, "Model walks forward and turns slightly toward camera.");
    assert_eq!(shot.camera_movement, "slow push-in");
    assert_eq!(shot.scene, "minimal warm studio");
    assert_eq!(shot.lighting, "soft side light");
    assert_eq!(shot.subtitle_or_voiceover, "Light enough for changing weather");
    assert_eq!(shot.rationale, "Opening shot establishes product and brand mood.");
    assert_eq!(shot.metadata_json["format_version"], "joi.shot_metadata.v1");
    assert_eq!(shot.metadata_json["garment_focus"], "relaxed trench silhouette and water-resistant cotton");
    assert_eq!(shot.metadata_json["transition"], "cut on movement");
    assert_eq!(shot.metadata_json["source_memory_ids"], json!(["memory-1"]));
    assert_eq!(shot.metadata_json["source_research_report_ids"], json!(["report-1"]));
}
```

Add test:

```rust
#[test]
fn updates_shot_details_and_preserves_source_metadata() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());
    let project_id = seed_project(&repo);
    let storyboard = repo
        .create_storyboard(StoryboardCreate {
            project_id,
            title: "15s spring launch film".into(),
            duration_seconds: 15,
        })
        .expect("storyboard");
    let shot = repo
        .create_shot_plan(ShotPlanCreate {
            storyboard_id: storyboard.id,
            shot_number: 1,
            duration_seconds: 3,
            visual_description: "Original description".into(),
            model_action: "Original action".into(),
            garment_focus: "Original garment focus".into(),
            camera_movement: "Original camera".into(),
            scene: "Original scene".into(),
            lighting: "Original lighting".into(),
            transition: "Original transition".into(),
            subtitle_or_text: "Original text".into(),
            rationale: "Original rationale".into(),
            source_memory_ids: vec!["memory-1".into()],
            source_research_report_ids: vec!["report-1".into()],
            generation_context: json!({"stage": "0.16", "source": "storyboard_generation"}),
        })
        .expect("shot");

    let updated = repo
        .update_shot(ShotUpdate {
            id: shot.id.clone(),
            duration_seconds: 4,
            visual_description: "Close texture detail fills the frame.".into(),
            model_action: "Model lifts sleeve edge to reveal fabric movement.".into(),
            garment_focus: "fabric texture and sleeve construction".into(),
            camera_movement: "macro slide".into(),
            scene: "studio detail insert".into(),
            lighting: "grazing highlight".into(),
            transition: "match cut to walking shot".into(),
            subtitle_or_text: "Texture that moves".into(),
            rationale: "Edited to make product proof more specific.".into(),
            is_locked: true,
        })
        .expect("update shot");

    assert_eq!(updated.duration_seconds, 4);
    assert_eq!(updated.description, "Close texture detail fills the frame.");
    assert!(updated.is_locked);
    assert_eq!(updated.metadata_json["garment_focus"], "fabric texture and sleeve construction");
    assert_eq!(updated.metadata_json["transition"], "match cut to walking shot");
    assert_eq!(updated.metadata_json["source_memory_ids"], json!(["memory-1"]));
}
```

Add test:

```rust
#[test]
fn lists_storyboards_with_typed_shots_for_project() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());
    let project_id = seed_project(&repo);
    let storyboard = repo
        .create_storyboard(StoryboardCreate {
            project_id: project_id.clone(),
            title: "15s spring launch film".into(),
            duration_seconds: 15,
        })
        .expect("storyboard");
    repo.create_shot_plan(ShotPlanCreate {
        storyboard_id: storyboard.id.clone(),
        shot_number: 1,
        duration_seconds: 3,
        visual_description: "Opening product entrance.".into(),
        model_action: "Model enters frame.".into(),
        garment_focus: "outerwear silhouette".into(),
        camera_movement: "push in".into(),
        scene: "studio".into(),
        lighting: "soft".into(),
        transition: "cut".into(),
        subtitle_or_text: String::new(),
        rationale: "Establish the product.".into(),
        source_memory_ids: Vec::new(),
        source_research_report_ids: Vec::new(),
        generation_context: json!({"stage": "0.16"}),
    }).expect("shot");

    let storyboards = repo
        .list_storyboards_with_typed_shots(&project_id)
        .expect("storyboards with shots");

    assert_eq!(storyboards.len(), 1);
    assert_eq!(storyboards[0].storyboard.id, storyboard.id);
    assert_eq!(storyboards[0].shots.len(), 1);
    assert_eq!(storyboards[0].shots[0].shot_number, 1);
}
```

- [ ] **Step 2: Run tests and confirm RED**

```powershell
cd src-tauri
cargo test --test structured_content_repository creates_shot_plan_with_visible_storyboard_fields -- --nocapture
cargo test --test structured_content_repository updates_shot_details_and_preserves_source_metadata -- --nocapture
cargo test --test structured_content_repository lists_storyboards_with_typed_shots_for_project -- --nocapture
```

Expected:

- Tests fail because `ShotPlanCreate`, `ShotUpdate`, `create_shot_plan`, `update_shot`, and `list_storyboards_with_typed_shots` do not exist.

- [ ] **Step 3: Add repository structs**

Add the structs defined in the Data Contract section near existing `ShotCreate`.

- [ ] **Step 4: Implement `create_shot_plan`**

Implementation rules:

- Validate `shot_number > 0`.
- Validate `duration_seconds > 0`.
- Validate required text with `validate_required_text`.
- Call a new `get_storyboard(&self, id: &str)` helper to ensure the storyboard exists.
- Insert all shot columns directly.
- Build metadata exactly with `joi.shot_metadata.v1`.

- [ ] **Step 5: Implement `get_shot` and `update_shot`**

Implementation rules:

- `get_shot` maps one row with existing `map_shot`.
- `update_shot` reads the existing shot first.
- `update_shot` replaces `garment_focus` and `transition` in metadata.
- `update_shot` preserves source id arrays and generation context.
- `update_shot` returns the updated shot.

- [ ] **Step 6: Implement typed storyboard listing**

Implementation:

```rust
pub fn list_storyboards_with_typed_shots(
    &self,
    project_id: &str,
) -> JoiResult<Vec<StoryboardWithShots>> {
    let storyboards = self.list_storyboards(project_id)?;
    let mut values = Vec::with_capacity(storyboards.len());
    for storyboard in storyboards {
        let shots = self.list_shots(&storyboard.id)?;
        values.push(StoryboardWithShots { storyboard, shots });
    }
    Ok(values)
}
```

- [ ] **Step 7: Run repository tests**

```powershell
cd src-tauri
cargo test --test structured_content_repository -- --nocapture
```

Expected:

- All structured content repository tests pass.

- [ ] **Step 8: Commit repository support**

```powershell
git add src-tauri/src/repositories.rs src-tauri/tests/structured_content_repository.rs
git commit -m "feat: add Joi 0.16 rich storyboard shot repository"
```

### Task 2: Storyboard Generation Service

**Files:**

- Create: `src-tauri/src/storyboard.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/storyboard_generation.rs`

- [ ] **Step 1: Write failing service tests**

Create `src-tauri/tests/storyboard_generation.rs`.

Add test:

```rust
#[test]
fn generates_duration_balanced_storyboard_from_project_context() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());
    let project = seed_storyboard_project(&repo, 15);

    let result = generate_storyboard(
        &repo,
        StoryboardGenerationInput {
            project_id: project.id.clone(),
            user_direction: "Make the opening feel tactile and premium.".into(),
            preferred_duration_seconds: None,
            preferred_shot_count: None,
        },
        "local-test".into(),
    )
    .expect("storyboard");

    assert_eq!(result.storyboard.project_id, project.id);
    assert_eq!(result.storyboard.duration_seconds, 15);
    assert_eq!(result.shots.len(), 5);
    assert_eq!(result.total_duration_seconds, 15);
    assert_eq!(
        result.shots.iter().map(|item| item.shot.duration_seconds).sum::<i64>(),
        15
    );
    assert_eq!(result.shots[0].shot.shot_number, 1);
    assert!(result.shots[0].visual_description.contains("trench"));
    assert!(result.shots.iter().any(|item| item.garment_focus.contains("cotton")));
    assert_eq!(result.agent_run.runtime_mode, "local_storyboard_bridge");
    assert_eq!(result.agent_events.len(), 6);
}
```

Add test:

```rust
#[test]
fn generation_uses_accepted_memory_and_ignores_rejected_memory() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());
    let project = seed_storyboard_project(&repo, 15);
    let accepted = repo.create_memory_candidate(MemoryCandidateCreate {
        scope: "project".into(),
        brand_id: None,
        project_id: Some(project.id.clone()),
        content: "Use tactile close-ups before model movement.".into(),
        source: "user feedback".into(),
        source_entity_type: "feedback".into(),
        source_entity_id: String::new(),
        confidence: 0.86,
    }).expect("accepted memory seed");
    repo.update_memory_entry_status(MemoryStatusUpdate {
        id: accepted.id.clone(),
        status: "accepted".into(),
    }).expect("accept memory");
    let rejected = repo.create_memory_candidate(MemoryCandidateCreate {
        scope: "project".into(),
        brand_id: None,
        project_id: Some(project.id.clone()),
        content: "Make the opening shot dark and unrelated to the product.".into(),
        source: "user feedback".into(),
        source_entity_type: "feedback".into(),
        source_entity_id: String::new(),
        confidence: 0.86,
    }).expect("rejected memory seed");
    repo.update_memory_entry_status(MemoryStatusUpdate {
        id: rejected.id,
        status: "rejected".into(),
    }).expect("reject memory");

    let result = generate_storyboard(
        &repo,
        StoryboardGenerationInput {
            project_id: project.id,
            user_direction: String::new(),
            preferred_duration_seconds: Some(15),
            preferred_shot_count: Some(5),
        },
        "local-test".into(),
    )
    .expect("storyboard");

    let used_memory_ids = result
        .shots
        .iter()
        .flat_map(|item| {
            item.shot.metadata_json["source_memory_ids"]
                .as_array()
                .cloned()
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();

    assert!(used_memory_ids.contains(&json!(accepted.id)));
    assert!(!result
        .shots
        .iter()
        .any(|item| item.visual_description.to_lowercase().contains("unrelated")));
}
```

Add test:

```rust
#[test]
fn rejects_storyboard_duration_outside_short_ad_range() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());
    let project = seed_storyboard_project(&repo, 45);

    let error = generate_storyboard(
        &repo,
        StoryboardGenerationInput {
            project_id: project.id,
            user_direction: String::new(),
            preferred_duration_seconds: None,
            preferred_shot_count: None,
        },
        "local-test".into(),
    )
    .expect_err("duration should fail");

    assert!(matches!(error, JoiError::Validation(message) if message == "Storyboard duration must be between 15 and 30 seconds"));
}
```

Add test:

```rust
#[test]
fn regenerates_selected_unlocked_shot_and_preserves_duration() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());
    let project = seed_storyboard_project(&repo, 15);
    let generated = generate_storyboard(
        &repo,
        StoryboardGenerationInput {
            project_id: project.id.clone(),
            user_direction: String::new(),
            preferred_duration_seconds: Some(15),
            preferred_shot_count: Some(5),
        },
        "local-test".into(),
    )
    .expect("storyboard");
    let original = generated.shots[1].shot.clone();

    let result = regenerate_shot(
        &repo,
        ShotRegenerationInput {
            project_id: project.id,
            storyboard_id: generated.storyboard.id,
            shot_id: original.id.clone(),
            revision_note: "Make this shot a clearer fabric macro insert.".into(),
        },
        "local-test".into(),
    )
    .expect("regenerate shot");

    assert_eq!(result.shot.shot.id, original.id);
    assert_eq!(result.shot.shot.shot_number, original.shot_number);
    assert_eq!(result.shot.shot.duration_seconds, original.duration_seconds);
    assert!(result.shot.garment_focus.to_lowercase().contains("fabric"));
    assert_eq!(result.agent_run.runtime_mode, "local_storyboard_regeneration_bridge");
    assert_eq!(result.agent_events.len(), 4);
}
```

- [ ] **Step 2: Add shared test seed helper**

Inside `storyboard_generation.rs`, add:

```rust
fn seed_storyboard_project(repo: &Repository<'_>, duration_seconds: i64) -> Project {
    let brand = repo.create_brand(BrandCreate {
        name: "Atelier Joi".into(),
        description: "Contemporary womenswear with clean studio lighting".into(),
    }).expect("brand");
    let project = repo.create_project(ProjectCreate {
        brand_id: brand.id,
        title: "Spring Drop Film".into(),
        advertising_goal: "Launch a lightweight trench collection".into(),
        duration_seconds,
    }).expect("project");
    repo.create_product_understanding(ProductUnderstandingCreate {
        project_id: project.id.clone(),
        product_name: "Lightweight trench".into(),
        category: "outerwear".into(),
        audience: "urban commuters".into(),
        selling_points: vec![
            "water-resistant cotton".into(),
            "soft structure".into(),
            "easy movement".into(),
        ],
        constraints: vec!["avoid heavy winter styling".into()],
        notes: json!({
            "brief_summary": "15 second outerwear launch ad",
            "visual_direction": "clean studio walk with close fabric texture"
        }).to_string(),
    }).expect("understanding");
    repo.create_creative_direction(CreativeDirectionCreate {
        project_id: project.id.clone(),
        title: "Clean tactile motion".into(),
        concept: "Show fabric proof before model movement.".into(),
        tone: "premium and direct".into(),
        visual_style: "minimal warm studio, tactile close-ups".into(),
        scene_direction: "studio entrance, macro insert, walking motion, closing product pose".into(),
        rationale: "Derived from brief and material understanding.".into(),
    }).expect("creative direction");
    repo.create_research_report(ResearchReportCreate {
        project_id: project.id.clone(),
        summary: "Reference-backed tactile product proof.".into(),
        findings_json: json!([
            {
                "title": "Texture proof",
                "insight": "Fabric detail supports premium positioning.",
                "creative_implication": "Use tactile close-ups as visual proof before the model movement."
            }
        ]),
        sources_json: json!([
            {
                "index": 1,
                "title": "Reference note",
                "url": "https://example.com/reference",
                "source_type": "reference",
                "excerpt": "Texture details support premium positioning."
            }
        ]),
    }).expect("research");
    project
}
```

- [ ] **Step 3: Run tests and confirm RED**

```powershell
cd src-tauri
cargo test --test storyboard_generation -- --nocapture
```

Expected:

- Tests fail because `storyboard` module and service types do not exist.

- [ ] **Step 4: Create `src-tauri/src/storyboard.rs` DTOs**

Add all types from the Data Contract:

- `StoryboardGenerationInput`
- `StoryboardShotView`
- `StoryboardGenerationResult`
- `ShotRegenerationInput`
- `ShotRegenerationResult`

Add private helper structs:

```rust
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
    generation_context: serde_json::Value,
}
```

- [ ] **Step 5: Implement duration planning helpers**

Implement:

```rust
fn resolve_duration(input: &StoryboardGenerationInput, project_duration: i64) -> JoiResult<i64>
fn resolve_shot_count(input: &StoryboardGenerationInput, duration_seconds: i64) -> JoiResult<i64>
fn distribute_durations(total_duration_seconds: i64, shot_count: i64) -> Vec<i64>
```

Expected behavior:

- Duration must be `15..=30`.
- Shot count must be `3..=10`.
- Distributed durations sum to total exactly.

- [ ] **Step 6: Implement context extraction helpers**

Implement:

```rust
fn accepted_memory(context: &AgentProjectContext) -> Vec<MemoryEntry>
fn selling_points(context: &AgentProjectContext) -> Vec<String>
fn constraints(context: &AgentProjectContext) -> Vec<String>
fn research_implications(repo: &Repository<'_>, project_id: &str) -> JoiResult<Vec<(String, String)>>
```

Rules:

- `accepted_memory` filters `status == "accepted"`.
- `selling_points` reads `latest_product_understanding.selling_points_json` array.
- `constraints` reads `latest_product_understanding.constraints_json` array.
- `research_implications` returns `(report_id, creative_implication)` pairs and falls back to `insight`.

- [ ] **Step 7: Implement shot drafting**

Implement:

```rust
fn build_shot_drafts(
    context: &AgentProjectContext,
    research: &[(String, String)],
    input: &StoryboardGenerationInput,
    duration_seconds: i64,
    shot_count: i64,
) -> Vec<ShotDraft>
```

Rules:

- Use deterministic strings.
- Include product name when available.
- Include at least one selling point in each shot's `garment_focus`.
- Include accepted memory id when the shot uses memory-derived direction.
- Include research report id when the shot uses research-derived direction.
- Avoid rejected memory by only reading `accepted_memory`.

- [ ] **Step 8: Implement `generate_storyboard`**

Signature:

```rust
pub fn generate_storyboard(
    repo: &Repository<'_>,
    input: StoryboardGenerationInput,
    hermes_version: String,
) -> JoiResult<StoryboardGenerationResult>
```

Flow:

1. Build project context.
2. Resolve duration and shot count.
3. Read research implications.
4. Draft shots.
5. Create storyboard titled `"{project.title} storyboard"`.
6. Create shots with `repo.create_shot_plan`.
7. Create Agent run with `local_storyboard_bridge`.
8. Create six Agent events.
9. Return storyboard, expanded shot views, total duration, run, and events.

- [ ] **Step 9: Implement `regenerate_shot`**

Signature:

```rust
pub fn regenerate_shot(
    repo: &Repository<'_>,
    input: ShotRegenerationInput,
    hermes_version: String,
) -> JoiResult<ShotRegenerationResult>
```

Flow:

1. Build project context.
2. Read storyboard list for project and verify `storyboard_id`.
3. Read shot and verify `shot.storyboard_id == input.storyboard_id`.
4. Reject locked shots with message `Locked shots cannot be regenerated`.
5. Build one replacement draft using original shot number and duration.
6. Update shot with `repo.update_shot`.
7. Create Agent run with `local_storyboard_regeneration_bridge`.
8. Create four Agent events.
9. Return expanded shot view, run, and events.

- [ ] **Step 10: Export module in `lib.rs`**

Add:

```rust
pub mod storyboard;
```

- [ ] **Step 11: Run storyboard service tests**

```powershell
cd src-tauri
cargo test --test storyboard_generation -- --nocapture
```

Expected:

- All storyboard generation tests pass.

- [ ] **Step 12: Commit service**

```powershell
git add src-tauri/src/storyboard.rs src-tauri/src/lib.rs src-tauri/tests/storyboard_generation.rs
git commit -m "feat: add Joi 0.16 storyboard generation service"
```

### Task 3: Storyboard Commands

**Files:**

- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/commands.rs`

- [ ] **Step 1: Add command input round-trip tests**

Extend `command_inputs_round_trip_through_json`:

```rust
let storyboard_generation: StoryboardGenerationInput = serde_json::from_value(json!({
    "project_id": "project-1",
    "user_direction": "Keep tactile product proof before the walk.",
    "preferred_duration_seconds": 15,
    "preferred_shot_count": 5
}))
.expect("storyboard generation input");
assert_eq!(storyboard_generation.preferred_shot_count, Some(5));

let shot_update: ShotUpdateInput = serde_json::from_value(json!({
    "id": "shot-1",
    "duration_seconds": 3,
    "visual_description": "Close fabric texture detail.",
    "model_action": "Model lifts sleeve edge.",
    "garment_focus": "fabric texture",
    "camera_movement": "macro slide",
    "scene": "studio insert",
    "lighting": "soft side light",
    "transition": "match cut",
    "subtitle_or_text": "Texture that moves",
    "rationale": "Clarifies product proof.",
    "is_locked": false
}))
.expect("shot update input");
assert_eq!(shot_update.garment_focus, "fabric texture");

let shot_regeneration: ShotRegenerationInput = serde_json::from_value(json!({
    "project_id": "project-1",
    "storyboard_id": "storyboard-1",
    "shot_id": "shot-1",
    "revision_note": "Make the garment proof clearer."
}))
.expect("shot regeneration input");
assert_eq!(shot_regeneration.shot_id, "shot-1");
```

- [ ] **Step 2: Add state helper command test**

Add test:

```rust
#[test]
fn state_helpers_generate_list_update_and_regenerate_storyboard() {
    let (_app, state) = test_state();
    let brand = create_brand(
        &state,
        BrandInput {
            name: "Atelier Joi".to_string(),
            description: "Premium womenswear".to_string(),
        },
    )
    .expect("brand");
    let project = create_project(
        &state,
        ProjectInput {
            brand_id: brand.id,
            title: "Spring Drop Film".to_string(),
            advertising_goal: "Launch awareness for a lightweight trench".to_string(),
            duration_seconds: 15,
        },
    )
    .expect("project");
    generate_brief_understanding(
        &state,
        BriefUnderstandingInput {
            project_id: project.id.clone(),
            brief_text: "15 second outerwear launch ad".to_string(),
            product_name: "Lightweight trench".to_string(),
            category: "outerwear".to_string(),
            audience: "urban commuters".to_string(),
            target_platforms: vec!["jimeng_video".to_string(), "grok_video".to_string()],
            selling_points_text: "water-resistant cotton, soft structure, easy movement".to_string(),
            visual_direction: "clean studio walk with close fabric texture".to_string(),
            constraints_text: "avoid heavy winter styling".to_string(),
            reference_asset_ids: Vec::new(),
        },
    )
    .expect("understanding");

    let result = generate_storyboard(
        &state,
        StoryboardGenerationInput {
            project_id: project.id.clone(),
            user_direction: "Make the opening tactile.".to_string(),
            preferred_duration_seconds: Some(15),
            preferred_shot_count: Some(5),
        },
    )
    .expect("generate storyboard");

    assert_eq!(result.shots.len(), 5);
    assert_eq!(result.agent_run.runtime_mode, "local_storyboard_bridge");

    let storyboards = list_storyboards(&state, project.id.clone()).expect("list storyboards");
    assert_eq!(storyboards.len(), 1);
    assert_eq!(storyboards[0].shots.len(), 5);

    let edited = update_shot(
        &state,
        ShotUpdateInput {
            id: result.shots[0].shot.id.clone(),
            duration_seconds: result.shots[0].shot.duration_seconds,
            visual_description: "Edited opening product entrance.".to_string(),
            model_action: "Model steps into frame.".to_string(),
            garment_focus: "trench silhouette".to_string(),
            camera_movement: "slow push".to_string(),
            scene: "studio".to_string(),
            lighting: "soft side light".to_string(),
            transition: "cut on movement".to_string(),
            subtitle_or_text: "Built for changing weather".to_string(),
            rationale: "User edit makes opening clearer.".to_string(),
            is_locked: false,
        },
    )
    .expect("update shot");
    assert_eq!(edited.visual_description, "Edited opening product entrance.");

    let regenerated = regenerate_shot(
        &state,
        ShotRegenerationInput {
            project_id: project.id,
            storyboard_id: result.storyboard.id,
            shot_id: result.shots[1].shot.id.clone(),
            revision_note: "Make this shot a clearer macro fabric insert.".to_string(),
        },
    )
    .expect("regenerate shot");
    assert_eq!(regenerated.agent_run.runtime_mode, "local_storyboard_regeneration_bridge");
    assert_eq!(regenerated.shot.shot.id, result.shots[1].shot.id);
}
```

- [ ] **Step 3: Run command tests and confirm RED**

```powershell
cd src-tauri
cargo test --test commands command_inputs_round_trip_through_json -- --nocapture
cargo test --test commands state_helpers_generate_list_update_and_regenerate_storyboard -- --nocapture
```

Expected:

- Tests fail because command imports, inputs, helpers, and handlers do not exist.

- [ ] **Step 4: Add command imports and DTO**

In `commands.rs`, import:

```rust
use crate::storyboard::{
    generate_storyboard as generate_storyboard_service,
    regenerate_shot as regenerate_shot_service,
    ShotRegenerationInput,
    ShotRegenerationResult,
    StoryboardGenerationInput,
    StoryboardGenerationResult,
    StoryboardShotView,
};
use crate::repositories::{ShotUpdate, StoryboardWithShots};
```

Add `ShotUpdateInput` from the Data Contract.

- [ ] **Step 5: Add Tauri command handlers**

Add:

```rust
#[tauri::command(rename_all = "snake_case")]
pub fn joi_generate_storyboard(
    state: State<'_, AppState>,
    input: StoryboardGenerationInput,
) -> JoiResult<StoryboardGenerationResult> {
    generate_storyboard(state.inner(), input)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_list_storyboards(
    state: State<'_, AppState>,
    project_id: String,
) -> JoiResult<Vec<StoryboardWithShots>> {
    list_storyboards(state.inner(), project_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_update_shot(
    state: State<'_, AppState>,
    input: ShotUpdateInput,
) -> JoiResult<StoryboardShotView> {
    update_shot(state.inner(), input)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_regenerate_shot(
    state: State<'_, AppState>,
    input: ShotRegenerationInput,
) -> JoiResult<ShotRegenerationResult> {
    regenerate_shot(state.inner(), input)
}
```

- [ ] **Step 6: Add helper functions**

Add:

```rust
pub fn generate_storyboard(
    state: &AppState,
    input: StoryboardGenerationInput,
) -> JoiResult<StoryboardGenerationResult> {
    let runtime_status = get_agent_runtime_status(state)?;
    let db = lock_db(state)?;
    generate_storyboard_service(
        &Repository::new(db.connection()),
        input,
        runtime_status.hermes_version,
    )
}

pub fn list_storyboards(
    state: &AppState,
    project_id: String,
) -> JoiResult<Vec<StoryboardWithShots>> {
    let db = lock_db(state)?;
    Repository::new(db.connection()).list_storyboards_with_typed_shots(&project_id)
}

pub fn update_shot(state: &AppState, input: ShotUpdateInput) -> JoiResult<StoryboardShotView> {
    let db = lock_db(state)?;
    let shot = Repository::new(db.connection()).update_shot(ShotUpdate {
        id: input.id,
        duration_seconds: input.duration_seconds,
        visual_description: input.visual_description,
        model_action: input.model_action,
        garment_focus: input.garment_focus,
        camera_movement: input.camera_movement,
        scene: input.scene,
        lighting: input.lighting,
        transition: input.transition,
        subtitle_or_text: input.subtitle_or_text,
        rationale: input.rationale,
        is_locked: input.is_locked,
    })?;
    Ok(StoryboardShotView::from_shot(shot))
}

pub fn regenerate_shot(
    state: &AppState,
    input: ShotRegenerationInput,
) -> JoiResult<ShotRegenerationResult> {
    let runtime_status = get_agent_runtime_status(state)?;
    let db = lock_db(state)?;
    regenerate_shot_service(
        &Repository::new(db.connection()),
        input,
        runtime_status.hermes_version,
    )
}
```

Add `StoryboardShotView::from_shot` as a public associated function in `storyboard.rs`.

- [ ] **Step 7: Register commands in `lib.rs`**

Add to `tauri::generate_handler!`:

```rust
commands::joi_generate_storyboard,
commands::joi_list_storyboards,
commands::joi_update_shot,
commands::joi_regenerate_shot,
```

- [ ] **Step 8: Run command tests**

```powershell
cd src-tauri
cargo test --test commands -- --nocapture
```

Expected:

- Command tests pass.

- [ ] **Step 9: Commit commands**

```powershell
git add src-tauri/src/commands.rs src-tauri/src/lib.rs src-tauri/tests/commands.rs
git commit -m "feat: expose Joi 0.16 storyboard commands"
```

### Task 4: Frontend Storyboard Workspace

**Files:**

- Modify: `src/types/joi.ts`
- Modify: `src/api/joiApi.ts`
- Modify: `src/App.tsx`
- Modify: `src/components/ProjectWorkspace.tsx`
- Create: `src/components/StoryboardWorkspace.tsx`
- Modify: `src/styles.css`
- Test: `src/App.test.tsx`

- [ ] **Step 1: Add frontend types**

Add to `src/types/joi.ts`:

```ts
export type Storyboard = {
  id: string;
  project_id: string;
  title: string;
  duration_seconds: number;
  created_at: string;
  updated_at: string;
};

export type Shot = {
  id: string;
  storyboard_id: string;
  shot_number: number;
  duration_seconds: number;
  description: string;
  model_action: string;
  camera_movement: string;
  scene: string;
  lighting: string;
  subtitle_or_voiceover: string;
  rationale: string;
  is_locked: boolean;
  metadata_json: unknown;
  created_at: string;
  updated_at: string;
};

export type StoryboardWithShots = {
  storyboard: Storyboard;
  shots: Shot[];
};

export type StoryboardGenerationInput = {
  project_id: string;
  user_direction: string;
  preferred_duration_seconds: number | null;
  preferred_shot_count: number | null;
};

export type StoryboardShotView = {
  shot: Shot;
  visual_description: string;
  garment_focus: string;
  transition: string;
};

export type StoryboardGenerationResult = {
  storyboard: Storyboard;
  shots: StoryboardShotView[];
  total_duration_seconds: number;
  agent_run: AgentRun;
  agent_events: AgentRunEvent[];
};

export type ShotUpdateInput = {
  id: string;
  duration_seconds: number;
  visual_description: string;
  model_action: string;
  garment_focus: string;
  camera_movement: string;
  scene: string;
  lighting: string;
  transition: string;
  subtitle_or_text: string;
  rationale: string;
  is_locked: boolean;
};

export type ShotRegenerationInput = {
  project_id: string;
  storyboard_id: string;
  shot_id: string;
  revision_note: string;
};

export type ShotRegenerationResult = {
  shot: StoryboardShotView;
  agent_run: AgentRun;
  agent_events: AgentRunEvent[];
};
```

- [ ] **Step 2: Add API wrappers**

Add imports and wrappers to `src/api/joiApi.ts`:

```ts
export function generateStoryboard(input: StoryboardGenerationInput): Promise<StoryboardGenerationResult> {
  return invoke<StoryboardGenerationResult>("joi_generate_storyboard", { input });
}

export function listStoryboards(projectId: string): Promise<StoryboardWithShots[]> {
  return invoke<StoryboardWithShots[]>("joi_list_storyboards", { project_id: projectId });
}

export function updateShot(input: ShotUpdateInput): Promise<StoryboardShotView> {
  return invoke<StoryboardShotView>("joi_update_shot", { input });
}

export function regenerateShot(input: ShotRegenerationInput): Promise<ShotRegenerationResult> {
  return invoke<ShotRegenerationResult>("joi_regenerate_shot", { input });
}
```

- [ ] **Step 3: Add failing UI test**

In `src/App.test.tsx`, extend the Tauri invoke mock:

```ts
case "joi_list_storyboards":
  return Promise.resolve([]);
case "joi_generate_storyboard":
  return Promise.resolve(mockStoryboardGenerationResult);
case "joi_update_shot":
  return Promise.resolve({
    shot: { ...mockStoryboardGenerationResult.shots[0].shot, description: args.input.visual_description },
    visual_description: args.input.visual_description,
    garment_focus: args.input.garment_focus,
    transition: args.input.transition,
  });
case "joi_regenerate_shot":
  return Promise.resolve({
    shot: {
      ...mockStoryboardGenerationResult.shots[1],
      visual_description: "Regenerated macro fabric insert.",
      garment_focus: "fabric texture",
    },
    agent_run: mockAgentRun("run-storyboard-regen", "local_storyboard_regeneration_bridge"),
    agent_events: [],
  });
```

Add fixture:

```ts
const mockStoryboardGenerationResult = {
  storyboard: {
    id: "storyboard-1",
    project_id: "project-1",
    title: "Spring Drop Film storyboard",
    duration_seconds: 15,
    created_at: "2026-06-15T00:00:00Z",
    updated_at: "2026-06-15T00:00:00Z",
  },
  shots: [
    {
      shot: {
        id: "shot-1",
        storyboard_id: "storyboard-1",
        shot_number: 1,
        duration_seconds: 3,
        description: "Model enters a clean studio frame wearing the trench.",
        model_action: "Model walks forward.",
        camera_movement: "slow push-in",
        scene: "minimal warm studio",
        lighting: "soft side light",
        subtitle_or_voiceover: "Light enough for changing weather",
        rationale: "Opening shot establishes product and brand mood.",
        is_locked: false,
        metadata_json: {
          format_version: "joi.shot_metadata.v1",
          garment_focus: "water-resistant cotton trench silhouette",
          transition: "cut on movement",
        },
        created_at: "2026-06-15T00:00:00Z",
        updated_at: "2026-06-15T00:00:00Z",
      },
      visual_description: "Model enters a clean studio frame wearing the trench.",
      garment_focus: "water-resistant cotton trench silhouette",
      transition: "cut on movement",
    },
    {
      shot: {
        id: "shot-2",
        storyboard_id: "storyboard-1",
        shot_number: 2,
        duration_seconds: 3,
        description: "Close fabric texture detail fills the frame.",
        model_action: "Model lifts sleeve edge.",
        camera_movement: "macro slide",
        scene: "studio insert",
        lighting: "grazing highlight",
        subtitle_or_voiceover: "Texture that moves",
        rationale: "Product proof shot.",
        is_locked: false,
        metadata_json: {
          format_version: "joi.shot_metadata.v1",
          garment_focus: "fabric texture",
          transition: "match cut",
        },
        created_at: "2026-06-15T00:00:00Z",
        updated_at: "2026-06-15T00:00:00Z",
      },
      visual_description: "Close fabric texture detail fills the frame.",
      garment_focus: "fabric texture",
      transition: "match cut",
    },
  ],
  total_duration_seconds: 15,
  agent_run: mockAgentRun("run-storyboard", "local_storyboard_bridge"),
  agent_events: [],
};
```

Add test:

```ts
test("generates edits and regenerates storyboard shots", async () => {
  render(<App />);
  await screen.findByText("Spring Drop Film");

  fireEvent.click(screen.getByRole("button", { name: "Storyboard" }));
  fireEvent.change(screen.getByLabelText(/Storyboard direction/i), {
    target: { value: "Make the opening tactile and premium." },
  });
  fireEvent.click(screen.getByRole("button", { name: /Generate Storyboard/i }));

  await waitFor(() => {
    expect(invokeMock).toHaveBeenCalledWith("joi_generate_storyboard", {
      input: {
        project_id: "project-1",
        user_direction: "Make the opening tactile and premium.",
        preferred_duration_seconds: 15,
        preferred_shot_count: 5,
      },
    });
  });
  expect(await screen.findByText("water-resistant cotton trench silhouette")).toBeInTheDocument();

  fireEvent.click(screen.getAllByRole("button", { name: /Edit Shot/i })[0]);
  fireEvent.change(screen.getByLabelText(/Visual description/i), {
    target: { value: "Edited opening product entrance." },
  });
  fireEvent.click(screen.getByRole("button", { name: /Save Shot/i }));
  await waitFor(() => {
    expect(invokeMock).toHaveBeenCalledWith("joi_update_shot", expect.objectContaining({
      input: expect.objectContaining({
        id: "shot-1",
        visual_description: "Edited opening product entrance.",
      }),
    }));
  });

  fireEvent.change(screen.getByLabelText(/Regeneration note/i), {
    target: { value: "Make shot 2 a clearer macro fabric insert." },
  });
  fireEvent.click(screen.getAllByRole("button", { name: /Regenerate Shot/i })[1]);
  await waitFor(() => {
    expect(invokeMock).toHaveBeenCalledWith("joi_regenerate_shot", expect.objectContaining({
      input: expect.objectContaining({
        project_id: "project-1",
        storyboard_id: "storyboard-1",
        shot_id: "shot-2",
      }),
    }));
  });
});
```

- [ ] **Step 4: Run UI test and confirm RED**

```powershell
npm test -- src/App.test.tsx
```

Expected:

- Test fails because Storyboard workspace and API wrappers are missing.

- [ ] **Step 5: Create `StoryboardWorkspace.tsx`**

Create component with props:

```ts
export type StoryboardDraft = {
  user_direction: string;
  preferred_duration_seconds: string;
  preferred_shot_count: string;
  regeneration_note: string;
};
```

The component must render:

- `Storyboard direction` textarea
- `Duration seconds` number input
- `Shot count` number input
- `Generate Storyboard` button
- saved storyboard summary
- shot rows/cards with stable dimensions
- `Edit Shot` button
- `Save Shot` button when editing
- `Regeneration note` textarea
- `Regenerate Shot` button per shot

Keep cards shallow; do not nest cards inside cards.

- [ ] **Step 6: Wire App state**

In `App.tsx`, add state:

```ts
const emptyStoryboardDraft: StoryboardDraft = {
  user_direction: "",
  preferred_duration_seconds: "15",
  preferred_shot_count: "5",
  regeneration_note: "",
};
const [storyboardDraft, setStoryboardDraft] = useState<StoryboardDraft>(emptyStoryboardDraft);
const [storyboards, setStoryboards] = useState<StoryboardWithShots[]>([]);
const [storyboardResult, setStoryboardResult] = useState<StoryboardGenerationResult | null>(null);
const [generatingStoryboard, setGeneratingStoryboard] = useState(false);
const [savingShotId, setSavingShotId] = useState<string | null>(null);
const [regeneratingShotId, setRegeneratingShotId] = useState<string | null>(null);
```

Update `refreshProjectState` to call `listStoryboards(projectId)`.

Reset storyboard state when project changes or new project starts.

Add handlers:

- `submitStoryboardGeneration`
- `handleUpdateShot`
- `handleRegenerateShot`

After generation/regeneration:

- refresh project state
- prepend returned Agent run to `agentRuns`
- add activity log entry

- [ ] **Step 7: Render Storyboard tab**

In `ProjectWorkspace.tsx`:

- Import `StoryboardWorkspace`.
- Add props for storyboard state and handlers.
- Render it when `activeTab === "Storyboard"`.
- Remove `Storyboard` from the placeholder branch.

- [ ] **Step 8: Add focused CSS**

In `src/styles.css`, add:

- `.storyboard-layout`
- `.storyboard-toolbar`
- `.storyboard-shot-grid`
- `.shot-row`
- `.shot-meta-grid`
- `.shot-actions`
- mobile rules to keep fields from overflowing.

Constraints:

- No one-note palette change.
- No nested card styling.
- Buttons must keep text readable at mobile width.
- Shot rows must not resize when edit controls appear.

- [ ] **Step 9: Run frontend tests and build**

```powershell
npm test
npm run build
```

Expected:

- Tests and build pass.

- [ ] **Step 10: Commit frontend**

```powershell
git add src/types/joi.ts src/api/joiApi.ts src/App.tsx src/components/ProjectWorkspace.tsx src/components/StoryboardWorkspace.tsx src/styles.css src/App.test.tsx
git commit -m "feat: add Joi 0.16 storyboard workspace"
```

### Task 5: Smoke, Review, Merge, Push

**Files:**

- Create: `docs/superpowers/reports/joi-0.16-storyboard-generation-smoke-test.md`

- [ ] **Step 1: Run full verification**

```powershell
npm test
npm run build
cd src-tauri
cargo test
cargo test --test storyboard_generation -- --nocapture
cargo test --test commands -- --nocapture
```

Expected:

- Frontend tests pass.
- Frontend production build passes.
- Rust tests pass.

- [ ] **Step 2: Browser smoke**

Start:

```powershell
npm run dev -- --host 127.0.0.1 --port 1420
```

Verify with a Tauri invoke mock when running in a normal browser:

- Storyboard tab renders generation controls.
- Generate Storyboard creates visible shot rows.
- Total duration equals project duration.
- Each shot shows visual description, model action, garment focus, camera movement, scene, transition, text, and rationale.
- Edit Shot opens editable fields.
- Save Shot calls `joi_update_shot`.
- Regenerate Shot calls `joi_regenerate_shot`.
- Desktop 1440x900 has no horizontal overflow.
- Mobile 390x844 has no horizontal overflow.

Normal browser limitation:

- A normal browser cannot call native Tauri commands; command integration is covered by Rust and React tests. Browser smoke may use a Tauri invoke mock.

- [ ] **Step 3: Write smoke report**

Create `docs/superpowers/reports/joi-0.16-storyboard-generation-smoke-test.md` with:

- commands run
- browser viewports checked
- acceptance checklist
- known limitations

- [ ] **Step 4: Commit smoke report**

```powershell
git add docs/superpowers/reports/joi-0.16-storyboard-generation-smoke-test.md
git commit -m "test: add Joi 0.16 storyboard smoke report"
```

- [ ] **Step 5: Merge to main**

From the main workspace:

```powershell
git status --short --branch
git merge --ff-only codex/joi-0.16-storyboard-generation
```

- [ ] **Step 6: Verify on main**

```powershell
npm test
npm run build
cd src-tauri
cargo test
```

- [ ] **Step 7: Push**

```powershell
git push origin main
```

If HTTPS push fails but GitHub API and SSH are reachable, use the temporary writable deploy key fallback already proven in earlier stages:

- create temporary ed25519 key in the system temp directory
- add it to `Drew1811266/Joi-agent` as writable deploy key
- push over `ssh.github.com:443`
- delete deploy key
- delete local temp key
- verify remote main SHA through GitHub API

- [ ] **Step 8: Clean up worktree**

```powershell
git worktree remove --force "D:\Software Project\Joi-agent\.worktrees\joi-0.16-storyboard-generation"
git worktree prune
git branch -d codex/joi-0.16-storyboard-generation
```

## Acceptance Criteria

0.16 is complete only when:

- User can generate a complete storyboard from a selected project.
- Generated storyboard duration is between 15 and 30 seconds.
- Sum of shot durations exactly matches storyboard duration.
- Each shot has visual description, model action, garment focus, camera movement, scene, transition, text suggestion, and rationale.
- Storyboard generation uses current project context.
- Storyboard generation uses accepted memory and ignores proposed/rejected memory.
- Storyboard generation can reference research report implications.
- Storyboard and shots are saved to the local repository.
- User can edit and save a shot.
- User can regenerate one selected unlocked shot.
- Storyboard generation creates Agent run/events.
- Shot regeneration creates Agent run/events.
- Storyboards appear in snapshots through existing snapshot support.
- Tests cover repository, service, commands, and frontend flow.
- Browser smoke report is written.
- Changes are merged to `main` and pushed to GitHub.

## Risks And Mitigations

### Risk: Existing Shot Schema Is Too Narrow

Mitigation:

- Store `garment_focus` and `transition` in `metadata_json` with an explicit format version.
- Keep UI and command contracts typed so future migration to first-class columns is straightforward.

### Risk: Deterministic Generation Feels Too Formulaic

Mitigation:

- 0.16 optimizes for usable structure, persistence, and editability.
- 0.17 and 0.19 can improve adapter quality and review loops.
- User direction, accepted memory, and research implications already vary the generated output.

### Risk: Regeneration Accidentally Changes Storyboard Timing

Mitigation:

- `regenerate_shot` preserves shot number and duration.
- Repository and service tests assert preserved duration.
- Full storyboard duration remains stable unless the user manually edits shot duration.

### Risk: Memory Status Leakage

Mitigation:

- Service helper filters memory to `status == "accepted"`.
- Tests seed accepted and rejected memory and assert rejected content is ignored.

## Handoff To 0.17

0.17 prompt adapters should consume the structured storyboard output:

- `shot.description` as visual description
- `shot.model_action` as action
- `shot.camera_movement` as camera language
- `shot.scene` and `shot.lighting` as visual environment
- `shot.metadata_json.garment_focus` as product proof
- `shot.metadata_json.transition` as video continuity
- `shot.subtitle_or_voiceover` as optional on-screen text
- `shot.rationale` as adapter reasoning context

0.17 should not need to infer shot structure from freeform text.

## Self-Review

- Spec coverage: This plan covers 0.16 roadmap scope: storyboard generation from project context, duration planning, required shot fields, editing UI, selected-shot regeneration, persistence, Agent logs, tests, smoke, merge, and push.
- Placeholder scan: No task contains unresolved placeholder instructions or unnamed tests. Files, commands, types, function names, and expected outputs are concrete.
- Type consistency: `StoryboardGenerationInput`, `StoryboardShotView`, `StoryboardGenerationResult`, `ShotUpdateInput`, `ShotRegenerationInput`, `ShotRegenerationResult`, `ShotPlanCreate`, `ShotUpdate`, and `StoryboardWithShots` are used consistently across backend, command, frontend, and tests.
