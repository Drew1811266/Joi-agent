# Joi Agent 0.17 Multi-Model Prompt Adapters Smoke Test

Date: 2026-06-15

## Verification Commands

All commands were run from `D:\Software Project\Joi-agent\.worktrees\joi-0.17-prompt-adapters`.

- `npm test` passed: 1 test file, 12 tests.
- `npm run build` passed: TypeScript build and Vite production build completed.
- `cargo test` passed from `src-tauri`: unit, integration, and doc tests completed.
- Focused tests passed during implementation:
  - `cargo test --test prompt_adapter -- --nocapture`
  - `cargo test --test commands -- --nocapture`
  - `cargo test --test db_migration -- --nocapture`
  - `cargo test --test structured_content_repository -- --nocapture`
  - `cargo test --test project_snapshots -- --nocapture`

Known warning: Rust integration tests still report the existing `TestApp.temp_dir` dead-code warning.

## Browser Smoke

Local server:

- `npm run dev -- --host 127.0.0.1 --port 1420`
- URL checked: `http://127.0.0.1:1420/`

Smoke checks:

- App shell rendered in a normal browser.
- `Prompts` tab opened successfully.
- Prompts workspace displayed `Prompt Generator` and `Prompt Packages`.
- `Generate Video Prompts` and `Generate Image Prompts` controls were visible.
- Desktop viewport `1440x900` had no horizontal overflow.
- Mobile viewport `390x844` had no horizontal overflow.

Normal browser limitation:

- The normal browser does not provide native Tauri IPC, so it shows `Cannot read properties of undefined (reading 'invoke')`.
- Command interaction is covered by Rust command tests and React Vitest mocks instead of the browser smoke.

## Platform Coverage

Video adapters:

- `jimeng_video`
- `grok_video`

Image adapters:

- `banana_2_image`
- `jimeng_image`
- `gpt_image_2`

## Acceptance Checklist

- Shot-bound video prompt packages are supported.
- Project-bound image prompt packages are supported.
- Prompt packages store prompt text, negative prompt, adapter/source metadata, completeness metadata, and copy text.
- Prompt adapter generation creates Agent run/events.
- React flow covers video generation, image generation, prompt editing, saving, and copying.
- Prompts workspace renders without layout overflow at desktop and mobile smoke sizes.
- Changes are ready to merge to `main` after this report is committed.
