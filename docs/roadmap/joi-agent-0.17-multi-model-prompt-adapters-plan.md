# Joi Agent 0.17 Multi-Model Prompt Adapters Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert Joi storyboards and image briefs into editable, saved, copyable prompt packages for Jimeng, Grok, Banana 2, Jimeng Image, and GPT Image 2.

**Architecture:** 0.17 keeps prompt generation local and deterministic. It extends the existing `prompt_packages` aggregate so video prompts can stay shot-bound while image prompts can be project-bound, then adds a `prompt_adapter` service that turns structured project context into platform-specific prompt packages with completeness checks and Agent run events.

**Tech Stack:** Tauri 2 commands, Rust, rusqlite, serde/serde_json, chrono, React 19, TypeScript, Vitest, Joi local repository, Joi 0.13 Agent run model, Joi 0.16 storyboard output.

---

## Product Outcome

After 0.17, a user can:

- Open the Prompts tab for a selected project.
- Generate Jimeng and Grok video prompts from selected storyboard shots.
- Generate Banana 2, Jimeng Image, and GPT Image 2 image prompts from one image brief.
- See platform-specific prompt text, negative prompt, adapter parameters, and missing-field warnings.
- Edit and save prompt text and negative prompt.
- Copy prompt text from the UI.
- See generated prompt packages persisted with project, platform, modality, source metadata, and Agent run logs.
- Save snapshots that include prompt packages through the existing snapshot path.

0.17 does not call external model APIs, manage model accounts, upload media to model platforms, or validate provider-side runtime limits. The supported adapter IDs are local output profiles that match the target labels already chosen for Joi 0.2.

## Scope

### In Scope

- Prompt package data hardening:
  - allow image prompt packages without `shot_id`
  - keep shot-bound prompts for video
  - save `negative_prompt`
  - save adapter parameters and completeness metadata in `parameters_json`
  - update prompt packages after user edits
- Prompt adapter service:
  - adapter registry for `jimeng_video`, `grok_video`, `banana_2_image`, `jimeng_image`, `gpt_image_2`
  - deterministic prompt generation from project context, storyboard shots, image brief, accepted memory, and research implications
  - per-platform negative prompt strategy
  - completeness checks for subject, scene, action, camera, material, lighting, style, and constraints
  - Agent run/events for generation
- Command layer:
  - `joi_generate_prompt_packages`
  - `joi_list_prompt_packages`
  - `joi_update_prompt_package`
  - `joi_get_prompt_adapter_profiles`
- Frontend Prompts workspace:
  - video platform selectors
  - image platform selectors
  - shot selection for video prompt generation
  - image brief field for image prompt generation
  - package list/editor
  - missing-field indicators
  - copy prompt button
- Tests and smoke report.

### Out Of Scope

- No external model API calls.
- No model account or token management.
- No image or video file generation.
- No provider upload workflow.
- No advanced prompt scoring beyond completeness checks.
- No semantic brand review; broader quality review starts in 0.19.

## Existing Code Context

Use these current pieces:

- `src-tauri/src/models.rs`
  - already defines `PromptPlatform` values:
    - `jimeng_video`
    - `grok_video`
    - `banana_2_image`
    - `jimeng_image`
    - `gpt_image_2`
  - already defines `PromptModality` values:
    - `video`
    - `image`
  - currently has `PromptPackage.shot_id: String`
- `src-tauri/src/db.rs`
  - currently creates `prompt_packages.shot_id TEXT NOT NULL`
  - current triggers enforce that shot-bound prompt packages belong to the same project
- `src-tauri/src/repositories.rs`
  - currently has `PromptPackageCreate`
  - currently supports `create_prompt_package` and `list_prompt_packages`
- `src-tauri/src/storyboard.rs`
  - provides `StoryboardShotView::from_shot`
  - stores `garment_focus` and `transition` in shot metadata
- `src/components/ProjectWorkspace.tsx`
  - already has a `Prompts` tab in navigation
  - currently routes that tab to `EmptyState`
- `src/App.tsx`
  - already refreshes assets, memory, understandings, research reports, storyboards, and agent runs
  - needs to refresh prompt packages as part of project state

## Data Contract

### Prompt Package Schema

0.17 changes `prompt_packages.shot_id` from required to optional.

Rules:

- Video prompt packages require `shot_id`.
- Image prompt packages may have `shot_id = NULL`.
- If `shot_id` is present, it must belong to a shot under the same project.
- If `shot_id` is absent, `modality` must be `image`.
- `platform` and `modality` must pass existing `validate_prompt_modality`.

### `PromptPackage.parameters_json`

Use this shape:

```json
{
  "format_version": "joi.prompt_package_parameters.v1",
  "adapter_id": "jimeng_video",
  "adapter_display_name": "Jimeng Video",
  "source_type": "storyboard_shot",
  "source_storyboard_id": "storyboard-1",
  "source_shot_id": "shot-1",
  "source_image_brief": "",
  "required_fields": ["subject", "scene", "action", "camera", "material", "lighting", "style"],
  "missing_fields": [],
  "copy_blocks": {
    "main_prompt": "A 3 second fashion advertising shot...",
    "negative_prompt": "low resolution, warped hands...",
    "notes": "Keep garment visible and preserve shot duration."
  },
  "generation_context": {
    "stage": "0.17",
    "user_direction": "Keep prompts concise and production-ready.",
    "accepted_memory_ids": ["memory-1"],
    "research_report_ids": ["research-1"]
  }
}
```

Rules:

- `format_version` must be `joi.prompt_package_parameters.v1`.
- `adapter_id` must match `platform`.
- `source_type` is `storyboard_shot` for video prompts and `image_brief` for image prompts.
- `missing_fields` stores field keys, not display labels.
- `copy_blocks.main_prompt` mirrors `prompt_text`.
- `copy_blocks.negative_prompt` mirrors `negative_prompt`.

### Rust Type Changes

Modify `src-tauri/src/models.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptPackage {
    pub id: String,
    pub project_id: String,
    pub shot_id: Option<String>,
    pub platform: String,
    pub modality: String,
    pub prompt_text: String,
    pub negative_prompt: String,
    pub parameters_json: Value,
    pub is_locked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

Modify `src-tauri/src/repositories.rs`:

```rust
#[derive(Debug, Clone)]
pub struct PromptPackageCreate {
    pub project_id: String,
    pub shot_id: Option<String>,
    pub platform: String,
    pub modality: String,
    pub prompt_text: String,
    pub negative_prompt: String,
    pub parameters_json: Value,
}

#[derive(Debug, Clone)]
pub struct PromptPackageUpdate {
    pub id: String,
    pub prompt_text: String,
    pub negative_prompt: String,
    pub parameters_json: Value,
    pub is_locked: bool,
}
```

### Prompt Adapter Types

Create `src-tauri/src/prompt_adapter.rs`:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PromptAdapterProfile {
    pub id: String,
    pub display_name: String,
    pub modality: String,
    pub default_negative_prompt: String,
    pub required_fields: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PromptGenerationInput {
    pub project_id: String,
    pub shot_ids: Vec<String>,
    pub image_brief: String,
    pub target_platforms: Vec<String>,
    pub user_direction: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PromptCompletenessCheck {
    pub field: String,
    pub label: String,
    pub present: bool,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PromptPackageView {
    pub package: crate::models::PromptPackage,
    pub adapter_display_name: String,
    pub completeness: Vec<PromptCompletenessCheck>,
    pub missing_fields: Vec<String>,
    pub copy_text: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PromptGenerationResult {
    pub packages: Vec<PromptPackageView>,
    pub agent_run: crate::models::AgentRun,
    pub agent_events: Vec<crate::models::AgentRunEvent>,
}
```

### Command Input Type

