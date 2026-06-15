# Joi Agent 0.12 Brief And Material Understanding Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first usable content-input workflow: a user can enter a fashion advertising brief, product details, target settings, and reference materials, then generate and save structured project understanding.

**Architecture:** 0.12 uses the existing Joi local data layer instead of adding an Agent runtime or LLM dependency. The backend exposes command wrappers around existing `product_understandings`, `creative_directions`, and `assets` persistence, plus a deterministic `understanding` service that turns form input into a saved understanding record and missing-question list. The frontend adds a data-backed Brief workspace and reference material panel on top of the 0.11 shell.

**Tech Stack:** React 19, TypeScript, Vitest, Tauri commands, Rust, rusqlite, serde/serde_json.

---

## Product Outcome

After 0.12, a user can:

- Select a brand and project in the 0.11 workspace.
- Open the Brief tab and edit:
  - project brief
  - product name
  - product category
  - target audience
  - target platforms
  - selling points
  - visual direction
  - constraints / forbidden items
  - duration
- Add link-based reference materials to the project.
- Generate a structured material understanding without leaving Joi.
- See and edit the saved product understanding.
- See missing information questions generated from incomplete inputs.
- Save a project snapshot that includes the generated understanding.

0.12 does not call external models. Its "generation" is deterministic structuring and question generation. This gives 0.13 Agent runtime a reliable project context to read.

## Scope

### In Scope

- Backend command surface for:
  - list product understandings
  - generate and save brief/material understanding
  - list creative directions
  - create link/reference assets
- Reuse existing tables:
  - `projects`
  - `assets`
  - `product_understandings`
  - `creative_directions`
  - `project_versions`
- Store 0.12 structured notes in `product_understandings.notes` as versioned JSON text.
- Frontend Brief tab with controlled form state.
- Frontend reference material form for link-style assets.
- Frontend result panel for structured understanding and missing questions.
- Frontend tests for brief generation and reference material creation.
- Rust tests for new command DTOs and generated understanding persistence.
- Browser smoke report for 0.12.

### Out Of Scope

- No web research.
- No Agent planner.
- No external LLM call.
- No native file picker.
- No direct upload to generation platforms.
- No storyboard or prompt generation.
- No schema migration framework.

## Key Design Decisions

### Decision 1: Use Existing Structured Tables

0.12 should not add a new database table. Current schema already has `product_understandings` and `creative_directions`, and project snapshots already include both. The implementation should expand repository input structs and commands so these tables can carry useful 0.12 data.

### Decision 2: Versioned Notes JSON

`product_understandings.notes` is a `TEXT` column. In 0.12, store JSON text with this shape:

```json
{
  "format_version": "joi.product_understanding_notes.v1",
  "brief_summary": "15 second spring outerwear launch film for short-video platforms.",
  "brand_summary": "Contemporary womenswear label with clean studio lighting.",
  "visual_direction": "Light movement, close fabric texture, confident model walk.",
  "target_platforms": ["jimeng_video", "grok_video"],
  "reference_asset_ids": ["asset-1"],
  "missing_questions": [
    "Which platform is the primary delivery target?",
    "Which garment feature must appear in the first three seconds?"
  ]
}
```

Frontend code should parse this JSON defensively. If parsing fails, show `notes` as raw text and do not crash.

### Decision 3: Deterministic Understanding Service

The generation service should produce useful structure from user input without pretending to be an LLM.

Rules:

- Split selling points and constraints by newline, comma, Chinese comma, semicolon, and Chinese semicolon.
- Trim empty values.
- Build `missing_questions` from missing fields.
- Use project and brand context in summaries.
- Persist one `ProductUnderstanding` record per generate action.
- Persist one `CreativeDirection` record only when `visual_direction` is non-empty.
- Return both saved records to the frontend.

## File Structure

### Backend

- Modify `src-tauri/src/models.rs`
  - No new persisted model is required.
  - Existing `ProductUnderstanding`, `CreativeDirection`, and `Asset` are reused.
- Modify `src-tauri/src/repositories.rs`
  - Expand `ProductUnderstandingCreate`.
  - Expand `CreativeDirectionCreate`.
  - Add `create_reference_asset`.
  - Keep list methods unchanged.
- Create `src-tauri/src/understanding.rs`
  - Owns deterministic brief/material understanding generation.
  - Defines command-facing input/result structs.
  - Builds `notes` JSON.
  - Computes missing questions.
- Modify `src-tauri/src/commands.rs`
  - Add command DTOs and command functions:
    - `joi_generate_brief_understanding`
    - `joi_list_product_understandings`
    - `joi_list_creative_directions`
    - `joi_create_reference_asset`
- Modify `src-tauri/src/lib.rs`
  - Register new module and commands.
