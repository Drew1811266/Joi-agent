# Joi Agent 0.13 Agent Runtime Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Joi's first executable Agent runtime layer: a user can ask Joi to create a project task plan, Joi reads the saved project context, persists an agent run, writes visible execution events, and exposes Hermes Core readiness without replacing Joi's data model.

**Architecture:** 0.13 introduces Joi-owned `agent_runs` and `agent_run_events` tables, a Rust `agent_runtime` service, a `hermes_bridge` status adapter, and frontend Agent panel controls. The default 0.13 execution path is a deterministic local planning bridge so the feature works without API keys, while the bridge records Hermes Core metadata and isolates the subprocess boundary for later model-backed execution.

**Tech Stack:** Tauri 2 commands, Rust, rusqlite, serde/serde_json, chrono, React 19, TypeScript, Vitest, local `.external/hermes-agent` checkout.

---

## Product Outcome

After 0.13, a user can:

- Select a project that already has 0.12 brief/product understanding records.
- Open the Agent panel and enter a goal such as `Plan the next content workflow steps for this project`.
- Run Joi's first Agent task plan.
- See runtime status for the local Hermes Core checkout.
- See a visible execution log with role-based events.
- See a saved agent run attached to the project.
- Reopen the project and see previous agent runs.

0.13 does not generate final research reports, storyboards, or prompt packages. It creates the runtime and tool bridge foundation those later stages will use.

## Scope

### In Scope

- Agent run persistence:
  - `agent_runs`
  - `agent_run_events`
- Hermes Core status bridge:
  - checkout presence
  - version from `pyproject.toml`
  - Phase 0 capability report presence
  - configured runtime mode
- Joi project context builder:
  - brand
  - project
  - assets
  - latest product understanding
  - latest creative direction
  - project memory
  - latest versions
- Deterministic local agent plan generation:
  - planner
  - researcher
  - storyboard_writer
  - prompt_adapter
  - reviewer
  - memory_curator
- Tauri commands:
  - `joi_get_agent_runtime_status`
  - `joi_start_agent_plan`
  - `joi_get_agent_run`
  - `joi_list_agent_runs`
- Frontend Agent panel:
  - goal input
  - run button
  - runtime status
  - latest run summary
  - event log
- Tests and smoke report.

### Out Of Scope

- No external LLM call by default.
- No long-running autonomous background execution.
- No web research.
- No storyboard generation.
- No prompt generation.
- No Hermes memory replacement.
- No direct Hermes UI embedding.
- No cloud or messaging gateway.

## Key Design Decisions

### Decision 1: Joi Owns The Structured Run Model

Hermes is the runtime core, but Joi owns project state. Agent outputs must be persisted in Joi's structured tables first. 0.13 therefore adds `agent_runs` and `agent_run_events` instead of storing run state only in Hermes sessions.

### Decision 2: Bridge Hermes, Do Not Fork Data Into Hermes

The 0.13 `hermes_bridge` reports whether `.external/hermes-agent` is present and usable. It does not move Joi project data into Hermes memory, and it does not require Hermes API keys to pass tests.

### Decision 3: Deterministic Planning First

The first runtime behavior is deterministic planning from project context. This keeps 0.13 independently testable. Later versions can replace the local planner with a Hermes subprocess call behind the same service boundary.

### Decision 4: Agent Roles Are Data, Not Hardcoded UI Text

The run stores roles and plan steps as JSON so future model-backed runs can use the same schema.

## File Structure

### Backend

- Modify `src-tauri/src/db.rs`
  - Add `agent_runs` and `agent_run_events` tables and indexes.
- Modify `src-tauri/src/models.rs`
  - Add `AgentRun` and `AgentRunEvent`.
  - Add runtime status DTOs only if they are persisted; command DTOs stay in `commands.rs`.
- Modify `src-tauri/src/repositories.rs`
  - Add create/list/get methods for agent runs and events.
- Create `src-tauri/src/hermes_bridge.rs`
  - Inspect the local Hermes checkout and Phase 0 report.
  - Return a serializable runtime status.
- Create `src-tauri/src/agent_context.rs`
  - Build the project context read by the Agent runtime.
- Create `src-tauri/src/agent_runtime.rs`
  - Create deterministic plans.
  - Persist agent run and events.
- Modify `src-tauri/src/commands.rs`
  - Add runtime command DTOs and command handlers.
- Modify `src-tauri/src/lib.rs`
  - Register new modules and commands.
- Modify tests:
  - `src-tauri/tests/db_migration.rs`
  - `src-tauri/tests/structured_content_repository.rs`
  - `src-tauri/tests/commands.rs`
  - Add `src-tauri/tests/agent_runtime.rs`

### Frontend

- Modify `src/types/joi.ts`
  - Add agent run, event, runtime status, and command input/result types.
- Modify `src/api/joiApi.ts`
  - Add wrappers for new commands.
- Modify `src/App.tsx`
  - Load runtime status and project agent runs.
  - Add Agent goal draft and run handler.
- Modify `src/components/AgentPanel.tsx`
  - Render goal input, run button, runtime status, latest run, and event log.
- Modify `src/App.test.tsx`
  - Cover runtime status render and starting a plan.
- Add smoke report:
  - `docs/superpowers/reports/joi-0.13-agent-runtime-smoke-test.md`

## Data Model

### `agent_runs`

```sql
CREATE TABLE IF NOT EXISTS agent_runs (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  user_goal TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL DEFAULT 'completed',
  runtime_kind TEXT NOT NULL DEFAULT 'hermes_core',
  runtime_mode TEXT NOT NULL DEFAULT 'local_planner_bridge',
  runtime_version TEXT NOT NULL DEFAULT '',
  roles_json TEXT NOT NULL DEFAULT '[]',
  plan_json TEXT NOT NULL DEFAULT '[]',
  result_summary TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);
```

### `agent_run_events`

```sql
CREATE TABLE IF NOT EXISTS agent_run_events (
  id TEXT PRIMARY KEY,
  agent_run_id TEXT NOT NULL,
  sequence_number INTEGER NOT NULL,
  role TEXT NOT NULL DEFAULT '',
  event_type TEXT NOT NULL DEFAULT '',
  message TEXT NOT NULL DEFAULT '',
  payload_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL,
  FOREIGN KEY (agent_run_id) REFERENCES agent_runs(id) ON DELETE CASCADE,
  UNIQUE(agent_run_id, sequence_number)
);
```

