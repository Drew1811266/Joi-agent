# Joi Agent 0.13 Agent Runtime Smoke Test

Date: 2026-06-15

## Scope

0.13 adds Joi's first executable Agent runtime layer:

- Joi-owned `agent_runs` and `agent_run_events` persistence.
- Hermes Core status bridge for the local `.external/hermes-agent` checkout.
- Local planner bridge that reads saved project context and creates a persisted run.
- Agent panel controls for runtime status, goal entry, run history, latest run, and events.

## Automated Verification

Commands run from `D:\Software Project\Joi-agent\.worktrees\joi-0.13-agent-runtime`:

```powershell
npm test
npm run build
```

Results:

- `npm test`: passed, 1 test file, 8 tests.
- `npm run build`: passed, TypeScript build and Vite production build completed.

Commands run from `D:\Software Project\Joi-agent\.worktrees\joi-0.13-agent-runtime\src-tauri`:

```powershell
cargo test
cargo test --test commands -- --nocapture
cargo test --test agent_runtime -- --nocapture
```

Results:

- `cargo test`: passed all unit and integration tests.
- `cargo test --test commands -- --nocapture`: passed, 8 tests.
- `cargo test --test agent_runtime -- --nocapture`: passed, 4 tests.

Existing warning:

- `tests/common/mod.rs` has an existing `TestApp.temp_dir` dead-code warning. This is expected because the field keeps the temporary directory alive for test lifetime.

## Browser Smoke Test

Vite dev server:

```powershell
npm run dev -- --host 127.0.0.1 --port 1420
```

Checked URL:

- `http://127.0.0.1:1420/`

Desktop viewport result:

- Page title: `Joi Agent`.
- Agent panel rendered.
- Runtime heading rendered.
- Agent goal textarea rendered.
- Start Plan button rendered.
- No horizontal overflow.

Mobile viewport result:

- Viewport width: 375 px.
- No horizontal overflow.
- Agent panel is hidden by the existing responsive breakpoint, as expected.

Note:

- In a normal browser, the Tauri IPC bridge is unavailable, so the page shows the expected `Cannot read properties of undefined (reading 'invoke')` alert. Rust command tests and React API tests cover the Tauri command path.

## Acceptance Review

- Agent run schema exists with project foreign key and event ordering.
- Repository can create, get, and list agent runs and ordered events.
- Hermes bridge reports ready and missing-checkout states.
- Local planner bridge writes a completed run with six roles and seven events.
- Command helpers expose runtime status, start plan, get run, and list runs.
- Frontend calls runtime status and run list APIs.
- Agent panel can submit a goal and render the returned run events.
- Verification passed.