- Modify `src-tauri/tests/commands.rs`
  - Add command DTO round-trip test coverage.
  - Add generation persistence test coverage.
- Modify Rust integration tests:
  - `src-tauri/tests/structured_content_repository.rs`

### Frontend

- Modify `src/types/joi.ts`
  - Add `ProductUnderstanding`, `CreativeDirection`, `BriefUnderstandingInput`, `BriefUnderstandingResult`, `ReferenceAssetInput`.
- Modify `src/api/joiApi.ts`
  - Add wrappers for new commands.
- Modify `src/App.tsx`
  - Add 0.12 state:
    - `briefDraft`
    - `productUnderstandings`
    - `creativeDirections`
    - `understandingResult`
    - `referenceAssetDraft`
  - Load understandings and creative directions when project changes.
  - Refresh assets after reference material creation.
- Modify `src/components/ProjectWorkspace.tsx`
  - Render a data-backed Brief panel instead of the old empty state.
  - Render current understanding and missing questions.
  - Render reference material form inside the Brief tab or Assets tab.
- Create focused Brief workspace components:
  - `src/components/BriefWorkspace.tsx`
  - `src/components/ReferenceMaterialPanel.tsx`
  - `src/components/UnderstandingResultPanel.tsx`
- Modify `src/App.test.tsx`
  - Add tests for generating understanding and creating reference assets.
- Modify `docs/superpowers/reports/`
  - Add `joi-0.12-brief-material-understanding-smoke-test.md`.

## Backend Contract

### Command: `joi_generate_brief_understanding`

Input:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BriefUnderstandingInput {
    pub project_id: String,
    pub brief_text: String,
    pub product_name: String,
    pub category: String,
    pub audience: String,
    pub target_platforms: Vec<String>,
    pub selling_points_text: String,
    pub visual_direction: String,
    pub constraints_text: String,
    pub reference_asset_ids: Vec<String>,
}
```

Result:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefUnderstandingResult {
    pub product_understanding: ProductUnderstanding,
    pub creative_direction: Option<CreativeDirection>,
    pub brief_summary: String,
    pub brand_summary: String,
    pub visual_direction: String,
    pub selling_points: Vec<String>,
    pub constraints: Vec<String>,
    pub missing_questions: Vec<String>,
}
```

Behavior:

- Validate that `project_id` exists.
- Load project and brand.
- Validate reference asset IDs belong to the same project.
- Require at least one of `brief_text`, `product_name`, `selling_points_text`, or `visual_direction` to be non-empty.
- Save a `ProductUnderstanding`.
- Save a `CreativeDirection` when `visual_direction` is non-empty.
- Return the saved rows plus parsed summary fields.

### Command: `joi_list_product_understandings`

Input:

```rust
#[tauri::command(rename_all = "snake_case")]
pub fn joi_list_product_understandings(
    state: State<'_, AppState>,
    project_id: String,
) -> JoiResult<Vec<ProductUnderstanding>>
```

Behavior:

- Return all product understandings for the project ordered by `created_at ASC`.

### Command: `joi_list_creative_directions`

Input:

```rust
#[tauri::command(rename_all = "snake_case")]
pub fn joi_list_creative_directions(
    state: State<'_, AppState>,
    project_id: String,
) -> JoiResult<Vec<CreativeDirection>>
```

Behavior:

- Return all creative directions for the project ordered by `created_at ASC`.

### Command: `joi_create_reference_asset`

