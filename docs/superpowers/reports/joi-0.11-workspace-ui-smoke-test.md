# Joi Agent 0.11 Workspace UI Smoke Test

Date: 2026-06-15

## Scope

This smoke test covers the 0.11 workspace UI shell, frontend command wiring, and existing Rust command contracts.

## Automated Checks

- `npm test`
  - Result: passed
  - Coverage: workspace shell rendering, backend command wrapper calls, snapshot save action, project memory creation, and create-mode regression for new brands/projects.
- `npm run build`
  - Result: passed
  - Coverage: TypeScript project build and Vite production bundle.
- `cargo test`
  - Result: passed
  - Coverage: existing Rust unit and integration tests.
- `cargo test --test commands -- --nocapture`
  - Result: passed
  - Coverage: Tauri command DTO and helper tests.
- `git diff --check`
  - Result: passed
  - Coverage: whitespace errors for changed 0.11 frontend files.

## Browser Smoke

Target: `http://127.0.0.1:1420/`

Observed desktop layout:

- Top bar renders app identity, backend status area, selected context area, and snapshot/export actions.
- Left rail renders Brands, Projects, and Workflow navigation.
- Main workspace renders project setup, metrics, and workflow map.
- Right Agent panel renders current task, context, and activity log sections.
- Workflow map wraps into multiple rows at medium desktop width instead of compressing long step names.
- Text overflow scan returned no overflowing text elements.

Observed narrow layout:

- Top bar, left rail, and main workspace stack into a single column.
- Right Agent panel is hidden under the responsive breakpoint.
- Page width stays within the viewport; no horizontal overflow was detected.
- Text overflow scan returned no overflowing text elements.

## Known Runtime Limitation

The Vite browser smoke runs outside the Tauri desktop runtime, so `@tauri-apps/api` cannot reach the native `invoke` bridge in a normal browser tab. The page renders the workspace shell and displays the expected error toast for the missing invoke bridge. Live command behavior is covered by Vitest mocks and Rust Tauri command tests.

## Acceptance Notes

- The placeholder status card has been replaced by a three-column workspace UI.
- Brand/project create and update flows are represented in the UI and covered by frontend tests.
- Project memory creation and snapshot save actions are wired through the frontend API wrapper and covered by frontend tests.
- Assets, memory, and versions panels are present and data-backed through existing command wrappers.
- Browser verification found no layout-blocking issue at desktop or narrow widths.
