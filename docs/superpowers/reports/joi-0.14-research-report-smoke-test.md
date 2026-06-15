# Joi Agent 0.14 Research Report Smoke Test

Date: 2026-06-15
Branch: `codex/joi-0.14-research-report`

## Automated Verification

- `npm test`: passed, 1 test file, 9 tests.
- `npm run build`: passed, TypeScript and Vite production build completed.
- `cargo test`: passed all Rust unit and integration tests.

## Browser Smoke

Target: `http://127.0.0.1:1420/`

Browser: system Chrome via Playwright with a minimal Tauri `invoke` mock.

Desktop viewport `1440x900`:

- Project workspace rendered `Spring Drop Film`.
- Research tab rendered a real form, not the reserved empty state.
- Visible fields: Research goal, Market focus, Platform focus, Source title, Source URL, Source excerpt.
- Saved Reports panel rendered.
- Generating a mocked research report displayed `Texture proof point`.
- Horizontal overflow check passed: document and body width matched viewport width.

Mobile viewport `390x844`:

- Research tab rendered the same required fields.
- Saved Reports panel rendered.
- Generated finding rendered.
- Horizontal overflow check passed: document and body width matched viewport width.

## Acceptance Checklist

- Research command API exists: `joi_generate_research_report`.
- Research report listing API exists: `joi_list_research_reports`.
- `research_reports.findings_json` and `research_reports.sources_json` persist structured content.
- Research generation creates a completed Agent run with `local_research_bridge`.
- Research generation creates ordered researcher, reviewer, and planner events.
- Frontend Research workspace can submit source-assisted research input.
- Frontend displays latest findings and saved reports.
- Automated tests cover repository, service, commands, and UI flow.

## Known Limitation

0.14 is source-assisted research. It does not crawl or fetch arbitrary web pages inside the desktop app; users provide source title, URL, source type, and excerpt. The browser smoke used a Tauri invoke mock because a normal browser does not provide the Tauri runtime.