Input:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReferenceAssetInput {
    pub project_id: String,
    pub kind: String,
    pub display_name: String,
    pub source_uri: String,
}
```

Behavior:

- Validate project exists.
- Validate kind with `AssetKind`.
- Require non-empty display name and source URI.
- Persist an asset with:
  - `relative_path = ""`
  - `mime_type = "text/uri-list"`
  - `file_size_bytes = 0`
  - `sha256 = ""`
- Return the saved `Asset`.

## Understanding Rules

### `split_list_text`

Input:

```text
water-resistant cotton, oversized collar
soft structured silhouette；close fabric texture
```

Output:

```json
[
  "water-resistant cotton",
  "oversized collar",
  "soft structured silhouette",
  "close fabric texture"
]
```

### Missing Questions

Generate these exact questions when fields are blank:

- Missing `brief_text`: `What is the core campaign brief for this project?`
- Missing `product_name`: `Which product or collection should the content focus on?`
- Missing `category`: `What garment category should Joi optimize the visual language for?`
- Missing `audience`: `Who is the primary audience for this ad?`
- Missing `target_platforms`: `Which output platforms should this project target?`
- Missing selling points: `Which product selling points must be visible in the content?`
- Missing `visual_direction`: `What visual direction should guide scenes, lighting, and camera language?`
- Missing `reference_asset_ids`: `Which reference materials should Joi use as visual anchors?`

### Summary Construction

`brief_summary`:

- If `brief_text` is non-empty, use the trimmed brief.
- Otherwise use: `{project.title}: {project.advertising_goal}`.

`brand_summary`:

- If brand description is non-empty, use `{brand.name}: {brand.description}`.
- Otherwise use `{brand.name}`.

`visual_direction`:

- Use trimmed `visual_direction`.
- If blank, use `Brand-led visual direction pending user input.`.

## Implementation Tasks

### Task 1: Backend Generation Service

**Files:**

- Create: `src-tauri/src/understanding.rs`
- Modify: `src-tauri/src/repositories.rs`
- Test: `src-tauri/tests/structured_content_repository.rs`

- [ ] **Step 1: Write repository test for full product understanding fields**

Add this test to `src-tauri/tests/structured_content_repository.rs`:

```rust
#[test]
fn creates_product_understanding_with_full_material_context() {
    let app = TestApp::new();
    let repo = Repository::new(app.db.connection());
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
            advertising_goal: "Launch a 15 second outerwear ad".to_string(),
            duration_seconds: 15,
        })
        .unwrap();

    let understanding = repo
        .create_product_understanding(ProductUnderstandingCreate {
            project_id: project.id.clone(),
            product_name: "Lightweight trench".to_string(),
            category: "outerwear".to_string(),
            audience: "urban commuters".to_string(),
            selling_points: vec!["water-resistant cotton".to_string(), "soft structure".to_string()],
            constraints: vec!["avoid heavy winter styling".to_string()],
            notes: "{\"format_version\":\"joi.product_understanding_notes.v1\"}".to_string(),
        })
        .unwrap();

    assert_eq!(understanding.product_name, "Lightweight trench");
    assert_eq!(understanding.category, "outerwear");
    assert_eq!(understanding.audience, "urban commuters");
    assert_eq!(understanding.selling_points_json, serde_json::json!(["water-resistant cotton", "soft structure"]));
    assert_eq!(understanding.constraints_json, serde_json::json!(["avoid heavy winter styling"]));
    assert!(understanding.notes.contains("joi.product_understanding_notes.v1"));
}
```

- [ ] **Step 2: Run the repository test and confirm RED**

Run:

```powershell
cd src-tauri
cargo test --test structured_content_repository creates_product_understanding_with_full_material_context -- --nocapture
```

Expected:

- Fails to compile because `ProductUnderstandingCreate` does not yet have `audience`, `selling_points`, `constraints`, or `notes`.

- [ ] **Step 3: Expand repository input and implementation**

Change `ProductUnderstandingCreate` in `src-tauri/src/repositories.rs` to:

```rust
#[derive(Debug, Clone)]
pub struct ProductUnderstandingCreate {
    pub project_id: String,
    pub product_name: String,
    pub category: String,
    pub audience: String,
    pub selling_points: Vec<String>,
    pub constraints: Vec<String>,
    pub notes: String,
}
```

Change `create_product_understanding` assignment to:

```rust
let understanding = ProductUnderstanding {
    id: new_id(),
    project_id: input.project_id,
    product_name: input.product_name.trim().to_string(),
    category: input.category.trim().to_string(),
    audience: input.audience.trim().to_string(),
    selling_points_json: json!(input.selling_points),
    constraints_json: json!(input.constraints),
    notes: input.notes,
    created_at: now,
    updated_at: now,
};
```

- [ ] **Step 4: Run the repository test and confirm GREEN**

Run:

```powershell
cd src-tauri
cargo test --test structured_content_repository creates_product_understanding_with_full_material_context -- --nocapture
```

Expected:

- Test passes.

- [ ] **Step 5: Add understanding service tests**

Create `src-tauri/src/understanding.rs` with unit tests first:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_selling_points_and_constraints_from_mixed_separators() {
        assert_eq!(
            split_list_text("water-resistant cotton, oversized collar\nsoft structure；close texture"),
            vec![
                "water-resistant cotton".to_string(),
                "oversized collar".to_string(),
                "soft structure".to_string(),
                "close texture".to_string(),
            ]
        );
    }

    #[test]
    fn asks_missing_questions_for_blank_inputs() {
        let questions = missing_questions(&BriefUnderstandingInput {
            project_id: "project-1".to_string(),
            brief_text: "".to_string(),
            product_name: "".to_string(),
            category: "".to_string(),
            audience: "".to_string(),
            target_platforms: Vec::new(),
            selling_points_text: "".to_string(),
            visual_direction: "".to_string(),
            constraints_text: "".to_string(),
            reference_asset_ids: Vec::new(),
        });

        assert!(questions.contains(&"What is the core campaign brief for this project?".to_string()));
        assert!(questions.contains(&"Which product or collection should the content focus on?".to_string()));
        assert!(questions.contains(&"Which reference materials should Joi use as visual anchors?".to_string()));
    }
}
```

