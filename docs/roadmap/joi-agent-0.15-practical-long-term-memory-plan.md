# Joi Agent 0.15 Practical Long-Term Memory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Joi memory usable in the real content workflow. Joi should propose memory candidates from project outputs and user feedback, let users accept or reject them, keep source traces, detect simple conflicts, and feed accepted memory back into later Agent context.

**Architecture:** 0.15 builds on the existing `memory_entries` table and the 0.13/0.14 Agent run model. No database migration is required because the memory table already has `source_entity_type`, `source_entity_id`, `confidence`, and `status`. The implementation adds repository helpers, a `memory_curation` service, Tauri commands, and a richer Memory workspace.

**Tech Stack:** Tauri 2 commands, Rust, rusqlite, serde/serde_json, chrono, React 19, TypeScript, Vitest, Joi 0.13 `agent_runs` and `agent_run_events`, Joi 0.14 `research_reports`.

---

## Product Outcome

After 0.15, a user can:

- Open the Memory tab for a project.
- See memory grouped by `proposed`, `accepted`, and `rejected`.
- Generate memory candidates from saved research reports and optional user feedback.
- See each candidate with source trace, confidence, and conflict status.
- Accept or reject proposed memory.
- Add manual memory as before.
- See accepted memory stay available in Agent project context for future storyboards and prompts.
- See memory curation recorded as an Agent run with visible events.

0.15 does not create external/cloud memory. It makes local user/brand/project memory operational inside Joi.

## Scope

### In Scope

- Repository support:
  - create proposed memory with source trace and confidence
  - update memory status to proposed/accepted/rejected
  - preserve existing manual memory creation behavior
- Memory curation service:
  - read project context
  - read saved research reports
  - turn stable research findings and creative implications into memory candidates
  - turn optional user feedback into memory candidates
  - detect simple duplicate/conflicting memory against existing entries
  - write proposed memory entries
  - write Agent run/events
- Command layer:
  - `joi_generate_memory_candidates`
  - `joi_update_memory_status`
- Frontend Memory workspace:
  - proposed/accepted/rejected grouping
  - generate candidates from research and feedback
  - accept/reject buttons
  - source trace display
  - conflict indicator
- Tests and smoke report.

### Out Of Scope

- No external Agent runtime memory write.
- No cloud sync or multi-user memory.
- No semantic vector similarity.
- No automatic acceptance of memory.
- No deletion UI.
- No memory suggestions from storyboard/prompt edits yet; 0.15 creates the service path that 0.16 and 0.17 can reuse.

## Data Contract

### Existing `memory_entries` Table

The current schema already supports 0.15:

```sql
source_entity_type TEXT NOT NULL DEFAULT '',
source_entity_id TEXT NOT NULL DEFAULT '',
confidence REAL NOT NULL DEFAULT 0.0,
status TEXT NOT NULL DEFAULT 'proposed'
```

No migration is required.

### `MemoryCandidateCreate`

Add to `src-tauri/src/repositories.rs`:

```rust
#[derive(Debug, Clone)]
pub struct MemoryCandidateCreate {
    pub scope: String,
    pub brand_id: Option<String>,
    pub project_id: Option<String>,
    pub content: String,
    pub source: String,
    pub source_entity_type: String,
    pub source_entity_id: String,
    pub confidence: f64,
}
```

Rules:

- `scope` must be `user`, `brand`, or `project`.
- 0.15 memory curation generates `project` scope only.
- `content` is required and trimmed.
- `confidence` must be clamped or rejected outside `0.0..=1.0`; prefer rejection for explicit correctness.
- `status` is always `proposed` on creation.
- Existing brand/project/user scope validation must be reused.

### `MemoryStatusUpdate`

Add to `src-tauri/src/repositories.rs`:

```rust
#[derive(Debug, Clone)]
pub struct MemoryStatusUpdate {
    pub id: String,
    pub status: String,
}
```

Rules:

- `status` must be one of `proposed`, `accepted`, `rejected`.
- Missing memory id returns `JoiError::NotFound`.
- Updating status refreshes `updated_at`.
- Accepted and rejected records remain in the table for source trace and audit.

### `MemoryCurationInput`