### Runtime Role IDs

The first supported role IDs are:

```json
[
  "planner",
  "researcher",
  "storyboard_writer",
  "prompt_adapter",
  "reviewer",
  "memory_curator"
]
```

## Backend Contracts

### `AgentRuntimeStatus`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentRuntimeStatus {
    pub runtime_kind: String,
    pub runtime_mode: String,
    pub hermes_checkout_path: String,
    pub hermes_present: bool,
    pub hermes_version: String,
    pub phase0_report_present: bool,
    pub ready: bool,
    pub message: String,
}
```

Expected behavior:

- `ready = true` when `.external/hermes-agent/pyproject.toml` exists and version can be read.
- `runtime_kind = "hermes_core"`.
- `runtime_mode = "local_planner_bridge"`.
- If Hermes is missing, return `ready = false` and a clear message instead of failing the command.

### `AgentPlanInput`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentPlanInput {
    pub project_id: String,
    pub user_goal: String,
}
```

Validation:

- `project_id` must exist.
- `user_goal` must be non-empty after trim.

### `AgentPlanResult`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPlanResult {
    pub run: AgentRun,
    pub events: Vec<AgentRunEvent>,
}
```

Expected behavior:

- Reads project context from Joi's repository.
- Creates one `AgentRun` with status `completed`.
- Creates ordered events:
  - sequence 1: planner reads context
  - sequence 2: planner creates task plan
  - sequence 3: researcher queues research task
  - sequence 4: storyboard_writer queues storyboard task
  - sequence 5: prompt_adapter queues prompt task
  - sequence 6: reviewer queues review task
  - sequence 7: memory_curator queues memory task
- Stores `plan_json` with role, title, rationale, required context, and next_version fields.
- Returns saved run and events.

## Implementation Tasks

### Task 1: Agent Run Schema And Repository

**Files:**

- Modify: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/repositories.rs`
- Test: `src-tauri/tests/db_migration.rs`
- Test: `src-tauri/tests/structured_content_repository.rs`

- [ ] **Step 1: Write failing migration test**

Add to `src-tauri/tests/db_migration.rs`:

```rust
#[test]
fn migration_creates_agent_run_tables() {
    let app = TestApp::new();
    let names = app.db.table_names().expect("table names");

    assert!(names.contains(&"agent_runs".to_string()));
    assert!(names.contains(&"agent_run_events".to_string()));
}
```

- [ ] **Step 2: Run migration test and confirm RED**

Run:

```powershell
cd src-tauri
cargo test --test db_migration migration_creates_agent_run_tables -- --nocapture
```

Expected:

- Fails because `agent_runs` and `agent_run_events` do not exist.

- [ ] **Step 3: Add schema tables and indexes**

Append the SQL tables from the Data Model section to `SCHEMA` in `src-tauri/src/db.rs`.

Add indexes:

```sql
CREATE INDEX IF NOT EXISTS idx_agent_runs_project_id ON agent_runs(project_id);
CREATE INDEX IF NOT EXISTS idx_agent_run_events_run_id ON agent_run_events(agent_run_id);
```

- [ ] **Step 4: Run migration test and confirm GREEN**

Run:

```powershell
cd src-tauri
cargo test --test db_migration migration_creates_agent_run_tables -- --nocapture
```

Expected:

- Test passes.

- [ ] **Step 5: Write failing repository test**

Add to `src-tauri/tests/structured_content_repository.rs`:

```rust
#[test]
fn stores_agent_runs_and_ordered_events() {
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
            advertising_goal: "Launch awareness".to_string(),
            duration_seconds: 15,
        })
        .unwrap();

    let run = repo
        .create_agent_run(AgentRunCreate {
            project_id: project.id.clone(),
            user_goal: "Plan the next workflow steps".to_string(),
            status: "completed".to_string(),
            runtime_kind: "hermes_core".to_string(),
            runtime_mode: "local_planner_bridge".to_string(),
            runtime_version: "0.16.0".to_string(),
            roles: vec!["planner".to_string(), "reviewer".to_string()],
            plan: serde_json::json!([
                {"role":"planner","title":"Read project context"}
            ]),
            result_summary: "Created a project task plan.".to_string(),
        })
        .unwrap();

    repo.create_agent_run_event(AgentRunEventCreate {
        agent_run_id: run.id.clone(),
        sequence_number: 1,
        role: "planner".to_string(),
        event_type: "context_read".to_string(),
        message: "Read project context.".to_string(),
        payload: serde_json::json!({"project_id": project.id}),
    })
    .unwrap();

    let runs = repo.list_agent_runs(&project.id).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].user_goal, "Plan the next workflow steps");
    assert_eq!(runs[0].roles_json, serde_json::json!(["planner", "reviewer"]));

    let events = repo.list_agent_run_events(&run.id).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].sequence_number, 1);
    assert_eq!(events[0].role, "planner");
}
```

Add imports:

```rust
use joi_agent_lib::repositories::{AgentRunCreate, AgentRunEventCreate};
```

- [ ] **Step 6: Run repository test and confirm RED**

Run:

```powershell
cd src-tauri
cargo test --test structured_content_repository stores_agent_runs_and_ordered_events -- --nocapture
```

Expected:

- Fails because `AgentRunCreate`, `AgentRunEventCreate`, and repository methods do not exist.

- [ ] **Step 7: Add models**

Add to `src-tauri/src/models.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRun {
    pub id: String,
    pub project_id: String,
    pub user_goal: String,
    pub status: String,
    pub runtime_kind: String,
    pub runtime_mode: String,
    pub runtime_version: String,
    pub roles_json: Value,
    pub plan_json: Value,
    pub result_summary: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRunEvent {
    pub id: String,
    pub agent_run_id: String,
    pub sequence_number: i64,
    pub role: String,
    pub event_type: String,
    pub message: String,
    pub payload_json: Value,
    pub created_at: DateTime<Utc>,
}
```

- [ ] **Step 8: Add repository create/list/get methods**

Add structs to `src-tauri/src/repositories.rs`:

```rust
#[derive(Debug, Clone)]
pub struct AgentRunCreate {
    pub project_id: String,
    pub user_goal: String,
    pub status: String,
    pub runtime_kind: String,
    pub runtime_mode: String,
    pub runtime_version: String,
    pub roles: Vec<String>,
    pub plan: serde_json::Value,
    pub result_summary: String,
}

#[derive(Debug, Clone)]
pub struct AgentRunEventCreate {
    pub agent_run_id: String,
    pub sequence_number: i64,
    pub role: String,
    pub event_type: String,
    pub message: String,
    pub payload: serde_json::Value,
}
```

Add methods:

```rust
pub fn create_agent_run(&self, input: AgentRunCreate) -> JoiResult<AgentRun> {
    validate_required_text("Agent run goal", &input.user_goal)?;
    self.get_project(&input.project_id)?;
    let now = Utc::now();
    let run = AgentRun {
        id: new_id(),
        project_id: input.project_id,
        user_goal: input.user_goal.trim().to_string(),
        status: input.status,
        runtime_kind: input.runtime_kind,
        runtime_mode: input.runtime_mode,
        runtime_version: input.runtime_version,
        roles_json: json!(input.roles),
        plan_json: input.plan,
        result_summary: input.result_summary,
        created_at: now,
        updated_at: now,
    };
    self.connection.execute(
        "INSERT INTO agent_runs (
            id, project_id, user_goal, status, runtime_kind, runtime_mode, runtime_version,
            roles_json, plan_json, result_summary, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            run.id,
            run.project_id,
            run.user_goal,
            run.status,
            run.runtime_kind,
            run.runtime_mode,
            run.runtime_version,
            run.roles_json.to_string(),
            run.plan_json.to_string(),
            run.result_summary,
            run.created_at.to_rfc3339(),
            run.updated_at.to_rfc3339()
        ],
    )?;
    Ok(run)
}

pub fn get_agent_run(&self, id: &str) -> JoiResult<AgentRun> {
    self.connection
        .query_row(
            "SELECT id, project_id, user_goal, status, runtime_kind, runtime_mode, runtime_version,
                    roles_json, plan_json, result_summary, created_at, updated_at
             FROM agent_runs WHERE id = ?1",
            params![id],
            map_agent_run,
        )
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => JoiError::NotFound(format!("agent run {}", id)),
            other => other.into(),
        })
}

pub fn list_agent_runs(&self, project_id: &str) -> JoiResult<Vec<AgentRun>> {
    let mut statement = self.connection.prepare(
        "SELECT id, project_id, user_goal, status, runtime_kind, runtime_mode, runtime_version,
                roles_json, plan_json, result_summary, created_at, updated_at
         FROM agent_runs WHERE project_id = ?1 ORDER BY created_at ASC",
    )?;
    let rows = statement.query_map(params![project_id], map_agent_run)?;
    collect_rows(rows)
}

pub fn create_agent_run_event(&self, input: AgentRunEventCreate) -> JoiResult<AgentRunEvent> {
    self.get_agent_run(&input.agent_run_id)?;
    let now = Utc::now();
    let event = AgentRunEvent {
        id: new_id(),
        agent_run_id: input.agent_run_id,
        sequence_number: input.sequence_number,
        role: input.role,
        event_type: input.event_type,
        message: input.message,
        payload_json: input.payload,
        created_at: now,
    };
    self.connection.execute(
        "INSERT INTO agent_run_events (
            id, agent_run_id, sequence_number, role, event_type, message, payload_json, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            event.id,
            event.agent_run_id,
            event.sequence_number,
            event.role,
            event.event_type,
            event.message,
            event.payload_json.to_string(),
            event.created_at.to_rfc3339()
        ],
    )?;
    Ok(event)
}

pub fn list_agent_run_events(&self, run_id: &str) -> JoiResult<Vec<AgentRunEvent>> {
    let mut statement = self.connection.prepare(
        "SELECT id, agent_run_id, sequence_number, role, event_type, message, payload_json, created_at
         FROM agent_run_events WHERE agent_run_id = ?1 ORDER BY sequence_number ASC",
    )?;
    let rows = statement.query_map(params![run_id], map_agent_run_event)?;
    collect_rows(rows)
}
```

Add mappers using existing `parse_json` and `parse_time` helpers.

- [ ] **Step 9: Run repository tests**

Run:

```powershell
cd src-tauri
cargo test --test structured_content_repository stores_agent_runs_and_ordered_events -- --nocapture
cargo test --test db_migration migration_creates_agent_run_tables -- --nocapture
```

Expected:

- Both tests pass.

### Task 2: Hermes Runtime Status Bridge

**Files:**

- Create: `src-tauri/src/hermes_bridge.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/agent_runtime.rs`

- [ ] **Step 1: Write failing Hermes status tests**

Create `src-tauri/tests/agent_runtime.rs`:

```rust
mod common;

use std::fs;

use joi_agent_lib::hermes_bridge::{inspect_hermes_runtime, HermesRuntimeConfig};

#[test]
fn reports_ready_hermes_runtime_from_checkout_fixture() {
    let app = common::TestApp::new();
    let checkout = app.temp_dir.path().join("hermes-agent");
    fs::create_dir_all(&checkout).unwrap();
    fs::write(
        checkout.join("pyproject.toml"),
        "[project]\nname = \"hermes-agent\"\nversion = \"0.16.0\"\nrequires-python = \">=3.11,<3.14\"\n",
    )
    .unwrap();
    let report = app.temp_dir.path().join("hermes-phase0-report.md");
    fs::write(&report, "Status: pass\n").unwrap();

    let status = inspect_hermes_runtime(HermesRuntimeConfig {
        checkout_path: checkout,
        phase0_report_path: report,
        runtime_mode: "local_planner_bridge".to_string(),
    });

    assert!(status.ready);
    assert!(status.hermes_present);
    assert_eq!(status.hermes_version, "0.16.0");
    assert!(status.phase0_report_present);
}

#[test]
fn reports_not_ready_when_hermes_checkout_is_missing() {
    let app = common::TestApp::new();
    let status = inspect_hermes_runtime(HermesRuntimeConfig {
        checkout_path: app.temp_dir.path().join("missing"),
        phase0_report_path: app.temp_dir.path().join("missing-report.md"),
        runtime_mode: "local_planner_bridge".to_string(),
    });

    assert!(!status.ready);
    assert!(!status.hermes_present);
    assert!(status.message.contains("Hermes checkout not found"));
}
```