Create in `src-tauri/src/commands.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromptPackageUpdateInput {
    pub id: String,
    pub prompt_text: String,
    pub negative_prompt: String,
    pub parameters_json: serde_json::Value,
    pub is_locked: bool,
}
```

### Frontend Types

Add to `src/types/joi.ts`:

```ts
export type PromptPackage = {
  id: string;
  project_id: string;
  shot_id: string | null;
  platform: string;
  modality: string;
  prompt_text: string;
  negative_prompt: string;
  parameters_json: unknown;
  is_locked: boolean;
  created_at: string;
  updated_at: string;
};

export type PromptAdapterProfile = {
  id: string;
  display_name: string;
  modality: string;
  default_negative_prompt: string;
  required_fields: string[];
};

export type PromptGenerationInput = {
  project_id: string;
  shot_ids: string[];
  image_brief: string;
  target_platforms: string[];
  user_direction: string;
};

export type PromptCompletenessCheck = {
  field: string;
  label: string;
  present: boolean;
  message: string;
};

export type PromptPackageView = {
  package: PromptPackage;
  adapter_display_name: string;
  completeness: PromptCompletenessCheck[];
  missing_fields: string[];
  copy_text: string;
};

export type PromptGenerationResult = {
  packages: PromptPackageView[];
  agent_run: AgentRun;
  agent_events: AgentRunEvent[];
};

export type PromptPackageUpdateInput = {
  id: string;
  prompt_text: string;
  negative_prompt: string;
  parameters_json: unknown;
  is_locked: boolean;
};
```

## Adapter Rules

### Supported Profiles

The registry must return exactly these profiles:

```text
jimeng_video   | Jimeng Video   | video | subject, scene, action, camera, material, lighting, style
grok_video     | Grok Video     | video | subject, scene, action, camera, material, lighting, style
banana_2_image | Banana 2 Image | image | subject, scene, garment, material, lighting, style
jimeng_image   | Jimeng Image   | image | subject, scene, garment, material, lighting, style
gpt_image_2    | GPT Image 2    | image | subject, scene, garment, material, lighting, style
```

### Video Prompt Composition

For each selected video platform and each selected shot:

- `subject`: product name or project title
- `scene`: `shot.scene`
- `action`: `shot.model_action`
- `camera`: `shot.camera_movement`
- `material`: `shot.metadata_json.garment_focus`
- `lighting`: `shot.lighting`
- `style`: latest creative direction visual style or `clean fashion advertising`
- `text`: `shot.subtitle_or_voiceover`
- `duration`: `shot.duration_seconds`
- `continuity`: `shot.metadata_json.transition`
- `constraints`: latest product constraints and accepted memory

Jimeng video prompt text must use compact production language:

```text
Jimeng video prompt:
3s fashion advertising shot. Subject: Lightweight trench. Scene: minimal warm studio. Action: Model walks forward. Camera: slow push-in. Garment focus: water-resistant cotton trench silhouette. Lighting: soft side light. Style: clean studio fashion ad. On-screen text: Light enough for changing weather. Transition: cut on movement. Keep fabric texture visible.
```

Grok video prompt text must be slightly more descriptive:

```text
Grok video prompt:
Create a 3 second fashion ad shot for Lightweight trench. Show the model walking forward in a minimal warm studio, with a slow push-in camera move and soft side light. Prioritize the water-resistant cotton trench silhouette, natural garment movement, and a polished clean studio fashion style. Optional text: Light enough for changing weather. Avoid heavy winter styling.
```

### Image Prompt Composition

For each selected image platform:

- `subject`: product name or project title
- `scene`: image brief scene if present, otherwise latest creative direction scene
- `garment`: image brief garment phrase plus product name
- `material`: selling points and accepted memory
- `lighting`: image brief lighting if present, otherwise derived from creative direction
- `style`: latest creative direction visual style
- `constraints`: product constraints, brand negative preferences, and user direction

Banana 2 prompt text:

```text
Banana 2 image prompt:
Fashion model product photo for Lightweight trench, full-body editorial pose, minimal warm studio, water-resistant cotton texture visible, clean studio fashion ad style, soft directional light, premium ecommerce-ready composition, natural hands, accurate garment construction.
```

Jimeng Image prompt text:

```text
Jimeng image prompt:
服装广告模拍图，Lightweight trench，模特自然站姿，极简暖色棚拍场景，突出 water-resistant cotton texture，柔和定向光，干净高级的春季外套广告风格，服装结构准确，面料纹理清晰。
```

GPT Image 2 prompt text:

```text
GPT Image 2 prompt:
Create a realistic fashion campaign image of a model wearing Lightweight trench in a minimal warm studio. Emphasize water-resistant cotton texture, relaxed trench silhouette, accurate garment construction, soft directional fashion lighting, clean premium composition, and natural pose.
```

### Negative Prompt Strategy

Video platforms:

```text
low resolution, distorted garment shape, warped hands, extra limbs, unreadable text, heavy flicker, abrupt camera jitter, fabric texture lost, off-brand styling
```

Image platforms:

```text
low resolution, distorted hands, extra fingers, incorrect garment construction, warped seams, blurry fabric texture, messy background, harsh shadows, over-stylized illustration, off-brand colors
```

### Completeness Checks

Use deterministic string checks:

- subject is present when product name or project title is non-empty.
- scene is present when shot scene, image brief, or creative direction scene is non-empty.
- action is present for video when shot model action is non-empty.
- camera is present for video when shot camera movement is non-empty.
- garment is present for image when product name or image brief is non-empty.
- material is present when garment focus, selling points, or image brief is non-empty.
- lighting is present when shot lighting, image brief, or derived lighting is non-empty.
- style is present when latest creative direction visual style or fallback style is non-empty.

Missing fields must not block generation. They appear in UI as warnings and are stored in `parameters_json.missing_fields`.

## Implementation Tasks