Add to `src-tauri/src/memory_curation.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryCurationInput {
    pub project_id: String,
    pub feedback_text: String,
    pub include_research_reports: bool,
}
```

Rules:

- `project_id` must exist.
- At least one of `feedback_text` or `include_research_reports` must produce candidate material.
- Empty `feedback_text` is allowed when `include_research_reports = true`.

### `MemoryCandidateResult`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCandidateResult {
    pub entry: MemoryEntry,
    pub reason: String,
    pub has_conflict: bool,
    pub conflict_memory_ids: Vec<String>,
}
```

### `MemoryCurationResult`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCurationResult {
    pub candidates: Vec<MemoryCandidateResult>,
    pub agent_run: AgentRun,
    pub agent_events: Vec<AgentRunEvent>,
}
```

### `MemoryStatusInput`

Add to `src-tauri/src/commands.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryStatusInput {
    pub id: String,
    pub status: String,
}
```

## Candidate Rules

### Research-Derived Candidates

When `include_research_reports = true`, Joi reads saved `research_reports` for the project.

From each `findings_json` item:

- if `creative_implication` exists, propose it as memory
- otherwise if `insight` exists, propose it as memory
- source trace:
  - `source = "research report"`
  - `source_entity_type = "research_report"`
  - `source_entity_id = report.id`
  - `confidence = 0.72`

Example candidate:

```text
Use tactile close-ups as visual proof before the model movement.
```

### Feedback-Derived Candidates

When `feedback_text` is non-empty, split it by line breaks and sentence separators. Each stable instruction becomes one candidate.

Source trace:

- `source = "user feedback"`
- `source_entity_type = "feedback"`
- `source_entity_id = ""`
- `confidence = 0.86`

### Conflict Detection

0.15 should implement simple deterministic conflict detection:

- normalize content by lowercasing, trimming, and collapsing whitespace
- compare only within the same memory scope and same project/brand target
- mark conflict when normalized content exactly matches an existing `proposed` or `accepted` memory
- return `conflict_memory_ids`
- still create the candidate as `proposed` so the user can decide

0.15 does not do semantic contradiction detection.

## Agent Events

Generating memory candidates creates one Agent run:

- `status`: `completed`
- `runtime_kind`: `hermes_core`
- `runtime_mode`: `local_memory_bridge`
- `roles_json`: `["memory_curator", "reviewer"]`

Expected events:

1. memory_curator `memory_context_read`
2. memory_curator `candidate_sources_collected`
3. memory_curator `memory_candidates_drafted`
4. reviewer `memory_conflicts_checked`
5. memory_curator `memory_candidates_saved`

## Implementation Tasks

### Task 1: Repository Support For Memory Status And Candidates

**Files:**

- Modify: `src-tauri/src/repositories.rs`
- Test: `src-tauri/tests/memory_ledger.rs`

- [ ] **Step 1: Write failing repository tests**

Add tests:

```rust
#[test]
fn creates_memory_candidate_with_source_trace_and_confidence() {
    let app = TestApp::new();
    let db = open_repo(&app);
    let repo = Repository::new(db.connection());
    let brand = repo.create_brand(BrandCreate {
        name: "Atelier Joi".into(),
        description: String::new(),
    }).unwrap();
    let project = repo.create_project(ProjectCreate {
        brand_id: brand.id.clone(),
        title: "Campaign".into(),
        advertising_goal: String::new(),
        duration_seconds: 15,
    }).unwrap();

    let memory = repo.create_memory_candidate(MemoryCandidateCreate {
        scope: "project".into(),
        brand_id: Some(brand.id),
        project_id: Some(project.id.clone()),
        content: "Use tactile close-ups as visual proof.".into(),
        source: "research report".into(),
        source_entity_type: "research_report".into(),
        source_entity_id: "research-1".into(),
        confidence: 0.72,
    }).unwrap();

    assert_eq!(memory.status, "proposed");
    assert_eq!(memory.source_entity_type, "research_report");
    assert_eq!(memory.source_entity_id, "research-1");
    assert_eq!(memory.confidence, 0.72);
}
```

Add:

```rust
#[test]
fn updates_memory_status_to_accepted_or_rejected() {
    // create proposed memory
    // update to accepted
    // update to rejected
    // reject invalid status
    // reject missing id
}
```

