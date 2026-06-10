# Joi Agent Phase 1 Data Store Design

## Summary

Phase 1 establishes Joi Agent's local product-state foundation. The goal is not
to build the final three-column workspace or generation workflow yet. The goal
is to create a Tauri application shell with a Rust backend, a SQLite local
database, project asset storage, validated domain schemas, version snapshots,
and JSON project import/export.

After Phase 1, Joi should be able to create and persist the core objects needed
by the fashion advertising workflow even without Hermes generation calls:
brands, projects, assets, research reports, product understanding, creative
directions, storyboards, shots, prompt packages, project versions, and memory
entries.

## Confirmed Decisions

- Local storage uses SQLite.
- The data layer is owned by the Rust/Tauri backend.
- Phase 1 creates the Tauri app skeleton.
- The frontend is a minimal placeholder page only.
- Phase 1 covers the full MVP data skeleton.
- Phase 1 does not implement Agent generation logic.
- Assets are copied into a Joi-managed project asset directory.
- SQLite stores asset metadata and relative paths, not file blobs.
- Versioning uses project-level complete snapshots plus optional entity-level
  change metadata.
- Import/export uses `.joi-project.json` plus a sibling assets folder.
- Memory entries are stored in a Joi-owned memory ledger and are not written to
  Hermes memory in Phase 1.

## Goals

Phase 1 should deliver:

- A runnable Tauri application shell.
- A Rust backend crate or module layout that owns Joi domain persistence.
- SQLite migrations for the MVP data model.
- Typed Rust domain models and validation boundaries.
- Tauri commands for basic data operations.
- Local asset import that copies files into app-managed storage.
- Project snapshot creation and project-level rollback.
- JSON project export/import using a manifest plus assets folder.
- A minimal frontend page that proves the app and backend are wired.
- Tests for the data layer, asset handling, snapshots, and import/export.

## Non-Goals

Phase 1 explicitly does not include:

- The final three-column Joi Desktop UI.
- Storyboard generation.
- Prompt generation.
- Research automation.
- Reference video frame extraction.
- Calls to Jimeng, Grok, Banana 2, Jimeng Image, or GPT Image 2.
- Hermes runtime integration.
- Hermes memory writes.
- Accounts, permissions, cloud sync, or multi-tenant behavior.
- Packaged production installers.

## Architecture

Phase 1 uses a local-first desktop architecture:

- `src-tauri/` contains the Tauri app, Rust backend, SQLite connection
  management, migrations, commands, and tests.
- The frontend exists only as a minimal placeholder shell in the Tauri app.
- SQLite stores structured entities and JSON snapshots.
- The app data directory contains project asset folders managed by Joi.
- Tauri commands expose backend operations to the frontend and future workflow
  UI.

The Rust backend is the source of truth for structured project state. Frontend
code and future Hermes domain tools must go through backend commands or backend
service interfaces rather than writing project files directly.

## Storage Layout

The app should use a local app data root such as:

```text
<app-data>/joi-agent/
  joi.db
  projects/
    <project-id>/
      assets/
        <asset-id>.<extension>
      exports/
```

The exact app data root should use Tauri's supported app data path mechanism.
Tests should use temporary directories rather than the real user app data path.

SQLite stores relative asset paths such as:

```text
projects/<project-id>/assets/<asset-id>.<extension>
```

Absolute source paths may be stored only as optional provenance metadata. Joi
must not depend on the original user file remaining in place after import.

## Data Model

The database should use stable string IDs, preferably UUID-style IDs generated
by the backend. All major tables include `created_at` and `updated_at` where
applicable.

### Brand

Stores reusable brand-level context.

Fields:

- `id`
- `name`
- `description`
- `style_keywords`
- `visual_preferences`
- `negative_preferences`
- `common_scenes`
- `model_preferences`
- `platform_preferences`
- `created_at`
- `updated_at`

Brand memory is stored as scoped `memory_entries`, not as a large opaque brand
field.

### Project

Stores the main fashion advertising work container.

Fields:

- `id`
- `brand_id`
- `title`
- `advertising_goal`
- `duration_seconds`
- `target_platforms`
- `workflow_stage`
- `current_version_id`
- `final_version_id`
- `created_at`
- `updated_at`

`duration_seconds` should support the primary MVP range of 15 to 30 seconds,
but the database should not make future longer formats impossible.

### Asset

Represents imported product images, reference images, reference videos, and
links.

Fields:

- `id`
- `project_id`
- `kind`
- `display_name`
- `relative_path`
- `source_uri`
- `mime_type`
- `file_size_bytes`
- `sha256`
- `metadata_json`
- `created_at`
- `updated_at`

`kind` should at minimum support:

- `product_image`
- `reference_image`
- `reference_video`
- `link`
- `other`