### Task 1: Optional-Shot Prompt Package Repository

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
fn prompt_packages_allow_project_bound_image_prompts() {
    let db = migrated_in_memory_database();
    insert_brand_project_storyboard_shot(&db);

    db.connection()
        .execute(
            "INSERT INTO prompt_packages (
                id, project_id, shot_id, platform, modality, prompt_text, negative_prompt,
                parameters_json, is_locked, created_at, updated_at
            ) VALUES (
                'prompt-image-1', 'project-1', NULL, 'gpt_image_2', 'image',
                'image prompt', 'negative prompt', '{}', 0,
                '2026-06-15T00:00:00Z', '2026-06-15T00:00:00Z'
            )",
            [],
        )
        .expect("project-bound image prompt insert");
}
```

Expected RED:

```powershell
cd src-tauri
cargo test --test db_migration prompt_packages_allow_project_bound_image_prompts -- --nocapture
```

The test fails because `prompt_packages.shot_id` is currently `NOT NULL`.

- [ ] **Step 2: Write failing repository tests**

Update `src-tauri/tests/structured_content_repository.rs` imports:

```rust
use joi_agent_lib::repositories::{
    PromptPackageCreate, PromptPackageUpdate, Repository, ResearchReportCreate, ShotCreate,
    ShotPlanCreate, ShotUpdate,
};
```

Add test:

```rust
#[test]
fn creates_project_bound_image_prompt_package_with_adapter_metadata() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());
    let project_id = seed_project(&repo);

    let prompt = repo
        .create_prompt_package(PromptPackageCreate {
            project_id: project_id.clone(),
            shot_id: None,
            platform: "gpt_image_2".into(),
            modality: "image".into(),
            prompt_text: "Create a realistic fashion campaign image.".into(),
            negative_prompt: "distorted hands, incorrect garment construction".into(),
            parameters_json: json!({
                "format_version": "joi.prompt_package_parameters.v1",
                "adapter_id": "gpt_image_2",
                "source_type": "image_brief",
                "missing_fields": []
            }),
        })
        .expect("image prompt");

    assert_eq!(prompt.project_id, project_id);
    assert_eq!(prompt.shot_id, None);
    assert_eq!(prompt.platform, "gpt_image_2");
    assert_eq!(prompt.modality, "image");
    assert_eq!(prompt.negative_prompt, "distorted hands, incorrect garment construction");
    assert_eq!(prompt.parameters_json["adapter_id"], "gpt_image_2");
}
```

Add test:

```rust
#[test]
fn rejects_video_prompt_without_shot() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());
    let project_id = seed_project(&repo);

    let result = repo.create_prompt_package(PromptPackageCreate {
        project_id,
        shot_id: None,
        platform: "jimeng_video".into(),
        modality: "video".into(),
        prompt_text: "video prompt".into(),
        negative_prompt: String::new(),
        parameters_json: json!({}),
    });

    assert!(result.is_err());
}
```

Add test:

```rust
#[test]
fn updates_prompt_package_text_negative_prompt_parameters_and_lock() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());
    let project_id = seed_project(&repo);
    let prompt = repo
        .create_prompt_package(PromptPackageCreate {
            project_id,
            shot_id: None,
            platform: "banana_2_image".into(),
            modality: "image".into(),
            prompt_text: "first prompt".into(),
            negative_prompt: "first negative".into(),
            parameters_json: json!({"format_version": "joi.prompt_package_parameters.v1"}),
        })
        .expect("prompt");

    let updated = repo
        .update_prompt_package(PromptPackageUpdate {
            id: prompt.id.clone(),
            prompt_text: "edited prompt".into(),
            negative_prompt: "edited negative".into(),
            parameters_json: json!({
                "format_version": "joi.prompt_package_parameters.v1",
                "adapter_id": "banana_2_image",
                "missing_fields": ["lighting"]
            }),
            is_locked: true,
        })
        .expect("updated prompt");

    assert_eq!(updated.prompt_text, "edited prompt");
    assert_eq!(updated.negative_prompt, "edited negative");
    assert!(updated.is_locked);
    assert_eq!(updated.parameters_json["missing_fields"], json!(["lighting"]));
}
```

Run:

```powershell
cd src-tauri
cargo test --test structured_content_repository creates_project_bound_image_prompt_package_with_adapter_metadata -- --nocapture
cargo test --test structured_content_repository rejects_video_prompt_without_shot -- --nocapture
cargo test --test structured_content_repository updates_prompt_package_text_negative_prompt_parameters_and_lock -- --nocapture
```

Expected RED:

- `PromptPackage.shot_id` is still `String`.
- `PromptPackageCreate` does not accept `negative_prompt`, `parameters_json`, or optional `shot_id`.
- `PromptPackageUpdate` and `update_prompt_package` do not exist.

- [ ] **Step 3: Update schema creation and migration**

In `src-tauri/src/db.rs`, change the `prompt_packages` table definition:

```sql
shot_id TEXT,
```

Update both prompt package triggers:

```sql
WHEN NEW.shot_id IS NOT NULL
  AND EXISTS (SELECT 1 FROM shots WHERE shots.id = NEW.shot_id)
  AND EXISTS (SELECT 1 FROM projects WHERE projects.id = NEW.project_id)
  AND NOT EXISTS (
    SELECT 1
    FROM shots
    JOIN storyboards ON storyboards.id = shots.storyboard_id
    WHERE shots.id = NEW.shot_id
      AND storyboards.project_id = NEW.project_id
  )
```

Add a post-schema migration in `Database::migrate`:

```rust
pub fn migrate(&self) -> JoiResult<()> {
    self.connection.execute_batch(SCHEMA)?;
    self.migrate_prompt_packages_optional_shot()?;
    Ok(())
}
```

Add helper:

```rust
fn migrate_prompt_packages_optional_shot(&self) -> JoiResult<()> {
    let not_null = {
        let mut statement = self.connection.prepare("PRAGMA table_info(prompt_packages)")?;
        let rows = statement.query_map([], |row| {
            let name: String = row.get(1)?;
            let not_null: i64 = row.get(3)?;
            Ok((name, not_null))
        })?;
        let mut shot_not_null = false;
        for row in rows {
            let (name, not_null) = row?;
            if name == "shot_id" {
                shot_not_null = not_null == 1;
            }
        }
        shot_not_null
    };

    if !not_null {
        return Ok(());
    }

    self.connection.execute_batch(
        r#"
        DROP TRIGGER IF EXISTS trg_prompt_packages_shot_belongs_to_project_insert;
        DROP TRIGGER IF EXISTS trg_prompt_packages_shot_belongs_to_project_update;
        DROP INDEX IF EXISTS idx_prompt_packages_project_id;
        DROP INDEX IF EXISTS idx_prompt_packages_shot_id;
        ALTER TABLE prompt_packages RENAME TO prompt_packages_legacy;
        CREATE TABLE prompt_packages (
          id TEXT PRIMARY KEY,
          project_id TEXT NOT NULL,
          shot_id TEXT,
          platform TEXT NOT NULL,
          modality TEXT NOT NULL,
          prompt_text TEXT NOT NULL DEFAULT '',
          negative_prompt TEXT NOT NULL DEFAULT '',
          parameters_json TEXT NOT NULL DEFAULT '{}',
          is_locked INTEGER NOT NULL DEFAULT 0,
          created_at TEXT NOT NULL,
          updated_at TEXT NOT NULL,
          FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
          FOREIGN KEY (shot_id) REFERENCES shots(id) ON DELETE CASCADE
        );
        INSERT INTO prompt_packages (
          id, project_id, shot_id, platform, modality, prompt_text, negative_prompt,
          parameters_json, is_locked, created_at, updated_at
        )
        SELECT id, project_id, shot_id, platform, modality, prompt_text, negative_prompt,
               parameters_json, is_locked, created_at, updated_at
        FROM prompt_packages_legacy;
        DROP TABLE prompt_packages_legacy;
        "#,
    )?;
    self.connection.execute_batch(SCHEMA)?;
    Ok(())
}
```

- [ ] **Step 4: Update Rust model and repository**

In `src-tauri/src/models.rs`, change `PromptPackage.shot_id` to `Option<String>`.

In `src-tauri/src/repositories.rs`:

- update `PromptPackageCreate`
- add `PromptPackageUpdate`
- in `create_prompt_package`:
  - validate platform/modality
  - validate `prompt_text` with `validate_required_text("Prompt text", ...)`
  - reject video modality when `shot_id.is_none()`
  - when `shot_id.is_some()`, call `get_shot`
  - insert optional `shot_id`
  - save provided `negative_prompt` and `parameters_json`
- add `get_prompt_package(&self, id: &str)`
- add `update_prompt_package(&self, input: PromptPackageUpdate)`
- update `map_prompt_package` to read `shot_id: Option<String>`

Use this update SQL:

```rust
let affected = self.connection.execute(
    "UPDATE prompt_packages
     SET prompt_text = ?1, negative_prompt = ?2, parameters_json = ?3,
         is_locked = ?4, updated_at = ?5
     WHERE id = ?6",
    params![
        input.prompt_text.trim(),
        input.negative_prompt.trim(),
        input.parameters_json.to_string(),
        if input.is_locked { 1 } else { 0 },
        now.to_rfc3339(),
        input.id
    ],
)?;
```

- [ ] **Step 5: Update snapshot tests**

In `src-tauri/tests/project_snapshots.rs`, update prompt fixture creation to pass:

```rust
negative_prompt: String::new(),
parameters_json: json!({
    "format_version": "joi.prompt_package_parameters.v1",
    "adapter_id": "jimeng_video"
}),
```

Add an image prompt to the snapshot test:

```rust
let image_prompt = repo
    .create_prompt_package(PromptPackageCreate {
        project_id: project_id.clone(),
        shot_id: None,
        platform: "gpt_image_2".into(),
        modality: "image".into(),
        prompt_text: "A campaign still prompt".into(),
        negative_prompt: "distorted hands".into(),
        parameters_json: json!({
            "format_version": "joi.prompt_package_parameters.v1",
            "adapter_id": "gpt_image_2"
        }),
    })
    .expect("image prompt package");
