# Joi 0.19 Quality Review Iteration Smoke Test

Date: 2026-06-15

Branch: `codex/joi-0.19-quality-review`

## Scope

0.19 adds quality review and iteration support:

- `quality_reviews` persistence with checks and suggestions.
- Deterministic quality review generation from saved brand, project, understanding, storyboard, and prompt context.
- Suggestion application for supported storyboard shot and prompt package fields.
- Agent run/event records for review and iteration actions.
- Tauri commands for generate, list, and apply review suggestions.
- Snapshot inclusion for quality review records.
- Frontend Review workspace with checklist and revision suggestion controls.
- Browser preview fallback message when the Tauri desktop backend is unavailable.

## Automated Verification

Passed:

- `npm test` passed: 1 test file, 15 tests.
- `npm run build` passed: TypeScript build and Vite production build completed.
- `cargo test` passed from `src-tauri`.

Known warning:

- Rust tests still warn that `TestApp.temp_dir` is never read. This is existing test fixture ownership behavior and not a 0.19 regression.

## Browser Smoke

Local dev server:

- URL: `http://127.0.0.1:55306/`
- Command: `npm run dev -- --host 127.0.0.1 --port 55306`

Desktop check:

- Opened app in the Codex in-app browser.
- Clicked the `Review` tab between `Prompts` and `Delivery`.
- Verified `Quality Review`, `Review Checklist`, and `Revision Suggestions` are visible.
- Verified the browser preview shows a readable backend fallback message instead of the raw `invoke` exception.
- Horizontal overflow check: `scrollWidth = 1265`, `clientWidth = 1265`.
- Browser error log query returned no entries.

Mobile check:

- Viewport: `390 x 844`.
- Reloaded app and clicked the `Review` tab.
- Verified `Quality Review`, `Review Checklist`, and `Revision Suggestions` remain visible.
- Verified no element extends beyond the viewport bounds.
- Horizontal overflow check: `scrollWidth = 375`, `clientWidth = 375`.
- Browser error log query returned no entries.

## Acceptance Checklist

Passed:

- Quality reviews are persisted and listable by project.
- Quality review records store typed checks and revision suggestions.
- Review generation detects missing or weak project, storyboard, prompt, and brand consistency signals.
- Review generation produces score, summary, checks, suggestions, and Agent run/event records.
- Suggestions can be applied to supported shot descriptions and prompt package prompt text.
- Locked shots and prompt packages reject suggestion application.
- Unsupported suggestion targets are rejected.
- Quality reviews are included in project snapshots.
- Tauri command helpers cover generate, list, and apply suggestion flows.
- Frontend Review tab covers generate, checklist display, suggestions display, and apply suggestion flow through invoke mocks.
- Review workspace renders without layout overflow at desktop and mobile smoke sizes.
- Browser preview degrades with a clear desktop backend unavailable message.

## Known Limitations

- Browser smoke runs in a normal browser, so it verifies layout and shell behavior only. Native Tauri command execution is covered by Rust command tests and React invoke mocks.
- In a normal browser without a selected project, `Generate Review` remains disabled. End-to-end generation and suggestion application are covered by automated command and frontend tests.
- Review suggestions currently apply to selected high-value fields only: storyboard shot description and prompt package prompt text. Broader editable targets remain future scope.

## Result

0.19 quality review and iteration workflow passed automated and browser smoke verification.