For local files, Joi copies the file into the project asset directory and
stores a hash. For links, `source_uri` is required and `relative_path` may be
empty.

### ResearchReport

Stores structured research output slots. Phase 1 only persists user-provided or
placeholder content.

Fields:

- `id`
- `project_id`
- `summary`
- `findings_json`
- `sources_json`
- `created_at`
- `updated_at`

### ProductUnderstanding

Stores structured interpretation of the product and campaign context.

Fields:

- `id`
- `project_id`
- `product_name`
- `category`
- `audience`
- `selling_points_json`
- `constraints_json`
- `notes`
- `created_at`
- `updated_at`

### CreativeDirection

Stores one or more creative routes for a project.

Fields:

- `id`
- `project_id`
- `title`
- `concept`
- `tone`
- `visual_style`
- `scene_direction`
- `rationale`
- `created_at`
- `updated_at`

### Storyboard

Stores the storyboard container for a project.

Fields:

- `id`
- `project_id`
- `title`
- `duration_seconds`
- `created_at`
- `updated_at`

### Shot

Stores shot-level storyboard rows.

Fields:

- `id`
- `storyboard_id`
- `shot_number`
- `duration_seconds`
- `description`
- `model_action`
- `camera_movement`
- `scene`
- `lighting`
- `subtitle_or_voiceover`
- `rationale`
- `is_locked`
- `metadata_json`
- `created_at`
- `updated_at`

The backend should preserve shot ordering and support future reorder behavior.

### PromptPackage

Stores platform prompt outputs by project, shot, and target platform. Phase 1
does not generate prompt text automatically, but it must support storing and
editing prompt package records.

Fields:

- `id`
- `project_id`
- `shot_id`
- `platform`
- `modality`
- `prompt_text`
- `negative_prompt`
- `parameters_json`
- `is_locked`
- `created_at`
- `updated_at`

Platforms:

- `jimeng_video`
- `grok_video`
- `banana_2_image`
- `jimeng_image`
- `gpt_image_2`

Modalities:

- `video`
- `image`

### ProjectVersion

Stores project-level complete snapshots.

Fields:

- `id`
- `project_id`
- `version_number`
- `label`
- `change_reason`
- `changed_entities_json`
- `snapshot_json`
- `created_by`
- `is_final_candidate`
- `created_at`

Snapshots contain the full restorable project state, including structured
entities and asset manifest metadata. Snapshots do not embed binary asset file
contents.

Rollback restores structured project state from `snapshot_json`. It should not
delete asset files by default; unused asset cleanup can be a later maintenance
feature.

### MemoryEntry

Stores Joi-owned memory candidates and accepted memories.

Fields:

- `id`
- `scope`
- `brand_id`
- `project_id`
- `content`
- `source`
- `source_entity_type`
- `source_entity_id`
- `confidence`
- `status`
- `created_at`
- `updated_at`

Scopes:

- `user`
- `brand`
- `project`

Statuses:

- `proposed`
- `accepted`
- `rejected`

Phase 1 does not sync these entries to Hermes memory. Future `memory_router`
work decides what should be shared with Hermes.

## Backend Services

The Rust backend should be organized around small services with explicit
responsibilities.

### Database Service

Responsibilities:

- Open the SQLite connection.
- Apply migrations.
- Provide transactional execution helpers.
- Enforce foreign keys.

### Repository Layer

Responsibilities:

- CRUD operations for domain entities.
- Query projects by brand.
- Query assets, shots, prompt packages, versions, and memory entries by project.
- Keep SQL localized rather than spread across commands.

### Asset Service

Responsibilities:

- Copy imported local files into the project asset directory.
- Compute SHA-256.
- Derive file metadata.
- Reject missing or unsupported local paths with clear errors.
- Store asset rows transactionally with copied file metadata.

### Snapshot Service

Responsibilities:

- Build complete project snapshots from database state.
- Store project versions.
- Mark final candidate versions.
- Restore structured state from snapshots.

### Import/Export Service

Responsibilities:

- Export `.joi-project.json`.
- Export sibling assets folder.
- Import project JSON.
- Copy imported assets into app-managed storage.
- Validate package schema before database writes.
- Avoid path traversal when reading import assets.

## Tauri Commands

Phase 1 should expose a practical but small command surface:

- `joi_health_check`
- `joi_create_brand`
- `joi_list_brands`
- `joi_get_brand`
- `joi_update_brand`
- `joi_create_project`
- `joi_list_projects`
- `joi_get_project`
- `joi_update_project`
- `joi_import_asset`
- `joi_list_assets`
- `joi_save_project_snapshot`
- `joi_list_project_versions`
- `joi_restore_project_version`
- `joi_export_project`
- `joi_import_project`
- `joi_create_memory_entry`
- `joi_list_memory_entries`

