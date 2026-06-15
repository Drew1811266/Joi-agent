# Joi Agent 0.11 Workspace UI Implementation Plan

## Purpose

0.11 turns Joi from a backend-backed placeholder page into a usable local workspace shell for fashion advertising projects.

This version does not add Agent generation. It creates the product surface that later versions will use: project navigation, editable brand/project forms, status panels for assets/memory/versions, and a right-side Agent activity panel stub.

## Product Outcome

After 0.11, a user can:

- Open Joi as a desktop workspace instead of a status card.
- See backend health in the app chrome.
- Create, select, and update brands.
- Create, select, and update projects under a brand.
- View project assets, versions, and memory entries as workspace sections.
- Save a project snapshot from the UI.
- Understand where brief, research, creative direction, storyboard, prompts, and report workflows will live.

## Scope

### In Scope

- Replace the placeholder frontend with a three-column workspace layout.
- Add frontend API wrappers for existing Tauri commands.
- Add typed frontend models matching backend command payloads.
- Add local React state for selected brand/project and workspace data.
- Add forms for brand and project create/update.
- Add list views for brands and projects.
- Add read-only panels for assets, versions, and memory entries.
- Add snapshot save action.
- Add an Agent panel placeholder with task/status/log cards.
- Add frontend tests for critical state and UI behavior where practical.
- Keep Rust backend behavior mostly unchanged unless a small command contract gap is discovered.

### Out Of Scope

- No Agent runtime integration.
- No LLM calls.
- No web research.
- No storyboard or prompt generation.
- No full asset file picker workflow beyond showing the existing asset command contract.
- No report generation.
- No cloud sync or accounts.
- No full design system library.

## UX Structure

Joi 0.11 uses a dense desktop productivity layout.

```text
+----------------------------------------------------------------------+
| Top Bar: Joi Agent | backend status | selected project | actions      |
+---------------+--------------------------------------+---------------+
| Left Rail     | Main Workspace                       | Agent Panel   |
|               |                                      |               |
| Brands        | Project Overview                     | Current Task  |
| Projects      | Brief Placeholder                    | Plan Stub     |
| Assets        | Workflow Tabs                        | Memory Notes  |
| Versions      | Section Panels                       | Activity Log  |
| Memory        |                                      |               |
+---------------+--------------------------------------+---------------+
```

### Left Rail

Responsibilities:

- Show brand list.
- Show projects for the selected brand.
- Provide create brand and create project actions.
- Show quick navigation links:
  - Overview
  - Brief
  - Research
  - Creative Direction
  - Storyboard
  - Prompts
  - Delivery
  - Memory

Implementation:

- Use a constrained rail width around 280px.
- Keep brand/project rows compact.
- Use clear selected states.
- Avoid card nesting.

### Top Bar

Responsibilities:

- Show app name.
- Show backend status from `joi_health_check`.
- Show current brand/project summary.
- Provide primary action buttons:
  - Save Snapshot
  - Export Project

0.11 can render Export Project as a disabled or not-yet-wired action if a file dialog is not implemented. Save Snapshot should be wired.

### Main Workspace

Default view: Project Overview.

Sections:

- Project Setup
  - brand form
  - project form
  - duration and advertising goal
- Workflow Map
  - Brief
  - Research
  - Creative Direction
  - Storyboard
  - Prompts
  - Delivery
- Project State
  - Assets count
  - Memory count
  - Version count
  - Last snapshot status

Tabs or segmented navigation:

- Overview
- Brief
- Research
- Storyboard
- Prompts
- Assets
- Memory
- Versions

Only Overview, Assets, Memory, and Versions need data-backed panels in 0.11. Other tabs should show structured empty states that explain the workflow destination without pretending generation exists.

### Agent Panel

0.11 Agent panel is a non-generative control surface stub.

Sections:

- Current task
- Planned workflow
- Memory suggestions
- Activity log