- [ ] **Step 2: Run tests and confirm RED**

```powershell
cd src-tauri
cargo test --test memory_ledger creates_memory_candidate_with_source_trace_and_confidence -- --nocapture
cargo test --test memory_ledger updates_memory_status_to_accepted_or_rejected -- --nocapture
```

- [ ] **Step 3: Implement repository structs**

Add `MemoryCandidateCreate` and `MemoryStatusUpdate`.

- [ ] **Step 4: Implement `create_memory_candidate`**

Reuse the same scope validation rules as `create_memory_entry`.

Persist:

- `source_entity_type`
- `source_entity_id`
- `confidence`
- `status = proposed`

- [ ] **Step 5: Implement `update_memory_entry_status`**

Validate status via `MemoryStatus::try_from(status.as_str())`.

Update:

```sql
UPDATE memory_entries
SET status = ?1, updated_at = ?2
WHERE id = ?3
```

Return the updated `MemoryEntry`.

- [ ] **Step 6: Run memory repository tests**

```powershell
cd src-tauri
cargo test --test memory_ledger -- --nocapture
```

Expected:

- All memory tests pass.

### Task 2: Memory Curation Service

**Files:**

- Create: `src-tauri/src/memory_curation.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/memory_curation.rs`

- [ ] **Step 1: Write failing service tests**

Create tests:

```rust
#[test]
fn generates_memory_candidates_from_research_report() {
    // seed brand/project
    // seed research report with findings_json containing creative_implication
    // call curate_memory_candidates
    // assert candidate content, source trace, confidence, status, agent events
}
```

```rust
#[test]
fn marks_duplicate_memory_candidates_as_conflicts() {
    // seed accepted project memory
    // seed research report or feedback with identical content
    // call curate_memory_candidates
    // assert has_conflict = true and conflict_memory_ids contains existing memory id
}
```

```rust
#[test]
fn rejects_memory_curation_without_candidate_material() {
    // include_research_reports false and blank feedback should fail
}
```

- [ ] **Step 2: Run tests and confirm RED**

```powershell
cd src-tauri
cargo test --test memory_curation -- --nocapture
```

- [ ] **Step 3: Implement DTOs**

Add:

- `MemoryCurationInput`
- `MemoryCandidateResult`
- `MemoryCurationResult`

- [ ] **Step 4: Implement source extraction**

Implement helpers:

```rust
fn research_candidates(repo: &Repository<'_>, project_id: &str) -> JoiResult<Vec<CandidateDraft>>
fn feedback_candidates(feedback_text: &str) -> Vec<CandidateDraft>
```

`CandidateDraft` can be private:

```rust
struct CandidateDraft {
    content: String,
    source: String,
    source_entity_type: String,
    source_entity_id: String,
    confidence: f64,
    reason: String,
}
```

- [ ] **Step 5: Implement conflict detection**

Read existing project memory:

```rust
repo.list_memory_entries("project", None, Some(project_id))
```

Compare normalized content against existing entries with status `proposed` or `accepted`.

- [ ] **Step 6: Implement `curate_memory_candidates`**

Signature:

```rust
pub fn curate_memory_candidates(
    repo: &Repository<'_>,
    input: MemoryCurationInput,
    hermes_version: String,
) -> JoiResult<MemoryCurationResult>
```

Flow:

1. Validate project exists via `build_project_context`.
2. Build candidate drafts from research and feedback.
3. Reject when no candidate material exists.
4. Create proposed `memory_entries`.
5. Create Agent run with `local_memory_bridge`.
6. Create five ordered events.
7. Return candidates and events.

- [ ] **Step 7: Run service tests**

```powershell
cd src-tauri
cargo test --test memory_curation -- --nocapture
```

Expected:

- Service tests pass.

### Task 3: Memory Commands

**Files:**

- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/commands.rs`

- [ ] **Step 1: Add failing command tests**

Extend command JSON round-trip:

```rust
let curation: MemoryCurationInput = serde_json::from_value(json!({
    "project_id": "project-1",
    "feedback_text": "Keep the opening shot more tactile.",
    "include_research_reports": true
})).unwrap();
```

Add helper test:

```rust
#[test]
fn state_helpers_generate_memory_candidates_and_update_status() {
    // seed brand/project/research report
    // generate candidates
    // update first candidate to accepted
    // assert list_memory_entries returns accepted status
}
```

- [ ] **Step 2: Run test and confirm RED**

```powershell
cd src-tauri
cargo test --test commands state_helpers_generate_memory_candidates_and_update_status -- --nocapture
```

- [ ] **Step 3: Add command handlers**

Add:

```rust
#[tauri::command(rename_all = "snake_case")]
pub fn joi_generate_memory_candidates(
    state: State<'_, AppState>,
    input: MemoryCurationInput,
) -> JoiResult<MemoryCurationResult>
```

Add:

```rust
#[tauri::command(rename_all = "snake_case")]
pub fn joi_update_memory_status(
    state: State<'_, AppState>,
    input: MemoryStatusInput,
) -> JoiResult<MemoryEntry>
```

- [ ] **Step 4: Register commands**

Register in `tauri::generate_handler!`:

- `commands::joi_generate_memory_candidates`
- `commands::joi_update_memory_status`

- [ ] **Step 5: Run command tests**

```powershell
cd src-tauri
cargo test --test commands -- --nocapture
```

Expected:

- Command tests pass.

### Task 4: Frontend Memory Workspace

**Files:**

- Modify: `src/types/joi.ts`
- Modify: `src/api/joiApi.ts`
- Modify: `src/App.tsx`
- Modify: `src/components/ProjectWorkspace.tsx`
- Test: `src/App.test.tsx`

- [ ] **Step 1: Add frontend types**

Add:

```ts
export type MemoryCurationInput = {
  project_id: string;
  feedback_text: string;
  include_research_reports: boolean;
};

export type MemoryCandidateResult = {
  entry: MemoryEntry;
  reason: string;
  has_conflict: boolean;
  conflict_memory_ids: string[];
};

export type MemoryCurationResult = {
  candidates: MemoryCandidateResult[];
  agent_run: AgentRun;
  agent_events: AgentRunEvent[];
};

export type MemoryStatusInput = {
  id: string;
  status: "proposed" | "accepted" | "rejected";
};
```

- [ ] **Step 2: Add API wrappers**

Add:

```ts
export function generateMemoryCandidates(input: MemoryCurationInput): Promise<MemoryCurationResult>
export function updateMemoryStatus(input: MemoryStatusInput): Promise<MemoryEntry>
```

- [ ] **Step 3: Add failing UI test**

Add a test that:

1. Opens Memory tab.
2. Enters feedback.
3. Clicks `Generate Memory Candidates`.
4. Asserts `joi_generate_memory_candidates` payload.
5. Sees a proposed candidate.
6. Clicks `Accept`.
7. Asserts `joi_update_memory_status` payload.

- [ ] **Step 4: Run UI test and confirm RED**

```powershell
npm test -- src/App.test.tsx
```

- [ ] **Step 5: Upgrade Memory panel**

Keep manual memory entry form.

Add:

- `feedback_text` textarea
- `include_research_reports` checkbox
- `Generate Memory Candidates` button
- Proposed section with accept/reject controls
- Accepted section
- Rejected section
- source trace display:
  - source
  - source_entity_type
  - source_entity_id
  - confidence
- conflict indicator when `has_conflict = true`

Keep UI quiet and work-focused; do not turn it into a marketing page.

- [ ] **Step 6: Wire App state**

Add state:

```ts
const [memoryCurationDraft, setMemoryCurationDraft] = useState({
  feedback_text: "",
  include_research_reports: true,
});
const [curatingMemory, setCuratingMemory] = useState(false);
const [memoryCurationResult, setMemoryCurationResult] = useState<MemoryCurationResult | null>(null);
```

Handlers:

- `submitMemoryCandidates`
- `updateMemoryStatus`

After status update:

- refresh project state
- keep activity log entry

After candidate generation:

- refresh project state
- add Agent run to `agentRuns`

- [ ] **Step 7: Run frontend tests and build**

```powershell
npm test
npm run build
```

Expected:

- Tests and build pass.

### Task 5: Smoke, Commit, Merge, Push

**Files:**

- Create: `docs/superpowers/reports/joi-0.15-practical-long-term-memory-smoke-test.md`

- [ ] **Step 1: Run full verification**

```powershell
npm test
npm run build
cd src-tauri
cargo test
cargo test --test memory_curation -- --nocapture
cargo test --test commands -- --nocapture
```

- [ ] **Step 2: Browser smoke**

Start:

```powershell
npm run dev -- --host 127.0.0.1 --port 1420
```

Verify:

- Memory tab renders manual memory form.
- Memory tab renders candidate generation controls.
- Proposed / Accepted / Rejected sections render.
- Generated candidate appears with source trace.
- Accept and Reject buttons are visible for proposed candidates.
- Desktop layout has no horizontal overflow.
- Mobile layout has no horizontal overflow.

Normal browser limitation:

- A normal browser still cannot call Tauri `invoke`; command integration is covered by Rust and React tests. Browser smoke may use a Tauri invoke mock.

- [ ] **Step 3: Write smoke report**

Include:

- commands run
- browser observations
- acceptance checklist
- known limitations

- [ ] **Step 4: Commit implementation**

Use focused commits:

1. `feat: add Joi 0.15 memory curation backend`
2. `feat: add Joi 0.15 memory workspace`
3. `test: add Joi 0.15 memory smoke report`

- [ ] **Step 5: Merge to main**

```powershell
git checkout main
git merge --ff-only codex/joi-0.15-practical-memory
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