- [ ] **Step 2: Run tests and confirm RED**

Run:

```powershell
cd src-tauri
cargo test --test agent_runtime -- --nocapture
```

Expected:

- Fails because `hermes_bridge` does not exist.

- [ ] **Step 3: Implement `hermes_bridge.rs`**

Create `src-tauri/src/hermes_bridge.rs`:

```rust
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct HermesRuntimeConfig {
    pub checkout_path: PathBuf,
    pub phase0_report_path: PathBuf,
    pub runtime_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentRuntimeStatus {
    pub runtime_kind: String,
    pub runtime_mode: String,
    pub hermes_checkout_path: String,
    pub hermes_present: bool,
    pub hermes_version: String,
    pub phase0_report_present: bool,
    pub ready: bool,
    pub message: String,
}

pub fn inspect_hermes_runtime(config: HermesRuntimeConfig) -> AgentRuntimeStatus {
    let pyproject = config.checkout_path.join("pyproject.toml");
    if !pyproject.exists() {
        return AgentRuntimeStatus {
            runtime_kind: "hermes_core".to_string(),
            runtime_mode: config.runtime_mode,
            hermes_checkout_path: config.checkout_path.display().to_string(),
            hermes_present: false,
            hermes_version: String::new(),
            phase0_report_present: config.phase0_report_path.exists(),
            ready: false,
            message: "Hermes checkout not found. Expected .external/hermes-agent/pyproject.toml.".to_string(),
        };
    }

    let text = std::fs::read_to_string(&pyproject).unwrap_or_default();
    let version = parse_project_version(&text);
    let ready = !version.is_empty();
    AgentRuntimeStatus {
        runtime_kind: "hermes_core".to_string(),
        runtime_mode: config.runtime_mode,
        hermes_checkout_path: config.checkout_path.display().to_string(),
        hermes_present: true,
        hermes_version: version,
        phase0_report_present: config.phase0_report_path.exists(),
        ready,
        message: if ready {
            "Hermes Core checkout detected. Joi is using the local planner bridge.".to_string()
        } else {
            "Hermes checkout found but version could not be read.".to_string()
        },
    }
}

pub fn parse_project_version(pyproject_text: &str) -> String {
    pyproject_text
        .lines()
        .map(str::trim)
        .find_map(|line| {
            line.strip_prefix("version")
                .and_then(|rest| rest.split_once('='))
                .map(|(_, value)| value.trim().trim_matches('"').to_string())
        })
        .unwrap_or_default()
}
```

Modify `src-tauri/src/lib.rs`:

```rust
pub mod hermes_bridge;
```

- [ ] **Step 4: Run Hermes status tests**

Run:

```powershell
cd src-tauri
cargo test --test agent_runtime reports_ready_hermes_runtime_from_checkout_fixture -- --nocapture
cargo test --test agent_runtime reports_not_ready_when_hermes_checkout_is_missing -- --nocapture
```

Expected:

- Both tests pass.

### Task 3: Agent Context And Local Planning Runtime

**Files:**

- Create: `src-tauri/src/agent_context.rs`
- Create: `src-tauri/src/agent_runtime.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/agent_runtime.rs`

- [ ] **Step 1: Add failing context and plan test**

Append to `src-tauri/tests/agent_runtime.rs`:

```rust
use joi_agent_lib::agent_runtime::{start_agent_plan, AgentPlanInput};
use joi_agent_lib::repositories::{
    BrandCreate, CreativeDirectionCreate, ProductUnderstandingCreate, Repository,
};

#[test]
fn starts_agent_plan_from_saved_project_context() {
    let app = common::TestApp::new();
    let repo = Repository::new(app.db.connection());
    let brand = repo
        .create_brand(BrandCreate {
            name: "Atelier Joi".to_string(),
            description: "Editorial womenswear".to_string(),
        })
        .unwrap();
    let project = repo
        .create_project(joi_agent_lib::repositories::ProjectCreate {
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
        notes: "{}".to_string(),
    })
    .unwrap();
    repo.create_creative_direction(CreativeDirectionCreate {
        project_id: project.id.clone(),
        title: "Initial visual direction".to_string(),
        concept: "Clean studio walk".to_string(),
        tone: "premium".to_string(),
        visual_style: "soft daylight".to_string(),
        scene_direction: "model walk and fabric close-up".to_string(),
        rationale: "Use movement to show fabric.".to_string(),
    })
    .unwrap();

    let result = start_agent_plan(
        &repo,
        AgentPlanInput {
            project_id: project.id.clone(),
            user_goal: "Plan the next content workflow steps".to_string(),
        },
        "0.16.0",
    )
    .unwrap();

    assert_eq!(result.run.project_id, project.id);
    assert_eq!(result.run.status, "completed");
    assert!(result.run.result_summary.contains("Created"));
    assert_eq!(result.events.len(), 7);
    assert_eq!(result.events[0].role, "planner");
    assert_eq!(result.events[0].event_type, "context_read");

    let saved_runs = repo.list_agent_runs(&project.id).unwrap();
    assert_eq!(saved_runs.len(), 1);
}
```

- [ ] **Step 2: Run test and confirm RED**

Run:

```powershell
cd src-tauri
cargo test --test agent_runtime starts_agent_plan_from_saved_project_context -- --nocapture
```

Expected:

- Fails because `agent_runtime` does not exist.

- [ ] **Step 3: Implement `agent_context.rs`**