```

Assert:

```rust
assert!(snapshot["prompt_packages"]
    .as_array()
    .unwrap()
    .iter()
    .any(|item| item["id"] == image_prompt.id && item["shot_id"].is_null()));
```

- [ ] **Step 6: Run repository and migration tests**

```powershell
cd src-tauri
cargo test --test db_migration -- --nocapture
cargo test --test structured_content_repository -- --nocapture
cargo test --test project_snapshots -- --nocapture
```

Expected:

- All three test files pass.

- [ ] **Step 7: Commit repository foundation**

```powershell
git add src-tauri/src/db.rs src-tauri/src/models.rs src-tauri/src/repositories.rs src-tauri/tests/db_migration.rs src-tauri/tests/structured_content_repository.rs src-tauri/tests/project_snapshots.rs
git commit -m "feat: support Joi 0.17 prompt packages"
```

### Task 2: Prompt Adapter Service

**Files:**

- Create: `src-tauri/src/prompt_adapter.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/prompt_adapter.rs`

- [ ] **Step 1: Write failing service tests**

Create `src-tauri/tests/prompt_adapter.rs`:

```rust
mod common;

use serde_json::json;

use common::TestApp;
use joi_agent_lib::db::Database;
use joi_agent_lib::prompt_adapter::{
    generate_prompt_packages, prompt_adapter_profiles, PromptGenerationInput,
};
use joi_agent_lib::repositories::{
    CreativeDirectionCreate, ProductUnderstandingCreate, Repository, ShotPlanCreate,
    StoryboardCreate,
};

fn migrated_database() -> (TestApp, Database) {
    let app = TestApp::new();
    let db = Database::open(app.db_path()).expect("open db");
    db.migrate().expect("migrate");
    (app, db)
}
```

Add test:

```rust
#[test]
fn returns_expected_adapter_profiles() {
    let profiles = prompt_adapter_profiles();
    let ids = profiles.iter().map(|profile| profile.id.as_str()).collect::<Vec<_>>();

    assert_eq!(
        ids,
        vec![
            "jimeng_video",
            "grok_video",
            "banana_2_image",
            "jimeng_image",
            "gpt_image_2"
        ]
    );
    assert_eq!(profiles[0].modality, "video");
    assert_eq!(profiles[2].modality, "image");
}
```

Add test:

```rust
#[test]
fn generates_video_prompt_packages_for_selected_shots() {
    let (_app, db) = migrated_database();
    let repo = Repository::new(db.connection());
    let project_id = seed_prompt_project(&repo);
    let storyboard = repo
        .create_storyboard(StoryboardCreate {
            project_id: project_id.clone(),
            title: "Spring Drop storyboard".into(),
            duration_seconds: 15,
        })
        .expect("storyboard");
    let shot = repo
        .create_shot_plan(ShotPlanCreate {
            storyboard_id: storyboard.id.clone(),
            shot_number: 1,
            duration_seconds: 3,
            visual_description: "Model enters frame wearing a trench.".into(),
            model_action: "Model walks forward.".into(),
            garment_focus: "water-resistant cotton trench silhouette".into(),
            camera_movement: "slow push-in".into(),
            scene: "minimal warm studio".into(),
            lighting: "soft side light".into(),
            transition: "cut on movement".into(),
            subtitle_or_text: "Light enough for changing weather".into(),
            rationale: "Opening hook.".into(),
            source_memory_ids: Vec::new(),
            source_research_report_ids: Vec::new(),
            generation_context: json!({"stage": "0.16"}),
        })
        .expect("shot");

    let result = generate_prompt_packages(
        &repo,
        PromptGenerationInput {
            project_id: project_id.clone(),
            shot_ids: vec![shot.id.clone()],
            image_brief: String::new(),
            target_platforms: vec!["jimeng_video".into(), "grok_video".into()],
            user_direction: "Keep prompts concise.".into(),
        },
        "0.17.0".into(),
    )
    .expect("prompt generation");

    assert_eq!(result.packages.len(), 2);
    assert!(result.packages.iter().all(|item| item.package.shot_id == Some(shot.id.clone())));
    assert!(result.packages.iter().any(|item| item.package.platform == "jimeng_video"));
    assert!(result.packages.iter().any(|item| item.package.platform == "grok_video"));
    assert!(result.packages[0].package.prompt_text.contains("water-resistant cotton trench silhouette"));
    assert!(result.packages[0].missing_fields.is_empty());
    assert_eq!(result.agent_run.runtime_mode, "local_prompt_adapter_bridge");
    assert_eq!(result.agent_events.len(), 5);
}
```

Add test:

```rust
#[test]
fn generates_project_bound_image_prompt_packages_from_image_brief() {
    let (_app, db) = migrated_database();
    let repo = Repository::new(db.connection());
    let project_id = seed_prompt_project(&repo);

    let result = generate_prompt_packages(
        &repo,
        PromptGenerationInput {
            project_id: project_id.clone(),
            shot_ids: Vec::new(),
            image_brief: "Full-body ecommerce model photo, warm studio, emphasize cotton texture.".into(),
            target_platforms: vec![
                "banana_2_image".into(),
                "jimeng_image".into(),
                "gpt_image_2".into(),
            ],
            user_direction: "Natural model pose.".into(),
        },
        "0.17.0".into(),
    )
    .expect("image prompts");

    assert_eq!(result.packages.len(), 3);
    assert!(result.packages.iter().all(|item| item.package.shot_id.is_none()));
    assert!(result.packages.iter().all(|item| item.package.modality == "image"));
    assert!(result.packages.iter().all(|item| item.package.prompt_text.contains("Lightweight trench")));
    assert!(result
        .packages
        .iter()
        .any(|item| item.package.platform == "jimeng_image" && item.package.prompt_text.contains("服装广告模拍图")));
}
```

Add test:

```rust
#[test]
fn rejects_video_generation_without_shots_and_unknown_platforms() {
    let (_app, db) = migrated_database();
    let repo = Repository::new(db.connection());
    let project_id = seed_prompt_project(&repo);

    let missing_shot = generate_prompt_packages(
        &repo,
        PromptGenerationInput {
            project_id: project_id.clone(),
            shot_ids: Vec::new(),
            image_brief: String::new(),
            target_platforms: vec!["jimeng_video".into()],
            user_direction: String::new(),
        },
        "0.17.0".into(),
    );
    assert!(missing_shot.is_err());

    let unknown = generate_prompt_packages(
        &repo,
        PromptGenerationInput {
            project_id,
            shot_ids: Vec::new(),
            image_brief: "A studio still.".into(),
            target_platforms: vec!["unknown_platform".into()],
            user_direction: String::new(),
        },
        "0.17.0".into(),
    );
    assert!(unknown.is_err());
}
```

Add helper:

```rust
fn seed_prompt_project(repo: &Repository<'_>) -> String {
    let brand = repo
        .create_brand(joi_agent_lib::repositories::BrandCreate {
            name: "Atelier Joi".into(),
            description: "Editorial womenswear with clean studio campaigns.".into(),
        })
        .expect("brand");
    let project = repo
        .create_project(joi_agent_lib::repositories::ProjectCreate {
            brand_id: brand.id,
            title: "Spring Drop Film".into(),
            advertising_goal: "Launch awareness for lightweight trench.".into(),
            duration_seconds: 15,
        })
        .expect("project");
    repo.create_product_understanding(ProductUnderstandingCreate {
        project_id: project.id.clone(),
        product_name: "Lightweight trench".into(),
        category: "outerwear".into(),
        audience: "urban commuters".into(),
        selling_points: vec!["water-resistant cotton".into(), "soft structure".into()],
        constraints: vec!["avoid heavy winter styling".into()],
        notes: "{}".into(),
    })
    .expect("understanding");
    repo.create_creative_direction(CreativeDirectionCreate {
        project_id: project.id.clone(),
        title: "Clean studio movement".into(),
        concept: "clean studio walk with tactile close-up".into(),
        tone: "premium".into(),
        visual_style: "clean studio fashion ad".into(),
        scene_direction: "minimal warm studio".into(),
        rationale: "Matches launch goal.".into(),
    })
    .expect("direction");
    project.id
}
```

Run:

```powershell
cd src-tauri
cargo test --test prompt_adapter -- --nocapture
```

Expected RED:

- `prompt_adapter` module does not exist.

- [ ] **Step 2: Implement adapter registry**

Create `src-tauri/src/prompt_adapter.rs` and add:

```rust
const PROMPT_ADAPTER_ROLES: [&str; 3] = ["planner", "prompt_adapter", "reviewer"];