It should clearly communicate that execution is not active yet. It must not fake generated outputs.

## UI Components

Create or reorganize frontend files under `src/`.

Recommended structure:

```text
src/
  App.tsx
  styles.css
  api/
    joiApi.ts
  components/
    AgentPanel.tsx
    AppShell.tsx
    BrandProjectRail.tsx
    EmptyState.tsx
    MetricStrip.tsx
    ProjectWorkspace.tsx
    TopBar.tsx
  state/
    workspaceState.ts
  types/
    joi.ts
```

If the implementation remains small enough, fewer files are acceptable, but keep API wrappers and shared types separate from JSX.

## Frontend Data Model

Create TypeScript types matching backend outputs.

Minimum types:

```ts
export type HealthResponse = {
  status: string;
  app_name: string;
  phase: string;
};

export type Brand = {
  id: string;
  name: string;
  description: string;
  style_keywords: string[];
  visual_preferences: string[];
  negative_preferences: string[];
  common_scenes: string[];
  model_preferences: string[];
  platform_preferences: string[];
  created_at: string;
  updated_at: string;
};

export type Project = {
  id: string;
  brand_id: string;
  title: string;
  advertising_goal: string;
  duration_seconds: number;
  target_platforms: string[];
  content_type: string;
  status: string;
  current_version_id: string | null;
  final_version_id: string | null;
  created_at: string;
  updated_at: string;
};

export type Asset = {
  id: string;
  project_id: string;
  kind: string;
  display_name: string;
  relative_path: string;
  source_uri: string;
  mime_type: string;
  file_size_bytes: number;
  sha256: string;
  created_at: string;
  updated_at: string;
};

export type ProjectVersion = {
  id: string;
  project_id: string;
  version_number: number;
  label: string;
  change_reason: string;
  changed_entities: string[];
  snapshot_json: unknown;
  created_by: string;
  is_final_candidate: boolean;
  created_at: string;
};

export type MemoryEntry = {
  id: string;
  scope: string;
  brand_id: string | null;
  project_id: string | null;
  content: string;
  source: string;
  source_entity_type: string;
  source_entity_id: string;
  confidence: number;
  status: string;
  created_at: string;
  updated_at: string;
};
```

## Tauri Commands To Use

0.11 frontend should call:

- `joi_health_check`
- `joi_create_brand`
- `joi_list_brands`
- `joi_get_brand`
- `joi_update_brand`
- `joi_create_project`
- `joi_list_projects`
- `joi_get_project`
- `joi_update_project`
- `joi_list_assets`
- `joi_save_project_snapshot`
- `joi_list_project_versions`
- `joi_create_memory_entry`
- `joi_list_memory_entries`

Optional display-only future actions:

- `joi_import_asset`
- `joi_export_project`
- `joi_import_project`
- `joi_restore_project_version`

If a command requires a native file path and no file dialog exists yet, show the action as unavailable or as a non-wired placeholder.

## API Wrapper

Create `src/api/joiApi.ts`.

Responsibilities:

- Wrap `invoke`.
- Keep command names in one place.
- Keep snake_case payloads aligned with backend `rename_all = "snake_case"`.
- Normalize errors into display strings.

Example shape:

```ts
export async function listBrands(): Promise<Brand[]> {
  return invoke<Brand[]>("joi_list_brands");
}

export async function createBrand(input: BrandInput): Promise<Brand> {
  return invoke<Brand>("joi_create_brand", { input });
}
```

## State Management

Use React state and effects for 0.11. Do not introduce Redux, Zustand, TanStack Query, or router unless the UI becomes too complex.

Recommended state:

- `health`
- `brands`
- `selectedBrandId`
- `projects`
- `selectedProjectId`
- `assets`
- `versions`
- `memoryEntries`
- `activeWorkspaceTab`
- `loading`
- `error`
- `activityLog`

Data loading rules:

- On app load:
  - health check
  - list brands
