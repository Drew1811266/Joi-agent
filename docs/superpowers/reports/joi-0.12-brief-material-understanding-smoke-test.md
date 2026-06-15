# Joi Agent 0.12 Brief And Material Understanding Smoke Test

Date: 2026-06-15

## Automated Verification

- `npm test -- src/App.test.tsx`: passed, 7 tests.
- `npm run build`: passed.
- `npm test`: passed, 7 tests.
- `cargo test`: passed.

Rust emitted the existing `TestApp.temp_dir` dead-code warning in integration tests. No test failed.

## Browser Smoke

Target: `http://127.0.0.1:1420/` through Vite.

Desktop viewport:

- Workspace shell rendered with top bar, left rail, main workspace, and agent panel.
- The normal browser showed the expected captured Tauri invoke alert because it is not running inside the Tauri runtime.
- Brief tab rendered the 0.12 form:
  - Project brief
  - Product name
  - Product category
  - Audience
  - Target platforms
  - Selling points
  - Constraints
  - Visual direction
  - Generate Understanding
- Reference Materials rendered with reference name, URL, kind, and Add Reference.
- Reference kind default value was rechecked as `link`, matching the backend `AssetKind::Link` value.
- Structured Context rendered with the empty-state panel.
- Layout metrics showed no horizontal overflow:
  - body `scrollWidth`: 1265
  - viewport width: 1280
  - main horizontal overflow: false
  - Brief layout horizontal overflow: false

Mobile viewport, 390 x 844:

- Brief tab rendered after viewport switch and reload.
- Form labels remained visible.
- No label overflow was detected.
- Layout metrics showed no horizontal overflow:
  - body `scrollWidth`: 375
  - viewport width: 390
  - main horizontal overflow: false
  - Brief layout horizontal overflow: false

Browser console:

- No console error logs were recorded during smoke testing.

## Tauri Runtime Limitation

The Vite smoke target is a browser-only runtime, so `@tauri-apps/api/core` cannot reach `window.__TAURI_INTERNALS__`. The visible invoke alert is expected in this mode. Data-backed command behavior is covered by Vitest mocks and Rust command tests.

## Acceptance Checklist

- [x] Brief tab is data-backed in React and no longer uses the 0.11 empty-state placeholder.
- [x] User can enter brief, product, category, audience, platforms, selling points, visual direction, and constraints.
- [x] User can add link/reference material records through the frontend command wrapper.
- [x] User can generate structured understanding from the Brief tab.
- [x] Generated result displays brief summary, brand summary, visual direction, selling points, constraints, and missing questions.
- [x] Generated understanding is saved through the local Rust repository.
- [x] Creative direction is saved when visual direction is provided.
- [x] Existing snapshot tests cover inclusion of product understandings and creative directions.
- [x] Frontend tests cover brief generation and reference-material submit gating.
- [x] Rust tests cover command and persistence behavior.
- [x] Browser smoke report is written.