Create `src-tauri/src/agent_context.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::error::JoiResult;
use crate::models::{Asset, Brand, CreativeDirection, MemoryEntry, ProductUnderstanding, Project, ProjectVersion};
use crate::repositories::Repository;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProjectContext {
    pub brand: Brand,
    pub project: Project,
    pub assets: Vec<Asset>,
    pub latest_product_understanding: Option<ProductUnderstanding>,
    pub latest_creative_direction: Option<CreativeDirection>,
    pub project_memory: Vec<MemoryEntry>,
    pub versions: Vec<ProjectVersion>,
}

pub fn build_project_context(repo: &Repository<'_>, project_id: &str) -> JoiResult<AgentProjectContext> {
    let project = repo.get_project(project_id)?;
    let brand = repo.get_brand(&project.brand_id)?;
    let assets = repo.list_assets(project_id)?;
    let mut understandings = repo.list_product_understandings(project_id)?;
    let mut directions = repo.list_creative_directions(project_id)?;
    let project_memory = repo.list_memory_entries_for_project(project_id)?;
    let versions = repo.list_project_versions(project_id)?;

    let latest_product_understanding = understandings.pop();
    let latest_creative_direction = directions.pop();

    Ok(AgentProjectContext {
        brand,
        project,
        assets,
        latest_product_understanding,
        latest_creative_direction,
        project_memory,
        versions,
    })
}
```

Modify `src-tauri/src/lib.rs`:

```rust
pub mod agent_context;
```

- [ ] **Step 4: Implement `agent_runtime.rs`**

Create `src-tauri/src/agent_runtime.rs`:

```rust
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::agent_context::{build_project_context, AgentProjectContext};
use crate::error::{JoiError, JoiResult};
use crate::models::{AgentRun, AgentRunEvent};
use crate::repositories::{AgentRunCreate, AgentRunEventCreate, Repository};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentPlanInput {
    pub project_id: String,
    pub user_goal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPlanResult {
    pub run: AgentRun,
    pub events: Vec<AgentRunEvent>,
}

const ROLES: [&str; 6] = [
    "planner",
    "researcher",
    "storyboard_writer",
    "prompt_adapter",
    "reviewer",
    "memory_curator",
];

pub fn start_agent_plan(
    repo: &Repository<'_>,
    input: AgentPlanInput,
    hermes_version: &str,
) -> JoiResult<AgentPlanResult> {
    if input.user_goal.trim().is_empty() {
        return Err(JoiError::Validation("Agent goal is required".to_string()));
    }

    let context = build_project_context(repo, &input.project_id)?;
    let plan = build_plan_json(&context);
    let summary = format!(
        "Created {} task steps for {}.",
        plan.as_array().map(Vec::len).unwrap_or(0),
        context.project.title
    );
    let run = repo.create_agent_run(AgentRunCreate {
        project_id: context.project.id.clone(),
        user_goal: input.user_goal,
        status: "completed".to_string(),
        runtime_kind: "hermes_core".to_string(),
        runtime_mode: "local_planner_bridge".to_string(),
        runtime_version: hermes_version.to_string(),
        roles: ROLES.iter().map(|role| role.to_string()).collect(),
        plan,
        result_summary: summary,
    })?;

    let event_specs = build_event_specs(&context);
    let mut events = Vec::with_capacity(event_specs.len());
    for (index, (role, event_type, message, payload)) in event_specs.into_iter().enumerate() {
        events.push(repo.create_agent_run_event(AgentRunEventCreate {
            agent_run_id: run.id.clone(),
            sequence_number: (index + 1) as i64,
            role,
            event_type,
            message,
            payload,
        })?);
    }

    Ok(AgentPlanResult { run, events })
}

fn build_plan_json(context: &AgentProjectContext) -> serde_json::Value {
    json!([
        {
            "role": "planner",
            "title": "Confirm brief and material context",
            "rationale": "Use saved Joi project context before creating downstream content.",
            "required_context": ["project", "brand", "product_understanding", "creative_direction"],
            "next_version": "0.13"
        },
        {
            "role": "researcher",
            "title": "Prepare research questions",
            "rationale": "0.14 will turn these questions into sourced research.",
            "required_context": ["brand", "product_understanding", "reference_assets"],
            "next_version": "0.14"
        },
        {
            "role": "storyboard_writer",
            "title": "Prepare storyboard generation inputs",
            "rationale": "0.16 needs validated product, duration, and visual direction context.",
            "required_context": ["duration_seconds", "selling_points", "visual_direction"],
            "next_version": "0.16"
        },
        {
            "role": "prompt_adapter",
            "title": "Prepare prompt adapter targets",
            "rationale": "0.17 will map shots and image briefs to Jimeng, Grok, Banana 2, Jimeng Image, and GPT Image 2.",
            "required_context": ["target_platforms", "assets"],
            "next_version": "0.17"
        },
        {
            "role": "reviewer",
            "title": "Prepare quality review checklist",
            "rationale": "0.19 will review duration, brand consistency, garment visibility, and prompt completeness.",
            "required_context": ["storyboard", "prompt_packages"],
            "next_version": "0.19"
        },
        {
            "role": "memory_curator",
            "title": "Prepare memory capture points",
            "rationale": "Accepted user decisions should become reusable brand or project memory.",
            "required_context": ["project_memory", "user_feedback"],
            "next_version": "0.15"
        }
    ])
}

fn build_event_specs(context: &AgentProjectContext) -> Vec<(String, String, String, serde_json::Value)> {
    vec![
        (
            "planner".to_string(),
            "context_read".to_string(),
            format!("Read project context for {}.", context.project.title),
            json!({
                "brand": context.brand.name,
                "assets": context.assets.len(),
                "has_product_understanding": context.latest_product_understanding.is_some(),
                "has_creative_direction": context.latest_creative_direction.is_some(),
                "memory_entries": context.project_memory.len()
            }),
        ),
        (
            "planner".to_string(),
            "plan_created".to_string(),
            "Created role-based task plan.".to_string(),
            json!({"roles": ROLES}),
        ),
        ("researcher".to_string(), "task_queued".to_string(), "Queued research preparation task.".to_string(), json!({"target_version":"0.14"})),
        ("storyboard_writer".to_string(), "task_queued".to_string(), "Queued storyboard preparation task.".to_string(), json!({"target_version":"0.16"})),
        ("prompt_adapter".to_string(), "task_queued".to_string(), "Queued prompt adapter preparation task.".to_string(), json!({"target_version":"0.17"})),
        ("reviewer".to_string(), "task_queued".to_string(), "Queued quality review preparation task.".to_string(), json!({"target_version":"0.19"})),
        ("memory_curator".to_string(), "task_queued".to_string(), "Queued memory curation preparation task.".to_string(), json!({"target_version":"0.15"})),
    ]
}
```