If HTTPS push fails but GitHub API and SSH are reachable, use temporary writable deploy key fallback:

- create temporary ed25519 key in system temp directory
- add it to `Drew1811266/Joi-agent` as writable deploy key
- push over `ssh.github.com:443`
- delete deploy key
- delete local temp key
- verify remote main SHA through GitHub API

## Acceptance Criteria

0.15 is complete only when:

- User can generate proposed memory candidates from saved research reports.
- User can generate proposed memory candidates from feedback text.
- Each candidate has source trace and confidence.
- Duplicate candidate conflicts are detected and returned.
- User can accept or reject proposed memory.
- Memory tab groups proposed, accepted, and rejected memory.
- Accepted memory remains readable by Agent project context.
- Manual memory entry still works.
- Memory curation creates an Agent run and ordered events.
- Tests cover repository, service, commands, and frontend flow.
- Browser smoke report is written.
- Changes are merged to `main` and pushed to GitHub.

## Risks And Mitigations

### Risk: Memory Suggestions Become Noisy

Mitigation:

- 0.15 only proposes memory; it never auto-accepts.
- Candidate generation uses source-backed research implications and explicit user feedback.
- UI groups proposed memory separately from accepted memory.

### Risk: Conflict Detection Is Too Simple

Mitigation:

- Label it as deterministic duplicate detection in implementation comments/tests.
- Keep the output contract (`has_conflict`, `conflict_memory_ids`) ready for semantic conflict detection later.

### Risk: Brand And Project Memory Get Mixed

Mitigation:

- 0.15 curation writes project memory only.
- Existing repository validation continues enforcing brand/project scope rules.
- Future 0.16/0.17 work can add explicit brand-memory promotion.

### Risk: Research Report JSON Is Malformed

Mitigation:

- Parse `findings_json` defensively.
- Skip malformed findings rather than failing the whole curation task.
- Add tests with missing `creative_implication` fallback to `insight`.

## Handoff To 0.16

0.16 storyboard generation should read accepted project memory and use it when creating shot plans. Examples:

- accepted memory: `Use tactile close-ups as visual proof before the model movement.`
- storyboard effect: include a close fabric detail shot before or during model walk.

0.16 should not use rejected memory.

## Self-Review

- Spec coverage: This plan covers the 0.15 roadmap scope: proposed/accepted/rejected workflow, memory source trace, conflict detection, Agent memory curation, and UI controls.
- Placeholder scan: No task contains TBD/TODO/fill-in instructions. Commands, structs, files, tests, and expected outputs are named concretely.
- Type consistency: `MemoryCurationInput`, `MemoryCandidateResult`, `MemoryCurationResult`, `MemoryStatusInput`, `MemoryCandidateCreate`, and `MemoryStatusUpdate` are consistent across backend, frontend, and tests.