- On selected brand change:
  - list projects for brand
- On selected project change:
  - list assets
  - list versions
  - list project memory
- After create/update:
  - refresh affected list
  - keep selected entity stable when possible

## Forms

### Brand Form

Fields:

- name
- description

Behavior:

- Create brand when no selected brand exists or user chooses New Brand.
- Update selected brand when editing existing brand.
- Validate non-empty name in UI before command call.

### Project Form

Fields:

- title
- advertising_goal
- duration_seconds

Behavior:

- Requires selected brand.
- Create project under selected brand.
- Update selected project without changing brand.
- Validate non-empty title and positive duration in UI.

## Workspace Panels

### Overview Panel

Show:

- selected brand name
- selected project title
- advertising goal
- duration
- asset count
- memory count
- version count
- next recommended workflow steps

### Assets Panel

Show:

- asset kind
- display name
- mime type
- file size
- relative path

0.11 does not need file import UI unless it can be implemented safely with a native file dialog.

### Memory Panel

Show:

- memory scope
- content
- source
- status
- confidence

Add a small form to create project-scoped memory for the selected project:

- content
- source

Use `joi_create_memory_entry` with:

```ts
{
  scope: "project",
  brand_id: selectedBrandId,
  project_id: selectedProjectId,
  content,
  source
}
```

### Versions Panel

Show:

- version number
- label
- change reason
- created by
- created at

Save snapshot action:

```ts
{
  project_id: selectedProjectId,
  label,
  change_reason
}
```

Restore version can be visible but disabled unless UX for destructive rollback confirmation is implemented.

## Styling Direction

Joi is a professional desktop workspace, not a marketing page.

Guidelines:

- Dense but calm layout.
- Neutral background with high contrast text.
- Avoid oversized hero sections.
- Avoid decorative gradient blobs.
- Use 8px or smaller radius.
- Keep cards only for individual repeated items or panels, not nested decorative cards.
- Buttons should have clear command labels.
- Use segmented controls or tabs for workspace sections.
- Text must fit inside controls at desktop and narrow widths.
- Avoid one-note purple/blue gradient styling.

Suggested palette:

- app background: `#f6f5ef`
- panel background: `#ffffff`
- primary text: `#18211b`
- muted text: `#66706a`
- border: `#d9ddd4`
- accent: `#2f6f5e`
- warning/error: `#a64235`
- soft highlight: `#eef4ef`

## Accessibility

Minimum requirements:

- Buttons use `<button>`.
- Inputs have labels.
- Error messages are visible near affected forms or in a global status area.
- Selected items use semantic `aria-current` or clear text state where practical.
- Keyboard tab order follows left rail -> main workspace -> agent panel.

## Implementation Steps

### Step 1: Frontend Types And API Wrapper

Files:

- `src/types/joi.ts`
- `src/api/joiApi.ts`

Tasks:

- Define backend response and input types.
- Add wrappers for health, brand, project, assets, memory, versions, snapshots.
- Add shared `formatError(error: unknown): string`.

Verification:

- `npm run build`

### Step 2: Workspace Shell Components

Files:

- `src/components/AppShell.tsx`
- `src/components/TopBar.tsx`
- `src/components/BrandProjectRail.tsx`
- `src/components/AgentPanel.tsx`
- `src/components/ProjectWorkspace.tsx`
- `src/components/EmptyState.tsx`
- `src/components/MetricStrip.tsx`
- `src/App.tsx`
- `src/styles.css`

Tasks:

- Replace centered status card.
- Add three-column layout.
- Preserve health display.
- Add static workspace tabs and section placeholders.

Verification:

- `npm run build`
- Browser check at Vite dev URL.

### Step 3: Brand And Project Data Flow

Files:

- `src/App.tsx`
- components as needed
- `src/api/joiApi.ts`

Tasks:

- Load brands on app start.
- Select first brand automatically when available.
- Load projects for selected brand.
- Create/update brand.
- Create/update project.
- Keep selected project stable after update.

