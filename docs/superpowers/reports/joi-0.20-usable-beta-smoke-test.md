# Joi 0.20 Usable Beta Smoke Test

Date: 2026-06-15

Branch: `codex/joi-0.20-usable-beta`

## Automated Verification

- `npm test`: PASS
- `npm run build`: PASS
- `cargo test`: PASS
- `cargo test --test beta_workflow beta_run_generates_end_to_end_project_outputs`: PASS
- `cargo test --test commands beta_workflow_commands_report_status_and_run`: PASS

Known warning:

- Rust tests still warn that `TestApp.temp_dir` is never read. This is existing test fixture ownership behavior and not a 0.20 regression.

## Browser Verification

Desktop:

- Opened `http://127.0.0.1:55306/`.
- Overview renders `Beta Workflow`.
- Beta workflow controls, including `Run Beta Workflow`, are visible.
- Browser preview shows the readable backend unavailable fallback instead of a raw `invoke` exception.
- No horizontal overflow at the default desktop viewport.
- Console has no error logs.

Mobile:

- Verified at `390 x 844`.
- `Beta Workflow` remains available in the page.
- Beta form fields stack to one column.
- No horizontal overflow.
- Console has no error logs.

Standalone browser note:

- The in-app browser does not provide Tauri IPC, so real project-backed beta status cards cannot be loaded in this preview surface. Status card rendering is covered by `src/App.test.tsx`, and native beta execution is covered by the Rust command tests listed above.

## Benchmark Coverage

- Brand: contemporary womenswear label.
- Product: spring outerwear collection.
- Goal: 15 second short-video launch ad.
- Reference source: source-backed benchmark note.
- Output: creative direction, storyboard, Jimeng and Grok video prompts, Banana 2, Jimeng Image, and GPT Image 2 image prompts, quality review, delivery report, package preview, accepted memory participation, and snapshot.

## Result

0.20 usable beta workflow passed automated verification, browser layout smoke, and native command smoke.
