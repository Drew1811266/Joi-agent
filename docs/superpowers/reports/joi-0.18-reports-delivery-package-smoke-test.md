# Joi 0.18 Reports And Delivery Package Smoke Test

Date: 2026-06-15

Branch: `codex/joi-0.18-delivery-package`

## Scope

0.18 adds delivery reports and delivery package handoff support:

- `delivery_reports` persistence and repository CRUD.
- Delivery report Markdown generation from saved project context.
- Delivery package preview and optional Markdown export.
- Tauri commands for generate, list, update, preview, and export.
- Frontend Delivery workspace covering generate, edit, preview, and export.

## Automated Verification

Passed:

- `npm test`
- `npm run build`
- `cargo test`

Known warning:

- Rust tests still warn that `TestApp.temp_dir` is never read. This is existing test fixture ownership behavior and not a 0.18 regression.

## Browser Smoke

Local dev server:

- URL: `http://127.0.0.1:1420/`
- Command: `npm run dev -- --host 127.0.0.1`

Desktop check:

- Opened app in the Codex in-app browser.
- Clicked `Delivery` tab.
- Verified `Delivery Package`, `Report direction`, and report editor/empty-state text are visible.
- Horizontal overflow check: `scrollWidth = 1265`, `clientWidth = 1265`.

Mobile check:

- Viewport: `390 x 844`.
- Reloaded app and clicked `Delivery` tab.
- Verified `Delivery Package`, `Report direction`, and `Export directory` are visible.
- Horizontal overflow check: `scrollWidth = 375`, `clientWidth = 375`.

Console check:

- Browser warning/error log query returned no entries during smoke.

## Acceptance Checklist

Passed:

- Delivery reports are persisted in `delivery_reports`.
- Delivery reports are included in project snapshots.
- Joi can generate a deterministic Markdown delivery report from saved project context.
- Report Markdown includes project brief, brand, product, research, creative direction, storyboard, prompt packages, assets, version notes, and export notes.
- Missing source sections are represented as explicit warnings.
- Report title, Markdown, and final candidate status can be edited and saved.
- Delivery package preview returns JSON file, assets folder, Markdown report file, counts, and warnings.
- Project export can optionally include selected delivery report Markdown.
- Export without a report remains supported.
- Delivery report generation creates Agent run/events.
- Frontend Delivery tab covers generate, edit, preview, and export flow.
- Tests cover schema migration, repository, generator, commands, export integration, and frontend flow.

## Known Limitations

- Browser smoke runs in a normal browser, so it verifies layout and shell behavior only. Native Tauri command execution is covered by Rust command tests and React invoke mocks.
- The Delivery workspace uses a plain Markdown textarea in 0.18. Rich text editing remains out of scope.
- Export directory selection is a typed path field in 0.18. A native directory picker can be added in a later stage.

## Result

0.18 delivery report and package handoff workflow passed automated and browser smoke verification.