Modify `src-tauri/src/lib.rs`:

```rust
pub mod agent_runtime;
```

- [ ] **Step 5: Run agent runtime tests**

Run:

```powershell
cd src-tauri
cargo test --test agent_runtime -- --nocapture
```

Expected:

- All agent runtime tests pass.

### Task 4: Tauri Commands For Agent Runtime

**Files:**

- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/commands.rs`

- [ ] **Step 1: Add failing command test**

Add imports in `src-tauri/tests/commands.rs`:

```rust
use joi_agent_lib::commands::{
    get_agent_runtime_status, get_agent_run, list_agent_runs, start_agent_plan, AgentPlanInput,
};
```

Add test:

```rust
#[test]
fn starts_agent_plan_through_command_helpers() {
    let (_app, state) = test_state();
    let brand = create_brand(
        &state,
        BrandInput {
            name: "Atelier Joi".to_string(),
            description: "Editorial womenswear".to_string(),
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

    let result = start_agent_plan(
        &state,
        AgentPlanInput {
            project_id: project.id.clone(),
            user_goal: "Plan the next workflow steps".to_string(),
        },
    )
    .unwrap();

    assert_eq!(result.run.project_id, project.id);
    assert_eq!(result.events.len(), 7);
    let runs = list_agent_runs(&state, project.id).unwrap();
    assert_eq!(runs.len(), 1);
    let loaded = get_agent_run(&state, result.run.id).unwrap();
    assert_eq!(loaded.events.len(), 7);

    let status = get_agent_runtime_status(&state).unwrap();
    assert_eq!(status.runtime_kind, "hermes_core");
}
```

- [ ] **Step 2: Run command test and confirm RED**

Run:

```powershell
cd src-tauri
cargo test --test commands starts_agent_plan_through_command_helpers -- --nocapture
```

Expected:

- Fails because command helpers and DTOs do not exist.

- [ ] **Step 3: Add command DTOs and helpers**

In `src-tauri/src/commands.rs`, import:

```rust
use crate::agent_runtime::{start_agent_plan as run_agent_plan, AgentPlanResult};
use crate::agent_runtime::AgentPlanInput;
use crate::hermes_bridge::{inspect_hermes_runtime, AgentRuntimeStatus, HermesRuntimeConfig};
use crate::models::{AgentRun, AgentRunEvent};
```

Add result DTO:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRunWithEvents {
    pub run: AgentRun,
    pub events: Vec<AgentRunEvent>,
}
```

Add Tauri commands:

```rust
#[tauri::command(rename_all = "snake_case")]
pub fn joi_get_agent_runtime_status(state: State<'_, AppState>) -> JoiResult<AgentRuntimeStatus> {
    get_agent_runtime_status(state.inner())
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_start_agent_plan(
    state: State<'_, AppState>,
    input: AgentPlanInput,
) -> JoiResult<AgentPlanResult> {
    start_agent_plan(state.inner(), input)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_get_agent_run(state: State<'_, AppState>, run_id: String) -> JoiResult<AgentRunWithEvents> {
    get_agent_run(state.inner(), run_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn joi_list_agent_runs(state: State<'_, AppState>, project_id: String) -> JoiResult<Vec<AgentRun>> {
    list_agent_runs(state.inner(), project_id)
}
```

Add helper functions:

```rust
pub fn get_agent_runtime_status(state: &AppState) -> JoiResult<AgentRuntimeStatus> {
    Ok(inspect_hermes_runtime(HermesRuntimeConfig {
        checkout_path: std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".external")
            .join("hermes-agent"),
        phase0_report_path: std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("docs")
            .join("superpowers")
            .join("reports")
            .join("hermes-phase0-report.md"),
        runtime_mode: "local_planner_bridge".to_string(),
    }))
}

pub fn start_agent_plan(state: &AppState, input: AgentPlanInput) -> JoiResult<AgentPlanResult> {
    let runtime_status = get_agent_runtime_status(state)?;
    let db = lock_db(state)?;
    run_agent_plan(
        &Repository::new(db.connection()),
        input,
        &runtime_status.hermes_version,
    )
}

pub fn get_agent_run(state: &AppState, run_id: String) -> JoiResult<AgentRunWithEvents> {
    let db = lock_db(state)?;
    let repo = Repository::new(db.connection());
    let run = repo.get_agent_run(&run_id)?;
    let events = repo.list_agent_run_events(&run.id)?;
    Ok(AgentRunWithEvents { run, events })
}

pub fn list_agent_runs(state: &AppState, project_id: String) -> JoiResult<Vec<AgentRun>> {
    let db = lock_db(state)?;
    Repository::new(db.connection()).list_agent_runs(&project_id)
}
```

- [ ] **Step 4: Register commands**

In `src-tauri/src/lib.rs`, add command registrations:

```rust
commands::joi_get_agent_runtime_status,
commands::joi_start_agent_plan,
commands::joi_get_agent_run,
commands::joi_list_agent_runs,
```

- [ ] **Step 5: Run command tests**

Run:

```powershell
cd src-tauri
cargo test --test commands starts_agent_plan_through_command_helpers -- --nocapture
cargo test --test commands -- --nocapture
```

Expected:

- Command tests pass.

### Task 5: Frontend Agent Panel Integration

**Files:**

- Modify: `src/types/joi.ts`
- Modify: `src/api/joiApi.ts`
- Modify: `src/App.tsx`
- Modify: `src/components/AgentPanel.tsx`
- Test: `src/App.test.tsx`

- [ ] **Step 1: Add failing frontend test**

Add mock cases in `src/App.test.tsx`:

```ts
case "joi_get_agent_runtime_status":
  return Promise.resolve({
    runtime_kind: "hermes_core",
    runtime_mode: "local_planner_bridge",
    hermes_checkout_path: "D:/Software Project/Joi-agent/.external/hermes-agent",
    hermes_present: true,
    hermes_version: "0.16.0",
    phase0_report_present: true,
    ready: true,
    message: "Hermes Core checkout detected. Joi is using the local planner bridge.",
  });
case "joi_list_agent_runs":
  return Promise.resolve([]);
case "joi_start_agent_plan":
  return Promise.resolve({
    run: {
      id: "run-1",
      project_id: "project-1",
      user_goal: "Plan the next workflow steps",
      status: "completed",
      runtime_kind: "hermes_core",
      runtime_mode: "local_planner_bridge",
      runtime_version: "0.16.0",
      roles_json: ["planner", "researcher", "storyboard_writer", "prompt_adapter", "reviewer", "memory_curator"],
      plan_json: [{"role":"planner","title":"Confirm brief and material context"}],
      result_summary: "Created 6 task steps for Spring Drop Film.",
      created_at: "2026-06-15T00:00:00Z",
      updated_at: "2026-06-15T00:00:00Z",
    },
    events: [
      {
        id: "event-1",
        agent_run_id: "run-1",
        sequence_number: 1,
        role: "planner",
        event_type: "context_read",
        message: "Read project context for Spring Drop Film.",
        payload_json: {},
        created_at: "2026-06-15T00:00:00Z",
      },
    ],
  });
```

Add test:

```ts
test("starts an agent plan from the Agent panel", async () => {
  render(<App />);

  await screen.findByRole("heading", { name: "Spring Drop Film" });
  expect(await screen.findByText("Hermes Core")).toBeInTheDocument();
  expect(await screen.findByText("0.16.0")).toBeInTheDocument();

  fireEvent.change(screen.getByLabelText("Agent goal"), {
    target: { value: "Plan the next workflow steps" },
  });
  fireEvent.click(screen.getByRole("button", { name: /start plan/i }));

  await waitFor(() => {
    expect(invokeMock).toHaveBeenCalledWith("joi_start_agent_plan", {
      input: {
        project_id: "project-1",
        user_goal: "Plan the next workflow steps",
      },
    });
  });
  expect(await screen.findByText("Created 6 task steps for Spring Drop Film.")).toBeInTheDocument();
  expect(await screen.findByText("Read project context for Spring Drop Film.")).toBeInTheDocument();
});
```

- [ ] **Step 2: Run frontend test and confirm RED**

Run:

```powershell
npm test -- src/App.test.tsx
```

Expected:

- Fails because Agent panel has no runtime controls.

- [ ] **Step 3: Add frontend types**

Add to `src/types/joi.ts`:

```ts
export type AgentRun = {
  id: string;
  project_id: string;
  user_goal: string;
  status: string;
  runtime_kind: string;
  runtime_mode: string;
  runtime_version: string;
  roles_json: unknown;
  plan_json: unknown;
  result_summary: string;
  created_at: string;
  updated_at: string;
};

export type AgentRunEvent = {
  id: string;
  agent_run_id: string;
  sequence_number: number;
  role: string;
  event_type: string;
  message: string;
  payload_json: unknown;
  created_at: string;
};

export type AgentRuntimeStatus = {
  runtime_kind: string;
  runtime_mode: string;
  hermes_checkout_path: string;
  hermes_present: boolean;
  hermes_version: string;
  phase0_report_present: boolean;
  ready: boolean;
  message: string;
};

export type AgentPlanInput = {
  project_id: string;
  user_goal: string;
};

export type AgentPlanResult = {
  run: AgentRun;
  events: AgentRunEvent[];
};

export type AgentRunWithEvents = {
  run: AgentRun;
  events: AgentRunEvent[];
};
```

- [ ] **Step 4: Add API wrappers**

Add to `src/api/joiApi.ts`:

```ts
export function getAgentRuntimeStatus(): Promise<AgentRuntimeStatus> {
  return invoke<AgentRuntimeStatus>("joi_get_agent_runtime_status");
}

export function startAgentPlan(input: AgentPlanInput): Promise<AgentPlanResult> {
  return invoke<AgentPlanResult>("joi_start_agent_plan", { input });
}

export function getAgentRun(runId: string): Promise<AgentRunWithEvents> {
  return invoke<AgentRunWithEvents>("joi_get_agent_run", { run_id: runId });
}

export function listAgentRuns(projectId: string): Promise<AgentRun[]> {
  return invoke<AgentRun[]>("joi_list_agent_runs", { project_id: projectId });
}
```

- [ ] **Step 5: Add App state and handlers**

In `src/App.tsx`, add state:

```ts
const [agentGoalDraft, setAgentGoalDraft] = useState("");
const [agentRuntimeStatus, setAgentRuntimeStatus] = useState<AgentRuntimeStatus | null>(null);
const [agentRuns, setAgentRuns] = useState<AgentRun[]>([]);
const [latestAgentRun, setLatestAgentRun] = useState<AgentRunWithEvents | null>(null);
const [startingAgentPlan, setStartingAgentPlan] = useState(false);
```

In `loadInitialState`, load runtime status:

```ts
const [healthResult, brandList, runtimeStatus] = await Promise.all([
  healthCheck(),
  listBrands(),
  getAgentRuntimeStatus(),
]);
setAgentRuntimeStatus(runtimeStatus);
```

In `refreshProjectState`, add:

```ts
listAgentRuns(projectId),
```

and:

```ts
setAgentRuns(runList);
```

Add handler:

```ts
async function submitAgentPlan() {
  if (!selectedProject) {
    setError("Select a project before starting an agent plan.");
    return;
  }
  if (!agentGoalDraft.trim()) {
    setError("Agent goal is required.");
    return;
  }

  try {
    setStartingAgentPlan(true);
    setError(null);
    const result = await startAgentPlan({
      project_id: selectedProject.id,
      user_goal: agentGoalDraft,
    });
    setLatestAgentRun(result);
    setAgentGoalDraft("");
    await refreshProjectState(selectedProject.id);
    setActivityLog((entries) => [...entries, `Started agent run ${result.run.id}.`]);
  } catch (submitError) {
    setError(formatError(submitError));
  } finally {
    setStartingAgentPlan(false);
  }
}
```

- [ ] **Step 6: Update AgentPanel props and UI**

Update `src/components/AgentPanel.tsx` props:

```ts
type AgentPanelProps = {
  activityLog: string[];
  agentGoalDraft: string;
  agentRuntimeStatus: AgentRuntimeStatus | null;
  agentRuns: AgentRun[];
  latestAgentRun: AgentRunWithEvents | null;
  onAgentGoalDraftChange: (value: string) => void;
  onSubmitAgentPlan: () => void;
  selectedBrand: Brand | null;
  selectedProject: Project | null;
  startingAgentPlan: boolean;
};
```

Render a form:

```tsx
<section className="panel-section">
  <h3>Task Run</h3>
  <form onSubmit={submit(onSubmitAgentPlan)}>
    <label>
      Agent goal
      <textarea
        disabled={!selectedProject || startingAgentPlan}
        onChange={(event) => onAgentGoalDraftChange(event.target.value)}
        rows={3}
        value={agentGoalDraft}
      />
    </label>
    <button disabled={!selectedProject || startingAgentPlan || !agentGoalDraft.trim()} type="submit">
      {startingAgentPlan ? "Planning" : "Start Plan"}
    </button>
  </form>
</section>
```

Render status:

```tsx
<section className="panel-section">
  <h3>Runtime</h3>
  <dl className="compact-list">
    <div>
      <dt>Core</dt>
      <dd>{agentRuntimeStatus?.runtime_kind === "hermes_core" ? "Hermes Core" : "Unknown"}</dd>
    </div>
    <div>
      <dt>Version</dt>
      <dd>{agentRuntimeStatus?.hermes_version || "--"}</dd>
    </div>
  </dl>
</section>
```

Render latest run:

```tsx
{latestAgentRun ? (
  <section className="panel-section">
    <h3>Latest Run</h3>
    <p>{latestAgentRun.run.result_summary}</p>
    <ol className="activity-log">
      {latestAgentRun.events.map((event) => (
        <li key={event.id}>{event.message}</li>
      ))}
    </ol>
  </section>
) : null}
```

- [ ] **Step 7: Run frontend test and build**

Run:

```powershell
npm test -- src/App.test.tsx
npm run build
```

Expected:

- App tests pass.
- Build passes.

### Task 6: Smoke, Commit, Merge, Push

**Files:**

- Create: `docs/superpowers/reports/joi-0.13-agent-runtime-smoke-test.md`
- Commit all 0.13 implementation files.

- [ ] **Step 1: Run full verification**

Run:

```powershell
npm test
npm run build
cd src-tauri
cargo test
cargo test --test commands -- --nocapture
cargo test --test agent_runtime -- --nocapture
```

Expected:

- All commands pass.

- [ ] **Step 2: Browser smoke**

Run:

```powershell
npm run dev -- --host 127.0.0.1 --port 1420
```

Use the in-app browser to verify:

- Agent panel shows Runtime.
- Agent panel shows Hermes Core and version when running from the repo root.
- Agent goal form renders.
- In normal browser mode, backend invoke alert is visible instead of a crash.
- Desktop layout has no horizontal overflow.
- Mobile layout has no horizontal overflow.

- [ ] **Step 3: Write smoke report**

Create `docs/superpowers/reports/joi-0.13-agent-runtime-smoke-test.md` with:

- automated commands run
- browser observations
- Hermes bridge status
- Tauri browser limitation
- acceptance checklist

- [ ] **Step 4: Commit 0.13 implementation**

Run:

```powershell
git status --short
git add <0.13 files>
git commit -m "feat: add Joi 0.13 agent runtime"
```

- [ ] **Step 5: Merge to main**

Run from repository root:

```powershell
git checkout main
git merge --ff-only codex/joi-0.13-agent-runtime
```

Expected:

- Fast-forward merge.

- [ ] **Step 6: Verify on main**

Run:

```powershell
npm test
npm run build
cd src-tauri
cargo test
cargo test --test commands -- --nocapture
cargo test --test agent_runtime -- --nocapture
```

Expected:

- All commands pass.

- [ ] **Step 7: Push**

Run:

```powershell
git push origin main
```

Expected:

- GitHub `main` updates to the 0.13 commit.

## Acceptance Criteria

0.13 is complete only when:

- Joi has `agent_runs` and `agent_run_events` persisted in SQLite.
- Joi can report Hermes Core runtime status.
- Joi can build an Agent project context from brand, project, 0.12 understanding, creative direction, assets, memory, and versions.
- User can start an Agent plan from the Agent panel.
- Agent plan creates a saved `AgentRun`.
- Agent plan creates visible ordered events.
- Agent roles include planner, researcher, storyboard writer, prompt adapter, reviewer, and memory curator.
- Frontend shows runtime status, run summary, and event log.
- Rust tests cover schema, repository, runtime, and commands.
- Frontend tests cover Agent panel plan creation.
- Browser smoke report is written.
- Changes are merged to `main` and pushed to GitHub.

## Risks And Mitigations

### Risk: 0.13 Feels Like A Fake Agent

Mitigation:

- Be explicit in UI and docs that 0.13 is a local planner bridge.
- Persist real task runs and events.
- Keep the Hermes Core bridge and subprocess boundary ready for model-backed execution.

### Risk: Hermes Checkout Path Is Not Stable After Packaging

Mitigation:

- 0.13 resolves `.external/hermes-agent` from the repo root for development.
- Runtime status returns not-ready instead of crashing when packaged path is missing.
- Packaging-time runtime bundling remains out of scope.

### Risk: AgentPanel Becomes Too Busy

Mitigation:

- Keep 0.13 Agent panel compact: runtime, goal form, latest run, event log.
- Do not expose full plan editor until a later iteration.

### Risk: Future LLM Runtime Needs Different Plan Schema

Mitigation:

- Store plan as JSON.
- Keep role IDs stable.
- Keep command result typed around run and events, not around every future plan field.

## Handoff To 0.14

0.14 should use 0.13 `agent_runs` as the execution surface for research:

- User starts a research run from the Agent panel or Research tab.
- Researcher role uses web tools.
- Findings and sources are saved to `research_reports`.
- Agent events record source collection and report writing.

## Self-Review

- Spec coverage: This plan covers the 0.13 roadmap scope: runtime selection boundary, Joi-owned data layer, roles, tool bridge foundations, task run model, plan/execute/review/save records, and visible logs.
- Placeholder scan: No task contains TBD/TODO/fill-in instructions. Every new command, struct, table, and test has concrete names and expected behavior.
- Type consistency: `AgentRun`, `AgentRunEvent`, `AgentRuntimeStatus`, `AgentPlanInput`, `AgentPlanResult`, and command names are consistent across backend, frontend, and tests.