- [ ] **Step 6: Run service tests and confirm RED**

Run:

```powershell
cd src-tauri
cargo test understanding::tests -- --nocapture
```

Expected:

- Fails to compile because `BriefUnderstandingInput`, `split_list_text`, and `missing_questions` do not exist.

- [ ] **Step 7: Implement `src-tauri/src/understanding.rs`**

Implement:

```rust
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::{JoiError, JoiResult};
use crate::models::{CreativeDirection, ProductUnderstanding};
use crate::repositories::{CreativeDirectionCreate, ProductUnderstandingCreate, Repository};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BriefUnderstandingInput {
    pub project_id: String,
    pub brief_text: String,
    pub product_name: String,
    pub category: String,
    pub audience: String,
    pub target_platforms: Vec<String>,
    pub selling_points_text: String,
    pub visual_direction: String,
    pub constraints_text: String,
    pub reference_asset_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefUnderstandingResult {
    pub product_understanding: ProductUnderstanding,
    pub creative_direction: Option<CreativeDirection>,
    pub brief_summary: String,
    pub brand_summary: String,
    pub visual_direction: String,
    pub selling_points: Vec<String>,
    pub constraints: Vec<String>,
    pub missing_questions: Vec<String>,
}

pub fn split_list_text(value: &str) -> Vec<String> {
    value
        .split(|character| matches!(character, '\n' | ',' | '，' | ';' | '；'))
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}
```

Then add `missing_questions` using the exact question strings from this plan.

Then add:

```rust
pub fn generate_brief_understanding(
    repo: &Repository<'_>,
    input: BriefUnderstandingInput,
) -> JoiResult<BriefUnderstandingResult> {
    if input.brief_text.trim().is_empty()
        && input.product_name.trim().is_empty()
        && input.selling_points_text.trim().is_empty()
        && input.visual_direction.trim().is_empty()
    {
        return Err(JoiError::Validation(
            "brief, product name, selling points, or visual direction is required".to_string(),
        ));
    }

    let project = repo.get_project(&input.project_id)?;
    let brand = repo.get_brand(&project.brand_id)?;
    for asset_id in &input.reference_asset_ids {
        let assets = repo.list_assets(&project.id)?;
        if !assets.iter().any(|asset| &asset.id == asset_id) {
            return Err(JoiError::Validation(format!(
                "reference asset {asset_id} does not belong to project"
            )));
        }
    }

    let selling_points = split_list_text(&input.selling_points_text);
    let constraints = split_list_text(&input.constraints_text);
    let brief_summary = if input.brief_text.trim().is_empty() {
        format!("{}: {}", project.title, project.advertising_goal)
    } else {
        input.brief_text.trim().to_string()
    };
    let brand_summary = if brand.description.trim().is_empty() {
        brand.name.clone()
    } else {
        format!("{}: {}", brand.name, brand.description)
    };
    let visual_direction = if input.visual_direction.trim().is_empty() {
        "Brand-led visual direction pending user input.".to_string()
    } else {
        input.visual_direction.trim().to_string()
    };
    let missing_questions = missing_questions(&input);
    let notes = json!({
        "format_version": "joi.product_understanding_notes.v1",
        "brief_summary": brief_summary,
        "brand_summary": brand_summary,
        "visual_direction": visual_direction,
        "target_platforms": input.target_platforms,
        "reference_asset_ids": input.reference_asset_ids,
        "missing_questions": missing_questions,
    })
    .to_string();

    let product_understanding = repo.create_product_understanding(ProductUnderstandingCreate {
        project_id: project.id.clone(),
        product_name: input.product_name,
        category: input.category,
        audience: input.audience,
        selling_points: selling_points.clone(),
        constraints: constraints.clone(),
        notes,
    })?;
    let creative_direction = if input.visual_direction.trim().is_empty() {
        None
    } else {
        Some(repo.create_creative_direction(CreativeDirectionCreate {
            project_id: project.id,
            title: "Initial visual direction".to_string(),
            concept: visual_direction.clone(),
            tone: "user-defined".to_string(),
            visual_style: visual_direction.clone(),
            scene_direction: String::new(),
            rationale: "Generated from 0.12 brief and material understanding input.".to_string(),
        })?)
    };

    Ok(BriefUnderstandingResult {
        product_understanding,
        creative_direction,
        brief_summary,
        brand_summary,
        visual_direction,
        selling_points,
        constraints,
        missing_questions,
    })
}
```

