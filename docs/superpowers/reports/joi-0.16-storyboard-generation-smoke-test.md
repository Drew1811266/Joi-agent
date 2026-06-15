# Joi Agent 0.16 Storyboard Generation Smoke Test

## Scope

0.16 adds a storyboard generation workspace for short fashion advertising videos. The smoke test verifies that the UI can generate a storyboard, edit an individual shot, regenerate a shot, and remain usable across desktop and mobile widths.

## Verification Commands

- `npm test`
- `npm run build`
- `cargo test`
- `cargo test --test storyboard_generation -- --nocapture`
- `cargo test --test commands -- --nocapture`

All verification commands passed during the 0.16 implementation pass.

## Browser Smoke

The local Vite app was served at `http://127.0.0.1:1420/`.

The in-app browser loaded the app, but the standard browser window could not mutate `window.__TAURI_INTERNALS__` for Tauri invoke mocking because the window object was not extensible. The smoke test therefore used headless Chrome DevTools Protocol with a pre-load Tauri mock injected before the app bundle executed.

Smoke assertions:

- `joi_generate_storyboard` was invoked from the Storyboard tab.
- Generated storyboard title and `2 shots · 15s` summary were visible.
- Generated shot content showed the garment focus `water-resistant cotton trench silhouette`.
- `joi_update_shot` was invoked from the shot edit form.
- Edited shot text `Edited opening product entrance.` was visible after saving.
- `joi_regenerate_shot` was invoked from the second shot.
- Regenerated shot text `Regenerated macro fabric insert.` was visible.
- Desktop viewport had no horizontal overflow.
- Mobile viewport had no horizontal overflow.

Smoke result:

```json
{
  "titleVisible": true,
  "shotCountVisible": true,
  "generatedVisible": true,
  "editedVisible": true,
  "regeneratedVisible": true,
  "desktop": {
    "clientWidth": 1425,
    "scrollWidth": 1425,
    "noHorizontalOverflow": true
  },
  "mobile": {
    "clientWidth": 390,
    "scrollWidth": 390,
    "noHorizontalOverflow": true
  },
  "invoked": [
    "joi_generate_storyboard",
    "joi_update_shot",
    "joi_regenerate_shot"
  ]
}
```

## Acceptance Review

- Backend stores typed storyboards and shot-level metadata.
- Storyboard generation service creates deterministic short-video shot plans from project context, accepted memories, research reports, and existing brief understanding.
- Tauri commands expose generation, listing, shot update, and shot regeneration.
- Frontend Storyboard workspace supports generate, inspect, edit, and regenerate workflows.
- Automated unit and integration tests cover the repository, service, commands, and UI paths.
- Browser smoke covers the critical 0.16 workflow and responsive overflow checks.

## Limitation

This browser smoke validates the web frontend with a mocked Tauri invoke layer. Native Tauri command execution is covered by Rust command tests.
