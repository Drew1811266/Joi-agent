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

## Result

0.18 delivery report and package handoff workflow passed automated and browser smoke verification.