pub fn prompt_adapter_profiles() -> Vec<PromptAdapterProfile> {
    vec![
        profile("jimeng_video", "Jimeng Video", "video", VIDEO_NEGATIVE, video_fields()),
        profile("grok_video", "Grok Video", "video", VIDEO_NEGATIVE, video_fields()),
        profile("banana_2_image", "Banana 2 Image", "image", IMAGE_NEGATIVE, image_fields()),
        profile("jimeng_image", "Jimeng Image", "image", IMAGE_NEGATIVE, image_fields()),
        profile("gpt_image_2", "GPT Image 2", "image", IMAGE_NEGATIVE, image_fields()),
    ]
}
```

Add helpers:

```rust
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
    ["subject", "scene", "action", "camera", "material", "lighting", "style"]
        .into_iter()
        .map(ToString::to_string)
        .collect()
}

fn image_fields() -> Vec<String> {
    ["subject", "scene", "garment", "material", "lighting", "style"]
        .into_iter()
        .map(ToString::to_string)
        .collect()
}
```

- [ ] **Step 3: Implement prompt generation**

Add public function:

```rust
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

    let mut package_views = Vec::new();
    for profile in targets {
        if profile.modality == "video" {
            for shot_id in &input.shot_ids {
                let shot = repo.get_shot(shot_id)?;
                let view = create_video_prompt(
                    repo,
                    &context,
                    &profile,
                    &input,
                    &shot,
                    &accepted_memory,
                    &research,
                )?;
                package_views.push(view);
            }
        } else {
            let view = create_image_prompt(
                repo,
                &context,
                &profile,
                &input,
                &accepted_memory,
                &research,
            )?;
            package_views.push(view);
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
        plan_json: build_prompt_plan_json(&input),
        result_summary: format!(
            "Generated {} prompt package(s) for {}.",
            package_views.len(),
            context.project.title
        ),
    })?;

    let agent_events = create_prompt_events(repo, &agent_run.id, &context, &input, &package_views)?;
    Ok(PromptGenerationResult {
        packages: package_views,
        agent_run,
        agent_events,
    })
}
```

Required event types:

```text
1 planner        prompt_context_read
2 planner        prompt_targets_resolved
3 prompt_adapter prompts_drafted
4 reviewer       prompt_completeness_checked
5 prompt_adapter prompt_packages_saved
```

- [ ] **Step 4: Implement package view creation**

Add:

```rust
pub fn prompt_package_view(package: PromptPackage) -> PromptPackageView {
    let adapter_display_name = string_field(&package.parameters_json, "adapter_display_name")
        .unwrap_or_else(|| package.platform.clone());
    let missing_fields = package
        .parameters_json
        .get("missing_fields")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
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
```

Use `prompt_package_view` in list and update commands so the frontend receives consistent completeness data.

- [ ] **Step 5: Expose module**

Modify `src-tauri/src/lib.rs`:

```rust
pub mod prompt_adapter;
```

- [ ] **Step 6: Run service tests**

```powershell
cd src-tauri
cargo test --test prompt_adapter -- --nocapture
```

Expected:

- All prompt adapter tests pass.

- [ ] **Step 7: Commit service**

```powershell
git add src-tauri/src/prompt_adapter.rs src-tauri/src/lib.rs src-tauri/tests/prompt_adapter.rs
git commit -m "feat: add Joi 0.17 prompt adapter service"
```

### Task 3: Prompt Adapter Commands

**Files:**

- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/commands.rs`

- [ ] **Step 1: Write failing command tests**

In `src-tauri/tests/commands.rs`, extend `command_inputs_round_trip_through_json`:

```rust
let prompt_generation: joi_agent_lib::prompt_adapter::PromptGenerationInput =
    serde_json::from_value(json!({
        "project_id": "project-1",
        "shot_ids": ["shot-1"],
        "image_brief": "",
        "target_platforms": ["jimeng_video", "grok_video"],
        "user_direction": "Keep prompts concise."
    }))
    .expect("prompt generation input");
assert_eq!(prompt_generation.target_platforms.len(), 2);

let prompt_update: commands::PromptPackageUpdateInput = serde_json::from_value(json!({
    "id": "prompt-1",
    "prompt_text": "edited prompt",
    "negative_prompt": "edited negative",
    "parameters_json": {"format_version": "joi.prompt_package_parameters.v1"},
    "is_locked": true
}))
.expect("prompt update input");
assert!(prompt_update.is_locked);
```

Add state helper test:

```rust
#[test]
fn state_helpers_generate_list_and_update_prompt_packages() {
    let app = TestApp::new();
    let state = test_state(&app);
    let brand = commands::create_brand(
        &state,
        commands::BrandInput {
            name: "Atelier Joi".into(),
            description: "Editorial womenswear".into(),
        },
    )
    .expect("brand");
    let project = commands::create_project(
        &state,
        commands::ProjectInput {
            brand_id: brand.id,
            title: "Spring Drop Film".into(),
            advertising_goal: "Launch awareness".into(),
            duration_seconds: 15,
        },
    )
    .expect("project");
    let storyboard = seed_command_storyboard(&state, &project.id);

    let result = commands::generate_prompt_packages(
        &state,
        joi_agent_lib::prompt_adapter::PromptGenerationInput {
            project_id: project.id.clone(),
            shot_ids: vec![storyboard.shots[0].id.clone()],
            image_brief: "Full-body studio model photo.".into(),
            target_platforms: vec!["jimeng_video".into(), "gpt_image_2".into()],
            user_direction: "Make output production-ready.".into(),
        },
    )
    .expect("prompt generation");

    assert_eq!(result.packages.len(), 2);
    let listed = commands::list_prompt_packages(&state, project.id.clone()).expect("listed prompts");
    assert_eq!(listed.len(), 2);

    let updated = commands::update_prompt_package(
        &state,
        commands::PromptPackageUpdateInput {
            id: listed[0].package.id.clone(),
            prompt_text: "edited prompt".into(),
            negative_prompt: "edited negative".into(),
            parameters_json: listed[0].package.parameters_json.clone(),
            is_locked: true,
        },
    )
    .expect("updated prompt");
    assert_eq!(updated.package.prompt_text, "edited prompt");
    assert!(updated.package.is_locked);
}
```

Run:

```powershell
cd src-tauri
cargo test --test commands state_helpers_generate_list_and_update_prompt_packages -- --nocapture
```

Expected RED:

- command helper functions and input type do not exist.

- [ ] **Step 2: Add command imports and handlers**

In `src-tauri/src/commands.rs`, import:

```rust
use crate::prompt_adapter::{
    generate_prompt_packages as generate_prompt_packages_service, prompt_adapter_profiles,
    prompt_package_view, PromptAdapterProfile, PromptGenerationInput, PromptGenerationResult,
    PromptPackageView,
};
use crate::repositories::{PromptPackageUpdate, ...};
```

Add command functions:

```rust
#[tauri::command(rename_all = "snake_case")]
pub fn joi_get_prompt_adapter_profiles() -> Vec<PromptAdapterProfile> {
    get_prompt_adapter_profiles()
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_generate_prompt_packages(
    state: State<'_, AppState>,
    input: PromptGenerationInput,
) -> JoiResult<PromptGenerationResult> {
    generate_prompt_packages(state.inner(), input)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_list_prompt_packages(
    state: State<'_, AppState>,
    project_id: String,
) -> JoiResult<Vec<PromptPackageView>> {
    list_prompt_packages(state.inner(), project_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_update_prompt_package(
    state: State<'_, AppState>,
    input: PromptPackageUpdateInput,
) -> JoiResult<PromptPackageView> {
    update_prompt_package(state.inner(), input)
}
```

Add helper functions:

```rust
pub fn get_prompt_adapter_profiles() -> Vec<PromptAdapterProfile> {
    prompt_adapter_profiles()
}

pub fn generate_prompt_packages(
    state: &AppState,
    input: PromptGenerationInput,
) -> JoiResult<PromptGenerationResult> {
    let runtime_status = get_agent_runtime_status(state)?;
    let db = lock_db(state)?;
    generate_prompt_packages_service(
        &Repository::new(db.connection()),
        input,
        runtime_status.hermes_version,
    )
}

pub fn list_prompt_packages(
    state: &AppState,
    project_id: String,
) -> JoiResult<Vec<PromptPackageView>> {
    let db = lock_db(state)?;
    Repository::new(db.connection())
        .list_prompt_packages(&project_id)?
        .into_iter()
        .map(prompt_package_view)
        .collect::<Vec<_>>()
        .pipe(Ok)
}

pub fn update_prompt_package(
    state: &AppState,
    input: PromptPackageUpdateInput,
) -> JoiResult<PromptPackageView> {
    let db = lock_db(state)?;
    let package = Repository::new(db.connection()).update_prompt_package(PromptPackageUpdate {
        id: input.id,
        prompt_text: input.prompt_text,
        negative_prompt: input.negative_prompt,
        parameters_json: input.parameters_json,
        is_locked: input.is_locked,
    })?;
    Ok(prompt_package_view(package))
}
```

If `.pipe(Ok)` is not already in scope, use:

```rust
let packages = Repository::new(db.connection())
    .list_prompt_packages(&project_id)?
    .into_iter()
    .map(prompt_package_view)
    .collect::<Vec<_>>();
Ok(packages)
```

- [ ] **Step 3: Register Tauri commands**

In `src-tauri/src/lib.rs`, add to `tauri::generate_handler!`:

```rust
commands::joi_get_prompt_adapter_profiles,
commands::joi_generate_prompt_packages,
commands::joi_list_prompt_packages,
commands::joi_update_prompt_package,
```

- [ ] **Step 4: Run command tests**

```powershell
cd src-tauri
cargo test --test commands -- --nocapture
```

Expected:

- Command tests pass.

- [ ] **Step 5: Commit commands**

```powershell
git add src-tauri/src/commands.rs src-tauri/src/lib.rs src-tauri/tests/commands.rs
git commit -m "feat: expose Joi 0.17 prompt adapter commands"
```

### Task 4: Frontend Prompts Workspace

**Files:**

- Modify: `src/types/joi.ts`
- Modify: `src/api/joiApi.ts`
- Modify: `src/App.tsx`
- Modify: `src/components/ProjectWorkspace.tsx`
- Create: `src/components/PromptWorkspace.tsx`
- Modify: `src/styles.css`
- Test: `src/App.test.tsx`

- [ ] **Step 1: Add API wrappers**

In `src/api/joiApi.ts`, add imports and wrappers:

```ts
export function getPromptAdapterProfiles(): Promise<PromptAdapterProfile[]> {
  return invoke<PromptAdapterProfile[]>("joi_get_prompt_adapter_profiles");
}

export function generatePromptPackages(input: PromptGenerationInput): Promise<PromptGenerationResult> {
  return invoke<PromptGenerationResult>("joi_generate_prompt_packages", { input });
}

export function listPromptPackages(projectId: string): Promise<PromptPackageView[]> {
  return invoke<PromptPackageView[]>("joi_list_prompt_packages", { project_id: projectId });
}

export function updatePromptPackage(input: PromptPackageUpdateInput): Promise<PromptPackageView> {
  return invoke<PromptPackageView>("joi_update_prompt_package", { input });
}
```

- [ ] **Step 2: Add failing UI test**

In `src/App.test.tsx`, add mock fixture:

```ts
const mockPromptProfiles = [
  {
    id: "jimeng_video",
    display_name: "Jimeng Video",
    modality: "video",
    default_negative_prompt: "low resolution, distorted garment shape",
    required_fields: ["subject", "scene", "action", "camera", "material", "lighting", "style"],
  },
  {
    id: "grok_video",
    display_name: "Grok Video",
    modality: "video",
    default_negative_prompt: "low resolution, distorted garment shape",
    required_fields: ["subject", "scene", "action", "camera", "material", "lighting", "style"],
  },
  {
    id: "banana_2_image",
    display_name: "Banana 2 Image",
    modality: "image",
    default_negative_prompt: "distorted hands, incorrect garment construction",
    required_fields: ["subject", "scene", "garment", "material", "lighting", "style"],
  },
  {
    id: "jimeng_image",
    display_name: "Jimeng Image",
    modality: "image",
    default_negative_prompt: "distorted hands, incorrect garment construction",
    required_fields: ["subject", "scene", "garment", "material", "lighting", "style"],
  },
  {
    id: "gpt_image_2",
    display_name: "GPT Image 2",
    modality: "image",
    default_negative_prompt: "distorted hands, incorrect garment construction",
    required_fields: ["subject", "scene", "garment", "material", "lighting", "style"],
  },
];
```

Add mock command cases:

```ts
case "joi_get_prompt_adapter_profiles":
  return Promise.resolve(mockPromptProfiles);
case "joi_list_prompt_packages":
  return Promise.resolve([]);
case "joi_generate_prompt_packages":
  return Promise.resolve(mockPromptGenerationResult(args?.input?.target_platforms as string[]));
case "joi_update_prompt_package":
  return Promise.resolve({
    ...mockPromptGenerationResult(["gpt_image_2"]).packages[0],
    package: {
      ...mockPromptGenerationResult(["gpt_image_2"]).packages[0].package,
      id: args?.input?.id,
      prompt_text: args?.input?.prompt_text,
      negative_prompt: args?.input?.negative_prompt,
      is_locked: args?.input?.is_locked,
    },
  });
```

Add test:

```ts
test("generates edits and copies prompt packages", async () => {
  const writeText = vi.fn().mockResolvedValue(undefined);
  Object.assign(navigator, { clipboard: { writeText } });
  render(<App />);

  await screen.findByRole("heading", { name: "Spring Drop Film" });
  fireEvent.click(screen.getByRole("button", { name: "Prompts" }));

  expect(await screen.findByText("Jimeng Video")).toBeInTheDocument();
  fireEvent.click(screen.getByLabelText("Shot 1"));
  fireEvent.change(screen.getByLabelText("Prompt direction"), {
    target: { value: "Keep output production-ready." },
  });
  fireEvent.click(screen.getByRole("button", { name: /generate video prompts/i }));

  await waitFor(() => {
    expect(invokeMock).toHaveBeenCalledWith("joi_generate_prompt_packages", {
      input: expect.objectContaining({
        project_id: "project-1",
        shot_ids: ["shot-1"],
        target_platforms: ["jimeng_video", "grok_video"],
        user_direction: "Keep output production-ready.",
      }),
    });
  });
  expect(await screen.findByText("Jimeng video prompt")).toBeInTheDocument();

  fireEvent.change(screen.getByLabelText("Image brief"), {
    target: { value: "Full-body ecommerce model photo, warm studio." },
  });
  fireEvent.click(screen.getByRole("button", { name: /generate image prompts/i }));
  await waitFor(() => {
    expect(invokeMock).toHaveBeenCalledWith("joi_generate_prompt_packages", {
      input: expect.objectContaining({
        shot_ids: [],
        image_brief: "Full-body ecommerce model photo, warm studio.",
        target_platforms: ["banana_2_image", "jimeng_image", "gpt_image_2"],
      }),
    });
  });
  expect(await screen.findByText("GPT Image 2 prompt")).toBeInTheDocument();

  fireEvent.change(screen.getAllByLabelText("Prompt text")[0], {
    target: { value: "Edited prompt package text." },
  });
  fireEvent.click(screen.getAllByRole("button", { name: /save prompt/i })[0]);
  await waitFor(() => {
    expect(invokeMock).toHaveBeenCalledWith("joi_update_prompt_package", expect.objectContaining({
      input: expect.objectContaining({
        prompt_text: "Edited prompt package text.",
      }),
    }));
  });

  fireEvent.click(screen.getAllByRole("button", { name: /copy prompt/i })[0]);
  await waitFor(() => {
    expect(writeText).toHaveBeenCalled();
  });
});
```

Run:

```powershell
npm test -- src/App.test.tsx
```

Expected RED:

- Prompt API wrappers, prompt state, and Prompt workspace do not exist.

- [ ] **Step 3: Create `PromptWorkspace.tsx`**

Create `src/components/PromptWorkspace.tsx` with:

```ts
export type PromptDraft = {
  selected_video_platforms: string[];
  selected_image_platforms: string[];
  selected_shot_ids: string[];
  image_brief: string;
  user_direction: string;
};
```

Props:

```ts
type PromptWorkspaceProps = {
  adapterProfiles: PromptAdapterProfile[];
  generatingPrompts: boolean;
  onCopyPrompt: (copyText: string, packageId: string) => void;
  onPromptDraftChange: (draft: PromptDraft) => void;
  onSubmitImagePrompts: () => void;
  onSubmitVideoPrompts: () => void;
  onUpdatePromptPackage: (input: PromptPackageUpdateInput) => void;
  promptDraft: PromptDraft;
  promptPackages: PromptPackageView[];
  savingPromptId: string | null;
  selectedProject: Project | null;
  storyboards: StoryboardWithShots[];
};
```

Render:

- `Prompt direction` textarea
- video platform checkboxes for `Jimeng Video` and `Grok Video`
- image platform checkboxes for `Banana 2 Image`, `Jimeng Image`, `GPT Image 2`
- shot checkboxes from the latest storyboard, labels `Shot 1`, `Shot 2`, etc.
- `Generate Video Prompts` button
- `Image brief` textarea
- `Generate Image Prompts` button
- prompt package editor list:
  - adapter display name
  - modality
  - shot source or image brief source
  - missing fields
  - `Prompt text` textarea
  - `Negative prompt` textarea
  - `Lock prompt` checkbox
  - `Save Prompt` button
  - `Copy Prompt` button

Use shallow panels and rows. Do not nest card-like containers inside other card-like containers.

- [ ] **Step 4: Wire App state**

In `src/App.tsx`, import prompt API wrappers and types.

Add state:

```ts
const emptyPromptDraft: PromptDraft = {
  selected_video_platforms: ["jimeng_video", "grok_video"],
  selected_image_platforms: ["banana_2_image", "jimeng_image", "gpt_image_2"],
  selected_shot_ids: [],
  image_brief: "",
  user_direction: "",
};
const [adapterProfiles, setAdapterProfiles] = useState<PromptAdapterProfile[]>([]);
const [generatingPrompts, setGeneratingPrompts] = useState(false);
const [promptDraft, setPromptDraft] = useState<PromptDraft>(emptyPromptDraft);
const [promptPackages, setPromptPackages] = useState<PromptPackageView[]>([]);
const [savingPromptId, setSavingPromptId] = useState<string | null>(null);
const [copiedPromptId, setCopiedPromptId] = useState<string | null>(null);
```

Update `loadInitialState` to include `getPromptAdapterProfiles()`.

Update `refreshProjectState` to include `listPromptPackages(projectId)`.

When project changes, reset `promptDraft` to:

```ts
setPromptDraft({
  ...emptyPromptDraft,
  selected_shot_ids: latestShotIdsFrom(storyboards),
});
```

Because `storyboards` are loaded asynchronously, also add an effect:

```ts
useEffect(() => {
  const latest = storyboards[storyboards.length - 1];
  if (!latest || promptDraft.selected_shot_ids.length > 0) {
    return;
  }
  setPromptDraft((draft) => ({
    ...draft,
    selected_shot_ids: latest.shots.map((shot) => shot.id),
  }));
}, [storyboards, promptDraft.selected_shot_ids.length]);
```

Add handlers:

```ts
async function submitVideoPrompts() {
  if (!selectedProject) {
    setError("Select a project before generating prompts.");
    return;
  }
  if (promptDraft.selected_shot_ids.length === 0) {
    setError("Select at least one storyboard shot for video prompts.");
    return;
  }
  await submitPromptGeneration(promptDraft.selected_video_platforms, promptDraft.selected_shot_ids, "");
}

async function submitImagePrompts() {
  if (!selectedProject) {
    setError("Select a project before generating prompts.");
    return;
  }
  if (!promptDraft.image_brief.trim()) {
    setError("Image brief is required for image prompts.");
    return;
  }
  await submitPromptGeneration(promptDraft.selected_image_platforms, [], promptDraft.image_brief);
}

async function submitPromptGeneration(targetPlatforms: string[], shotIds: string[], imageBrief: string) {
  if (!selectedProject) return;
  try {
    setGeneratingPrompts(true);
    setError(null);
    const result = await generatePromptPackages({
      project_id: selectedProject.id,
      shot_ids: shotIds,
      image_brief: imageBrief,
      target_platforms: targetPlatforms,
      user_direction: promptDraft.user_direction,
    });
    setPromptPackages(result.packages);
    await refreshProjectState(selectedProject.id);
    setAgentRuns((runs) => [
      { run: result.agent_run, events: result.agent_events },
      ...runs.filter((item) => item.run.id !== result.agent_run.id),
    ]);
    setActivityLog((entries) => [
      ...entries,
      `Generated ${result.packages.length} prompt package(s).`,
    ]);
  } catch (submitError) {
    setError(formatError(submitError));
  } finally {
    setGeneratingPrompts(false);
  }
}
```

Add prompt update:

```ts
async function handleUpdatePromptPackage(input: PromptPackageUpdateInput) {
  try {
    setSavingPromptId(input.id);
    setError(null);
    const updated = await updatePromptPackage(input);
    setPromptPackages((packages) =>
      packages.map((item) => (item.package.id === updated.package.id ? updated : item)),
    );
    if (selectedProject) {
      await refreshProjectState(selectedProject.id);
    }
    setActivityLog((entries) => [...entries, `Updated prompt ${updated.package.id}.`]);
  } catch (submitError) {
    setError(formatError(submitError));
  } finally {
    setSavingPromptId(null);
  }
}
```

Add copy:

```ts
async function handleCopyPrompt(copyText: string, packageId: string) {
  try {
    await navigator.clipboard.writeText(copyText);
    setCopiedPromptId(packageId);
    setActivityLog((entries) => [...entries, `Copied prompt ${packageId}.`]);
  } catch {
    setError("Clipboard copy failed.");
  }
}
```

- [ ] **Step 5: Render Prompts tab**

In `src/components/ProjectWorkspace.tsx`:

- import `PromptWorkspace` and `PromptDraft`
- add props for prompt state and handlers
- render:

```tsx
{activeTab === "Prompts" ? (
  <PromptWorkspace
    adapterProfiles={adapterProfiles}
    generatingPrompts={generatingPrompts}
    onCopyPrompt={onCopyPrompt}
    onPromptDraftChange={onPromptDraftChange}
    onSubmitImagePrompts={onSubmitImagePrompts}
    onSubmitVideoPrompts={onSubmitVideoPrompts}
    onUpdatePromptPackage={onUpdatePromptPackage}
    promptDraft={promptDraft}
    promptPackages={promptPackages}
    savingPromptId={savingPromptId}
    selectedProject={selectedProject}
    storyboards={storyboards}
  />
) : null}
```

Remove `Prompts` from the `EmptyState` fallback list.

- [ ] **Step 6: Add focused CSS**

In `src/styles.css`, add:

```css
.prompt-layout {
  display: grid;
  gap: 16px;
}

.prompt-toolbar {
  display: grid;
  gap: 12px;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.prompt-platform-grid,
.prompt-shot-grid {
  display: grid;
  gap: 8px;
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
}

.prompt-package-list {
  display: grid;
  gap: 12px;
}

.prompt-package-row {
  border: 1px solid var(--border);
  border-radius: 8px;
  display: grid;
  gap: 12px;
  padding: 14px;
}

.prompt-editor-grid {
  display: grid;
  gap: 12px;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
}

.missing-field-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
```

Mobile rule:

```css
@media (max-width: 760px) {
  .prompt-toolbar,
  .prompt-editor-grid {
    grid-template-columns: 1fr;
  }
}
```

- [ ] **Step 7: Run frontend tests and build**

```powershell
npm test
npm run build
```

Expected:

- Frontend tests and build pass.

- [ ] **Step 8: Commit frontend**

```powershell
git add src/types/joi.ts src/api/joiApi.ts src/App.tsx src/components/ProjectWorkspace.tsx src/components/PromptWorkspace.tsx src/styles.css src/App.test.tsx
git commit -m "feat: add Joi 0.17 prompts workspace"
```

### Task 5: Smoke, Review, Merge, Push

**Files:**

- Create: `docs/superpowers/reports/joi-0.17-multi-model-prompt-adapters-smoke-test.md`

- [ ] **Step 1: Run full verification**

```powershell
npm test
npm run build
cd src-tauri
cargo test
cargo test --test prompt_adapter -- --nocapture
cargo test --test commands -- --nocapture
```

Expected:

- Frontend tests pass.
- Frontend production build passes.
- Rust tests pass.
- Prompt adapter focused tests pass.
- Command focused tests pass.

- [ ] **Step 2: Browser smoke**

Start:

```powershell
npm run dev -- --host 127.0.0.1 --port 1420
```

Verify with a Tauri invoke mock in a normal browser:

- Prompts tab renders platform selectors.
- Video generation can select `Shot 1`.
- `Generate Video Prompts` invokes `joi_generate_prompt_packages` with `jimeng_video` and `grok_video`.
- Generated video packages are visible and show prompt text and negative prompt.
- Image brief generation invokes `joi_generate_prompt_packages` with `banana_2_image`, `jimeng_image`, and `gpt_image_2`.
- Generated image packages are visible.
- Missing-field warnings render when present.
- Edited prompt text can be saved through `joi_update_prompt_package`.
- Copy prompt calls `navigator.clipboard.writeText`.
- Desktop 1440x900 has no horizontal overflow.
- Mobile 390x844 has no horizontal overflow.

Normal browser limitation:

- A normal browser cannot call native Tauri commands; command integration is covered by Rust and React tests. Browser smoke may use a Tauri invoke mock.

- [ ] **Step 3: Write smoke report**

Create `docs/superpowers/reports/joi-0.17-multi-model-prompt-adapters-smoke-test.md` with:

- verification commands run
- browser viewports checked
- prompt platforms covered
- acceptance checklist
- known limitations

- [ ] **Step 4: Commit smoke report**

```powershell
git add docs/superpowers/reports/joi-0.17-multi-model-prompt-adapters-smoke-test.md
git commit -m "test: add Joi 0.17 prompt adapter smoke report"
```

- [ ] **Step 5: Merge to main**

From the main workspace:

```powershell
git status --short --branch
git merge --ff-only codex/joi-0.17-prompt-adapters
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
git worktree remove --force "D:\Software Project\Joi-agent\.worktrees\joi-0.17-prompt-adapters"
git worktree prune
git branch -d codex/joi-0.17-prompt-adapters
```

## Acceptance Criteria

0.17 is complete only when:

- `prompt_packages` can store shot-bound video packages and project-bound image packages.
- Existing shot-bound prompt packages still satisfy project/shot integrity.
- Video prompts can be generated for Jimeng and Grok from selected storyboard shots.
- Image prompts can be generated for Banana 2, Jimeng Image, and GPT Image 2 from a project image brief.
- Prompt packages include prompt text, negative prompt, adapter profile metadata, source metadata, completeness metadata, and copy text.
- Prompt completeness warnings identify missing subject, scene, action, camera, material, lighting, style, or garment fields without blocking generation.
- User can edit and save prompt text and negative prompt.
- User can copy prompt text from the Prompts workspace.
- Prompt generation creates Agent run/events.
- Prompt packages appear in snapshots through existing snapshot support.
- Tests cover schema migration, repository, service, commands, and frontend flow.
- Browser smoke report is written.
- Changes are merged to `main` and pushed to GitHub.

## Risks And Mitigations

### Risk: Optional `shot_id` Breaks Existing Shot-Bound Integrity

Mitigation:

- Keep triggers for non-null `shot_id`.
- Add migration tests for project-bound image prompts and cross-project shot rejection.
- Repository rejects video prompts without `shot_id`.

### Risk: Prompt Output Feels Too Generic

Mitigation:

- Prompt adapter consumes 0.16 structured shot fields instead of freeform storyboard text.
- Image prompts combine product understanding, creative direction, accepted memory, and image brief.
- Missing-field warnings make weak context visible to the user.

### Risk: Provider Capabilities Change

Mitigation:

- 0.17 stores adapter profiles as local configurable output templates.
- No external API contract is hardcoded.
- Prompt parameters are stored in `parameters_json` with a format version, so later adapter revisions can be introduced without changing the core table again.

### Risk: Frontend Prompt Editor Becomes Crowded

Mitigation:

- Keep controls grouped by generation mode.
- Use shallow prompt rows rather than nested cards.
- Use responsive grid constraints and browser smoke at 1440x900 and 390x844.

## Handoff To 0.18

0.18 delivery reports should consume:

- `PromptPackage.platform`
- `PromptPackage.modality`
- `PromptPackage.prompt_text`
- `PromptPackage.negative_prompt`
- `PromptPackage.parameters_json.adapter_display_name`
- `PromptPackage.parameters_json.source_type`
- `PromptPackage.parameters_json.missing_fields`

0.18 should include prompt packages in the project delivery report and export preview.

## Self-Review

- Spec coverage: This plan covers 0.17 roadmap scope: adapter architecture, platform templates, video prompt generation, image prompt generation, negative prompts, package editor, per-shot and batch generation, validation rules, tests, smoke, merge, and push.
- Placeholder scan: No task contains unresolved placeholder instructions. File paths, command names, type names, test names, adapter IDs, and expected outputs are concrete.
- Type consistency: `PromptPackage`, `PromptPackageCreate`, `PromptPackageUpdate`, `PromptGenerationInput`, `PromptPackageView`, `PromptGenerationResult`, and `PromptPackageUpdateInput` are used consistently across backend, commands, frontend, and tests.