- [ ] **Step 8: Run service and repository tests**

Run:

```powershell
cd src-tauri
cargo test understanding::tests -- --nocapture
cargo test --test structured_content_repository -- --nocapture
```

Expected:

- All targeted tests pass.

### Task 2: Backend Commands

**Files:**

- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/commands.rs`

- [ ] **Step 1: Write failing command test**

Add this test to `src-tauri/tests/commands.rs`:

```rust
#[test]
fn generates_brief_understanding_and_lists_saved_records() {
    let app = TestApp::new();
    let brand = create_brand(
        &app.state,
        BrandInput {
            name: "Atelier Joi".to_string(),
            description: "Contemporary womenswear with clean studio lighting".to_string(),
        },
    )
    .unwrap();
    let project = create_project(
        &app.state,
        ProjectInput {
            brand_id: brand.id,
            title: "Spring Drop Film".to_string(),
            advertising_goal: "Launch awareness".to_string(),
            duration_seconds: 15,
        },
    )
    .unwrap();

    let result = generate_brief_understanding(
        &app.state,
        BriefUnderstandingInput {
            project_id: project.id.clone(),
            brief_text: "15 second outerwear launch ad".to_string(),
            product_name: "Lightweight trench".to_string(),
            category: "outerwear".to_string(),
            audience: "urban commuters".to_string(),
            target_platforms: vec!["jimeng_video".to_string(), "grok_video".to_string()],
            selling_points_text: "water-resistant cotton, soft structure".to_string(),
            visual_direction: "clean studio walk with close fabric texture".to_string(),
            constraints_text: "avoid heavy winter styling".to_string(),
            reference_asset_ids: Vec::new(),
        },
    )
    .unwrap();

    assert_eq!(result.product_understanding.product_name, "Lightweight trench");
    assert_eq!(result.selling_points, vec!["water-resistant cotton", "soft structure"]);
    assert_eq!(result.missing_questions, Vec::<String>::new());

    let understandings = list_product_understandings(&app.state, project.id.clone()).unwrap();
    assert_eq!(understandings.len(), 1);
    let directions = list_creative_directions(&app.state, project.id).unwrap();
    assert_eq!(directions.len(), 1);
}
```

- [ ] **Step 2: Run command test and confirm RED**

Run:

```powershell
cd src-tauri
cargo test --test commands generates_brief_understanding_and_lists_saved_records -- --nocapture
```

Expected:

- Fails to compile because command helper functions and DTO imports do not exist.

- [ ] **Step 3: Add command helpers and Tauri commands**

In `src-tauri/src/commands.rs`, import:

```rust
use crate::models::{Asset, Brand, CreativeDirection, MemoryEntry, Product, ProductUnderstanding, ProjectVersion};
use crate::understanding::{
    generate_brief_understanding as generate_understanding,
    BriefUnderstandingInput,
    BriefUnderstandingResult,
};
```

Add:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReferenceAssetInput {
    pub project_id: String,
    pub kind: String,
    pub display_name: String,
    pub source_uri: String,
}
```

Add Tauri commands and testable helpers:

```rust
#[tauri::command(rename_all = "snake_case")]
pub fn joi_generate_brief_understanding(
    state: State<'_, AppState>,
    input: BriefUnderstandingInput,
) -> JoiResult<BriefUnderstandingResult> {
    generate_brief_understanding(state.inner(), input)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_list_product_understandings(
    state: State<'_, AppState>,
    project_id: String,
) -> JoiResult<Vec<ProductUnderstanding>> {
    list_product_understandings(state.inner(), project_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_list_creative_directions(
    state: State<'_, AppState>,
    project_id: String,
) -> JoiResult<Vec<CreativeDirection>> {
    list_creative_directions(state.inner(), project_id)
}
```

Helper implementations:

```rust
pub fn generate_brief_understanding(
    state: &AppState,
    input: BriefUnderstandingInput,
) -> JoiResult<BriefUnderstandingResult> {
    let db = lock_db(state)?;
    generate_understanding(&Repository::new(db.connection()), input)
}

pub fn list_product_understandings(
    state: &AppState,
    project_id: String,
) -> JoiResult<Vec<ProductUnderstanding>> {
    let db = lock_db(state)?;
    Repository::new(db.connection()).list_product_understandings(&project_id)
}

pub fn list_creative_directions(
    state: &AppState,
    project_id: String,
) -> JoiResult<Vec<CreativeDirection>> {
    let db = lock_db(state)?;
    Repository::new(db.connection()).list_creative_directions(&project_id)
}
```

- [ ] **Step 4: Register module and commands**

In `src-tauri/src/lib.rs`:

```rust
pub mod understanding;
```

Register:

```rust
commands::joi_generate_brief_understanding,
commands::joi_list_product_understandings,
commands::joi_list_creative_directions,
commands::joi_create_reference_asset,
```

- [ ] **Step 5: Add reference asset command**

Add command and helper:

```rust
#[tauri::command(rename_all = "snake_case")]
pub fn joi_create_reference_asset(
    state: State<'_, AppState>,
    input: ReferenceAssetInput,
) -> JoiResult<Asset> {
    create_reference_asset(state.inner(), input)
}

pub fn create_reference_asset(state: &AppState, input: ReferenceAssetInput) -> JoiResult<Asset> {
    let db = lock_db(state)?;
    Repository::new(db.connection()).create_reference_asset(AssetCreate {
        project_id: input.project_id,
        kind: input.kind,
        display_name: input.display_name,
        relative_path: String::new(),
        source_uri: input.source_uri,
        mime_type: "text/uri-list".to_string(),
        file_size_bytes: 0,
        sha256: String::new(),
    })
}
```

- [ ] **Step 6: Run backend tests**

Run:

```powershell
cd src-tauri
cargo test --test commands -- --nocapture
cargo test
```

Expected:

- Command tests pass.
- Full Rust test suite passes.

### Task 3: Frontend Types And API

**Files:**

- Modify: `src/types/joi.ts`
- Modify: `src/api/joiApi.ts`
- Test: `src/App.test.tsx`

- [ ] **Step 1: Write failing frontend command wiring test**

Add a test that clicks Brief, fills understanding fields, and expects:

```ts
expect(invokeMock).toHaveBeenCalledWith("joi_generate_brief_understanding", {
  input: {
    project_id: "project-1",
    brief_text: "15 second outerwear launch ad",
    product_name: "Lightweight trench",
    category: "outerwear",
    audience: "urban commuters",
    target_platforms: ["jimeng_video", "grok_video"],
    selling_points_text: "water-resistant cotton, soft structure",
    visual_direction: "clean studio walk with close fabric texture",
    constraints_text: "avoid heavy winter styling",
    reference_asset_ids: [],
  },
});
```

- [ ] **Step 2: Run frontend test and confirm RED**

Run:

```powershell
npm test -- src/App.test.tsx
```

Expected:

- Fails because the Brief tab has no form or generate action.

- [ ] **Step 3: Add frontend types**

Add to `src/types/joi.ts`:

```ts
export type ProductUnderstanding = {
  id: string;
  project_id: string;
  product_name: string;
  category: string;
  audience: string;
  selling_points_json: unknown;
  constraints_json: unknown;
  notes: string;
  created_at: string;
  updated_at: string;
};

export type CreativeDirection = {
  id: string;
  project_id: string;
  title: string;
  concept: string;
  tone: string;
  visual_style: string;
  scene_direction: string;
  rationale: string;
  created_at: string;
  updated_at: string;
};

export type BriefUnderstandingInput = {
  project_id: string;
  brief_text: string;
  product_name: string;
  category: string;
  audience: string;
  target_platforms: string[];
  selling_points_text: string;
  visual_direction: string;
  constraints_text: string;
  reference_asset_ids: string[];
};

export type BriefUnderstandingResult = {
  product_understanding: ProductUnderstanding;
  creative_direction: CreativeDirection | null;
  brief_summary: string;
  brand_summary: string;
  visual_direction: string;
  selling_points: string[];
  constraints: string[];
  missing_questions: string[];
};

export type ReferenceAssetInput = {
  project_id: string;
  kind: string;
  display_name: string;
  source_uri: string;
};
```

- [ ] **Step 4: Add API wrappers**

Add to `src/api/joiApi.ts`:

```ts
export function generateBriefUnderstanding(
  input: BriefUnderstandingInput,
): Promise<BriefUnderstandingResult> {
  return invoke<BriefUnderstandingResult>("joi_generate_brief_understanding", { input });
}

export function listProductUnderstandings(projectId: string): Promise<ProductUnderstanding[]> {
  return invoke<ProductUnderstanding[]>("joi_list_product_understandings", { project_id: projectId });
}

export function listCreativeDirections(projectId: string): Promise<CreativeDirection[]> {
  return invoke<CreativeDirection[]>("joi_list_creative_directions", { project_id: projectId });
}

export function createReferenceAsset(input: ReferenceAssetInput): Promise<Asset> {
  return invoke<Asset>("joi_create_reference_asset", { input });
}
```

- [ ] **Step 5: Run frontend build**

Run:

```powershell
npm run build
```

Expected:

- Build fails until the UI state and component props are added in Task 4.

### Task 4: Brief Workspace UI