The command names should stay stable because Phase 2 frontend work will build
against them.

## Validation Rules

The backend should reject invalid writes before persistence.

Initial validation rules:

- Brand name is required.
- Project title is required.
- Project must reference an existing brand.
- Asset kind must be one of the allowed kinds.
- Local file assets require a readable source path.
- Link assets require `source_uri`.
- Shot duration must be non-negative.
- Prompt platform and modality must be valid enum values.
- Prompt modality must match platform family.
- Memory scope must be valid.
- Brand-scoped memory requires `brand_id`.
- Project-scoped memory requires `project_id`.
- Snapshot rollback requires a version belonging to the target project.

## Import/Export Format

Phase 1 project export produces:

```text
<export-dir>/
  <project-slug>.joi-project.json
  <project-slug>-assets/
    <asset files>
```

The JSON file includes:

- Export format version.
- Brand snapshot.
- Project record.
- Assets manifest.
- Research reports.
- Product understanding records.
- Creative directions.
- Storyboards and shots.
- Prompt packages.
- Version metadata.
- Memory entry metadata.

The JSON package should be human-readable with stable key ordering where
reasonable. It should not embed binary file content.

Future work may add `.joi-project.zip`, but Phase 1 should not implement zip
packaging.

## Frontend Placeholder

The Phase 1 frontend only needs to prove that the Tauri app and backend are
wired.

It should show:

- Product name: `Joi Agent`.
- Current phase: `Phase 1 local data store`.
- Backend health state from `joi_health_check`.

It should not attempt to implement the three-column workspace, project list, or
asset manager. Those belong to Phase 2.

## Testing Strategy

Phase 1 should include Rust tests for:

- Database migration initialization.
- Brand CRUD.
- Project CRUD.
- Asset import copies files and records hash/path metadata.
- Snapshot creation includes complete project state.
- Snapshot rollback restores structured state.
- JSON export writes project JSON and assets folder.
- JSON import validates and creates project data.
- Memory entries can be created and queried by scope.
- Invalid enum values and missing required fields are rejected.

Tests must use temporary directories and must not write to the user's real app
data directory.

## Error Handling

Errors should be structured and user-facing enough for Phase 2 UI to display
them.

Important cases:

- Database migration failure.
- Missing app data directory permissions.
- Missing source asset path.
- Unsupported or unreadable file.
- Asset copy failure.
- Invalid JSON import package.
- Import package path traversal attempt.
- Snapshot restore failure.
- Foreign key violations.

The backend should avoid panics for expected user or file-system errors.

## Integration With Hermes

Hermes remains the validated runtime foundation from Phase 0, but Phase 1 does
not call Hermes. This keeps product-state ownership independent from model
runtime behavior.

Future phases will integrate Hermes through:

- Domain tools such as `research_brief`, `product_understanding`,
  `creative_direction_generator`, `storyboard_generator`, `prompt_adapter`,
  `project_exporter`, and `memory_router`.
- Runtime access to model providers, search, browser, vision, and MCP.
- Explicit routing from Joi memory ledger entries into Hermes memory where
  appropriate.

## Acceptance Criteria

Phase 1 is complete when:

- The Tauri app skeleton runs locally.
- The placeholder frontend can call `joi_health_check`.
- SQLite migrations create the full MVP data skeleton.
- CRUD and query commands work for core entities.
- Assets are copied into Joi-managed storage and recorded in SQLite.
- Project snapshots can be created and restored.
- `.joi-project.json` plus assets folder export/import works.
- Joi memory entries are stored locally without Hermes memory writes.
- Tests cover migrations, CRUD, asset import, snapshots, import/export, and
  memory ledger behavior.
- No formal three-column UI or generation workflow is included.

## Risks

- Starting with the full MVP data skeleton can produce a large migration. Keep
  logic simple and defer complex workflow behavior.
- Tauri project setup can pull in frontend build complexity. Keep the Phase 1
  frontend minimal.
- Asset import must avoid path traversal and accidental dependence on original
  source files.
- Snapshot JSON can drift from table schemas. Keep snapshot building centralized
  in one service.
- Rust/Tauri backend ownership means Hermes integration needs a later bridge,
  but that is an intentional boundary.

## Approval

This spec reflects the confirmed Phase 1 choices:

- SQLite plus JSON import/export.
- Rust/Tauri backend ownership.
- Tauri app skeleton with placeholder frontend.
- Full MVP data skeleton.
- Joi-managed asset directory.
- Project-level snapshots with entity change metadata.
- `.joi-project.json` plus assets folder export/import.
- Joi-owned memory ledger, no Hermes memory writes in Phase 1.