Verification:

- Manual smoke in UI.
- Add frontend unit tests if test setup exists; if not, rely on build plus Tauri command tests.

### Step 4: Project State Panels

Files:

- `src/components/ProjectWorkspace.tsx`
- optional dedicated panels:
  - `AssetsPanel.tsx`
  - `MemoryPanel.tsx`
  - `VersionsPanel.tsx`

Tasks:

- Load project assets.
- Load project memory.
- Load project versions.
- Add project memory creation form.
- Add snapshot save form/action.

Verification:

- `npm run build`
- `cargo test`

### Step 5: UI Polish And Smoke Report

Files:

- `src/styles.css`
- `docs/superpowers/reports/joi-0.11-workspace-ui-smoke-test.md`

Tasks:

- Verify responsive desktop layout.
- Verify no text overflow in major controls.
- Verify empty states are clear.
- Run Vite or Tauri dev smoke.
- Document what was verified.

Verification:

- `npm run build`
- `cargo test`
- Browser visual check.

## Test Plan

### Frontend Build

```powershell
npm run build
```

Expected:

- TypeScript build passes.
- Vite production build passes.

### Rust Tests

```powershell
cd src-tauri
cargo test
```

Expected:

- Existing backend tests pass.

### Tauri Command Contract

```powershell
cd src-tauri
cargo test --test commands -- --nocapture
```

Expected:

- command DTO and helper tests pass.

### Browser Smoke

Use the in-app browser or a local browser against the Vite dev URL.

Expected visible state:

- Top bar shows Joi Agent.
- Backend status is ready.
- Left rail shows brand/project controls.
- Main workspace shows Overview and workflow tabs.
- Right panel shows Agent placeholder.
- User can create a brand.
- User can create a project under that brand.
- User can save a snapshot for selected project.

### Optional Tauri Dev Smoke

```powershell
npm run tauri:dev
```

Expected:

- Desktop app launches.
- Workspace renders without runtime errors.
- Backend health resolves.

## Acceptance Criteria

0.11 is complete only when:

- Placeholder status card is replaced by workspace UI.
- UI has left rail, main workspace, and right Agent panel.
- Health check is visible and uses live backend command.
- User can create and update brands from UI.
- User can create and update projects from UI.
- User can select a brand and project.
- UI lists assets, memory entries, and versions for selected project.
- User can create project-scoped memory from UI.
- User can save a project snapshot from UI.
- Frontend build passes.
- Rust tests pass.
- Browser smoke report is written.
- Changes are merged to `main` and pushed to GitHub.

## Risks And Mitigations

### Risk: Frontend grows too large in one file

Mitigation:

- Keep API types and wrappers separate.
- Split repeated UI into components early.

### Risk: Path-based commands need native file dialogs

Mitigation:

- Do not wire import/export buttons unless a safe path UX exists.
- Show disabled actions with clear labels.

### Risk: Tauri command payload names drift

Mitigation:

- Keep all invoke wrappers in `src/api/joiApi.ts`.
- Match backend snake_case payloads.

### Risk: UI implies Agent generation exists

Mitigation:

- Agent panel must say execution is not active yet.
- Empty states should point to upcoming workflow sections without faking results.

### Risk: No frontend test framework exists

Mitigation:

- Use TypeScript build as first gate.
- Use browser smoke report for UI behavior.
- Add a test framework only if UI logic becomes too complex for manual smoke.

## Deliverables

- Workspace UI implementation.
- Typed frontend API wrapper.
- Brand/project create and update UI.
- Project state panels.
- Memory create UI.
- Snapshot save UI.
- Smoke test report.
- Clean `main` branch push.

## Handoff To 0.12

0.12 should start from a usable workspace and add:

- brief editor
- product information editor
- target platform settings
- material understanding generation
- structured product understanding persistence
- missing information questions