**Files:**

- Modify: `src/App.tsx`
- Modify: `src/components/ProjectWorkspace.tsx`
- Create: `src/components/BriefWorkspace.tsx`
- Create: `src/components/ReferenceMaterialPanel.tsx`
- Create: `src/components/UnderstandingResultPanel.tsx`
- Test: `src/App.test.tsx`

- [ ] **Step 1: Add App state**

Add:

```ts
const emptyBriefDraft = {
  brief_text: "",
  product_name: "",
  category: "",
  audience: "",
  target_platforms_text: "",
  selling_points_text: "",
  visual_direction: "",
  constraints_text: "",
};

const emptyReferenceAssetDraft = {
  kind: "link",
  display_name: "",
  source_uri: "",
};
```

State:

```ts
const [briefDraft, setBriefDraft] = useState(emptyBriefDraft);
const [creativeDirections, setCreativeDirections] = useState<CreativeDirection[]>([]);
const [generatingUnderstanding, setGeneratingUnderstanding] = useState(false);
const [productUnderstandings, setProductUnderstandings] = useState<ProductUnderstanding[]>([]);
const [referenceAssetDraft, setReferenceAssetDraft] = useState(emptyReferenceAssetDraft);
const [understandingResult, setUnderstandingResult] = useState<BriefUnderstandingResult | null>(null);
```

- [ ] **Step 2: Load structured records on project change**

In `refreshProjectState`, add:

```ts
const [assetList, versionList, projectMemory, understandings, directions] = await Promise.all([
  listAssets(projectId),
  listProjectVersions(projectId),
  listMemoryEntries({ scope: "project", brand_id: null, project_id: projectId }),
  listProductUnderstandings(projectId),
  listCreativeDirections(projectId),
]);
setProductUnderstandings(understandings);
setCreativeDirections(directions);
```

- [ ] **Step 3: Add generate handler**

Add a small frontend helper in `src/App.tsx` near the draft constants:

```ts
function splitListText(value: string): string[] {
  return value
    .split(/[\n,，;；]/)
    .map((item) => item.trim())
    .filter(Boolean);
}
```

Add:

```ts
async function submitBriefUnderstanding() {
  if (!selectedProject) {
    setError("Select a project before generating understanding.");
    return;
  }
  try {
    setGeneratingUnderstanding(true);
    setError(null);
    const result = await generateBriefUnderstanding({
      project_id: selectedProject.id,
      brief_text: briefDraft.brief_text,
      product_name: briefDraft.product_name,
      category: briefDraft.category,
      audience: briefDraft.audience,
      target_platforms: splitListText(briefDraft.target_platforms_text),
      selling_points_text: briefDraft.selling_points_text,
      visual_direction: briefDraft.visual_direction,
      constraints_text: briefDraft.constraints_text,
      reference_asset_ids: assets.map((asset) => asset.id),
    });
    setUnderstandingResult(result);
    await refreshProjectState(selectedProject.id);
    setActivityLog((entries) => [...entries, "Generated project understanding."]);
  } catch (submitError) {
    setError(formatError(submitError));
  } finally {
    setGeneratingUnderstanding(false);
  }
}
```

- [ ] **Step 4: Add reference asset handler**

Add:

```ts
async function submitReferenceAsset() {
  if (!selectedProject) {
    setError("Select a project before adding a reference material.");
    return;
  }
  if (!referenceAssetDraft.display_name.trim() || !referenceAssetDraft.source_uri.trim()) {
    setError("Reference name and URI are required.");
    return;
  }
  try {
    setError(null);
    await createReferenceAsset({
      project_id: selectedProject.id,
      ...referenceAssetDraft,
    });
    setReferenceAssetDraft(emptyReferenceAssetDraft);
    await refreshProjectState(selectedProject.id);
    setActivityLog((entries) => [...entries, "Added reference material."]);
  } catch (submitError) {
    setError(formatError(submitError));
  }
}
```

- [ ] **Step 5: Render Brief tab**

`ProjectWorkspace` should render the Brief tab when `activeTab === "Brief"` with:

- `Project brief` textarea.
- `Product name` input.
- `Product category` input.
- `Audience` input.
- `Target platforms` input with comma-separated values such as `jimeng_video, grok_video`.
- `Selling points` textarea.
- `Visual direction` textarea.
- `Constraints` textarea.
- `Generate Understanding` button.
- Latest result section:
  - brief summary
  - brand summary
  - selling points
  - constraints
  - missing questions
- Reference material form:
  - reference name
  - source URI
  - Add Reference button
- Existing assets list scoped as references.

- [ ] **Step 6: Run frontend tests and build**

Run:

```powershell
npm test -- src/App.test.tsx
npm run build
```

