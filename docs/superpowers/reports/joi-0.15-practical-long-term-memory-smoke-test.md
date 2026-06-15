# Joi Agent 0.15 Practical Long-Term Memory Smoke Test

Date: 2026-06-15
Branch: `codex/joi-0.15-practical-memory`

## Automated Verification

- `npm test`: passed, 1 test file, 10 tests.
- `npm run build`: passed, TypeScript and Vite production build completed.
- `cargo test`: passed all Rust unit and integration tests.

## Browser Smoke

Target: `http://127.0.0.1:1420/`

Browser: system Chrome via Playwright with a minimal Tauri `invoke` mock.

Desktop viewport `1440x900`:

- Project workspace rendered `Spring Drop Film`.
- Memory tab rendered manual memory controls.
- Memory tab rendered candidate generation controls.
- `Use research reports` checkbox rendered checked by default.
- Proposed / Accepted / Rejected sections rendered.
- Generated candidate rendered: `Use tactile close-ups as visual proof before the model movement.`
- Source trace rendered: `research_report:research-1`.
- Accept action updated the candidate status to `accepted`.
- Horizontal overflow check passed: document and body width matched viewport width.

Mobile viewport `390x844`:

- Memory tab rendered manual memory controls.
- Memory tab rendered candidate generation controls.
- Proposed / Accepted / Rejected sections rendered.
- Generated candidate and source trace rendered.
- Accept action updated the candidate status to `accepted`.
- Horizontal overflow check passed: document and body width matched viewport width.

## Acceptance Checklist

- Memory candidate command exists: `joi_generate_memory_candidates`.
- Memory status update command exists: `joi_update_memory_status`.
- Repository can create proposed memory with source trace and confidence.
- Repository can update memory to proposed, accepted, or rejected.
- Memory curation service can generate candidates from research reports.
- Memory curation service can generate candidates from feedback text.
- Duplicate deterministic conflicts are returned with `conflict_memory_ids`.
- Memory tab groups proposed, accepted, and rejected memory.
- Proposed memory can be accepted or rejected from the UI.
- Memory curation creates an Agent run with `local_memory_bridge`.
- Automated tests cover repository, service, commands, and UI flow.

## Known Limitations

0.15 implements deterministic duplicate conflict detection only. It does not perform semantic contradiction detection or external/cloud memory sync. Browser smoke used a Tauri invoke mock because a normal browser does not provide the Tauri runtime.