Expected:

- Brief generation test passes.
- Build passes.

### Task 5: Snapshot And Smoke

**Files:**

- Modify: `docs/superpowers/reports/joi-0.12-brief-material-understanding-smoke-test.md`

- [ ] **Step 1: Verify snapshot includes generated understanding**

Run:

```powershell
cd src-tauri
cargo test --test project_snapshots creates_project_snapshot_with_related_sections -- --nocapture
```

Expected:

- Existing snapshot test passes because snapshots already include `product_understandings` and `creative_directions`.

- [ ] **Step 2: Run full verification**

Run:

```powershell
npm test
npm run build
cd src-tauri
cargo test
cargo test --test commands -- --nocapture
```

Expected:

- All commands pass.

- [ ] **Step 3: Browser smoke**

Run Vite:

```powershell
npm run dev -- --host 127.0.0.1 --port 1420
```

Use the in-app browser to verify:

- Brief tab renders the 0.12 form.
- Form labels are visible.
- Reference material form renders.
- Normal browser shows a Tauri invoke error toast instead of crashing.
- Desktop layout has no text overlap.
- Narrow layout has no horizontal overflow.

- [ ] **Step 4: Write smoke report**

Create:

```text
docs/superpowers/reports/joi-0.12-brief-material-understanding-smoke-test.md
```

Include:

- automated commands run
- browser smoke observations
- Tauri runtime limitation
- acceptance checklist

### Task 6: Commit, Merge, Push

**Files:**

- All 0.12 implementation files.

- [ ] **Step 1: Commit 0.12 implementation**

Run:

```powershell
git status --short
git add <0.12 files>
git commit -m "feat: add Joi 0.12 brief understanding"
```

- [ ] **Step 2: Merge to main**

Run from repository root:

```powershell
git checkout main
git merge --ff-only codex/joi-0.12-brief-understanding
```

Expected:

- Fast-forward merge if no unrelated main changes occurred.

- [ ] **Step 3: Verify on main**

Run:

```powershell
npm test
npm run build
cd src-tauri
cargo test
cargo test --test commands -- --nocapture
```

Expected:

- All commands pass.

- [ ] **Step 4: Push**

Run:

```powershell
git push origin main
```

Expected:

- Remote `main` updates to the 0.12 commit.

## Acceptance Criteria

0.12 is complete only when:

- Brief tab is data-backed and no longer an empty state.
- User can enter brief, product, audience, selling points, visual direction, constraints, platform context, and duration context.
- User can add link/reference material records.
- User can generate a structured understanding from the Brief tab.
- Generated result includes brand summary, brief summary, product understanding, selling points, constraints, visual direction, and missing questions.
- Generated understanding is saved in local repository.
- Creative direction is saved when visual direction is provided.
- Product understandings and creative directions are included in snapshots.
- Frontend tests cover brief generation.
- Rust tests cover command and persistence behavior.
- Browser smoke report is written.
- Changes are merged to `main` and pushed to GitHub.

## Risks And Mitigations

### Risk: 0.12 Looks Like AI Generation But Is Rule-Based

Mitigation:

- Button label should be `Generate Understanding`, not `Ask Agent`.
- Result panel should show structured interpretation, not claim autonomous reasoning.
- Agent runtime remains explicitly out of scope until 0.13.

### Risk: `notes` JSON Becomes Hard To Use

Mitigation:

- Store a `format_version`.
- Keep typed fields duplicated in the command result.
- Parse defensively in frontend.

### Risk: Reference Materials Need Native File Picker

Mitigation:

- 0.12 supports link/reference records through `joi_create_reference_asset`.
- Native file picker is outside 0.12 and should remain separate from brief understanding.

### Risk: ProjectWorkspace Becomes Too Large

Mitigation:

- Create `BriefWorkspace`, `ReferenceMaterialPanel`, and `UnderstandingResultPanel` in 0.12 so `ProjectWorkspace.tsx` remains a coordinator.

## Handoff To 0.13

0.13 should use the saved 0.12 records as Agent runtime context:

- latest project brief
- product understanding
- creative direction
- reference assets
- missing questions
- project memory

The 0.13 planner should not infer project context directly from freeform UI fields; it should read the structured records created in 0.12.

## Self-Review

- Spec coverage: The plan covers Brief editor, product info editor, platform/duration context, reference material panel, product understanding generation, brand context summary, missing information questions, and local persistence.
- Placeholder scan: The plan avoids placeholder tasks and defines command names, DTOs, file paths, test commands, and expected behavior.
- Type consistency: Rust command names map to TypeScript wrapper names; `BriefUnderstandingInput` and `BriefUnderstandingResult` fields match across backend and frontend sections.
