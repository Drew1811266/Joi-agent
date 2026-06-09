# Joi Agent Design

Date: 2026-06-09

## Summary

Joi Agent is a desktop AI agent product for fashion advertising content workflows.
It is not a generic prompt skill layered on top of another agent. Its product
value is a structured workspace that turns research, product understanding,
creative direction, storyboard planning, platform prompt adaptation, version
history, and memory into one repeatable production system.

The first product direction is:

- Short fashion advertisement videos, usually 15-30 seconds.
- Fashion model-shot image prompt generation.
- Chinese-first interface and output.
- Multi-brand and multi-project local desktop use.
- Hermes Agent core forked and retained as the runtime foundation.

## Architecture

Joi Agent uses a three-layer architecture.

### Joi Desktop

Joi Desktop is a Tauri desktop application. It provides the independent product
experience: brand and project management, chat, assets, research, storyboard
editing, platform prompt packages, version history, import/export, and memory
review.

### Joi Backend / Domain Layer

The domain layer is the product core. It owns:

- Multi-brand and multi-project data models.
- Fashion advertising workflow state.
- Structured content schemas.
- Version history and rollback.
- Modification reason capture.
- Export generation.
- Memory routing across user, brand, and project contexts.
- Translation of user actions into runtime tasks.

This layer decides the workflow. The model does not directly mutate project
state; structured writes go through backend validation.

### Joi Runtime

Joi Runtime is forked from Hermes Core. The first phase keeps most Hermes Core
capabilities so Joi remains extensible and can later integrate additional open
source projects.

Joi Runtime retains:

- Multi-model provider routing.
- Tools and toolsets.
- Skills.
- MCP integration.
- Memory.
- Image input and vision routing.
- Image/video generation provider abstractions.
- Web, search, and browser capabilities.
- Terminal, file, and basic execution tools.
- The extension surface needed for future open source project integrations.

Joi adds domain constraints and orchestration on top of this runtime rather
than immediately minimizing or rewriting it.

## Product Workflow

The default interaction is automatic progression with interruptible checkpoints.
The agent can produce a full first draft, while users can pause and revise at
important nodes.

### Project Creation

The user selects or creates a brand, then creates a project with:

- Advertising goal.
- Video duration.
- Target platforms.
- Product information.
- Product images.
- Reference images.
- Reference videos or video links.
- Product, brand, or reference links.

### Research Stage

Research is part of the MVP and is optional per project.

Modes:

- Fast mode: skip research and generate from user input and assets.
- Research mode: parse user links and run active public web search.
- Deep mode: include broader competitor, trend, brand, and audience analysis.

The first version supports user-provided links plus agent-initiated search. It
does not depend on logged-in or platform-specific crawling for Xiaohongshu,
Douyin, Taobao, or similar platforms.

Outputs include:

- Source list.
- Competitor observations.
- Trend summary.
- Audience insights.
- Scene and styling suggestions.
- Creative recommendations.
- Risks and constraints.

### Product Understanding

The agent analyzes uploaded fashion product images and user text to extract:

- Category.
- Color.
- Silhouette.
- Material guesses.
- Visual details.
- Likely selling points.
- Suggested scenes, model poses, and visual styles.

The user can quickly correct key facts. Corrected facts override model guesses.

### Creative Direction

The system generates multiple directions, for example:

- Premium minimal.
- Urban commute.
- Sweet-cool street style.
- Vacation atmosphere.
- E-commerce clean hero style.

Users can select, mix, reject, or regenerate directions.

### Storyboard Generation

The system generates a 15-30 second storyboard. Each shot includes:

- Shot number.
- Duration.
- Visual description.
- Model action.
- Camera movement.
- Scene.
- Lighting.
- Subtitle or voiceover suggestion.
- Generation rationale.

Shots can be edited, locked, deleted, reordered, or regenerated independently.

### Platform Prompt Adaptation

The MVP generates prompt packages but does not directly call generation
platforms.

Video prompt targets:

- Jimeng.
- Grok.

Image prompt targets:

- Banana 2.
- Jimeng Image.
- GPT Image 2.

Prompts are organized by platform and by shot. Each shot can regenerate prompts
for one platform without overwriting locked content.

### Reference Video Analysis

The MVP supports reference video files and links.

For local video files, the system should extract representative frames and
summarize:

- Key visual frames.
- Model movement.
- Camera motion.
- Lighting and color style.
- Scene composition.
- Approximate pacing and cut rhythm.

For video links, the system attempts link analysis or download where feasible.
If that fails, it degrades to page metadata, user description, and linked
context instead of blocking the whole workflow.

### Versioning And Memory

Every significant generation or user edit creates a version.

The version system supports:

- History.
- Rollback.
- Final-version marking.
- Modification reason recording.
- Locked shots.
- Per-shot regeneration history.

Modification reasons are used to improve long-term behavior. Memory writes are
routed to:

- User preference memory.
- Brand memory.
- Project memory.

Memory writes are confirmed by the user or controlled by explicit rules.

### Export And Import

The MVP supports:

- Markdown export for human-readable project documents.
- Excel/CSV export for storyboard tables and prompt management.
- JSON project packages for import/export and future automation.

Word and PDF exports are outside the MVP.

## Data Model

The MVP stores structured content rather than only chat logs.

### Brand

Fields include:

- Name.
- Positioning.
- Audience.
- Tone.
- Forbidden terms.
- Common scenes.
- Model preferences.
- Platform preferences.
- Brand memory.

### Project

Fields include:

- Brand reference.
- Project title.
- Advertising goal.
- Duration.
- Target platforms.
- Current workflow stage.
- Current version.
- Final version.
- Created and updated timestamps.

### Asset

Assets include:

- Product images.
- Reference images.
- Reference videos.
- Local files.
- Web links.
- Research sources.

Each asset stores type, source, description, and whether it is active in the
current generation.

### ResearchReport

Contains sources, competitor observations, trend summary, audience insights,
creative suggestions, and risks.

### ProductUnderstanding

Contains category, color, silhouette, material guesses, visual details, selling
points, and user corrections.

### CreativeDirection

Contains theme, emotion, scene, visual style, pacing, recommended platforms,
and rationale.

### Storyboard And Shot

Storyboard is a collection of shots. Each shot stores duration, visual
description, model action, camera movement, scene, lighting, subtitle or
voiceover suggestion, rationale, lock state, and regeneration metadata.

### PromptPackage

Prompt packages are organized by platform and shot:

- Jimeng video prompts.
- Grok video prompts.
- Banana 2 image prompts.
- Jimeng Image prompts.
- GPT Image 2 prompts.

### Version

Versions track generated or edited states, rollback targets, final markings,
locked content, and user modification reasons.

### MemoryEntry

Memory entries are scoped as user, brand, or project memory. They are derived
from confirmed preferences, modification reasons, final versions, and repeated
corrections.

## UI Design

Joi Desktop uses a three-column workspace.

### Left Column

The left column contains:

- Brand switcher.
- Project list.
- Current project assets.
- Product images.
- Reference images and videos.
- Links and research sources.
- Version history entry points.

### Middle Column

The middle column contains:

- Chat interaction.
- Automatic workflow progress.
- Checkpoint cards for product understanding, creative directions, storyboard,
  and memory writes.
- User revisions such as "make shot 3 more premium" or "make the model action
  more natural".

### Right Column

The right column is the structured result workspace.

Tabs:

- Project document: brief, research report, creative direction, final plan.
- Storyboard table: editable shot-level table.
- Prompt package: platform and shot prompt views.
- Export: Markdown, Excel/CSV, and JSON.
- Memory review, which may be postponed if needed.

Key interactions:

- Edit any shot.
- Regenerate any shot.
- Regenerate one platform prompt for one shot.
- Lock shots.
- Record modification reasons.
- Copy platform prompts.
- Export and import project packages.

## Runtime And Tool Boundaries

Joi Runtime is based on Hermes Core, but Joi Domain Layer owns product rules.

Domain tools include:

- `fashion_ad_research`
- `product_understanding`
- `creative_direction_generator`
- `storyboard_generator`
- `prompt_adapter`
- `project_exporter`
- `memory_router`

These tools are domain-facing orchestration units. They may call Hermes tools,
model providers, web/search/browser capabilities, vision analysis, video frame
analysis, file tools, and memory tools internally.

Failure handling:

- Research failures should not block fast generation mode.
- Video link failures degrade to link metadata and user-provided context.
- Video frame extraction failures degrade to text/reference analysis.
- Platform prompt generation failures can retry at platform or shot granularity.
- Structured writes must validate against Joi schemas before persistence.

## Collaboration And Storage

The MVP is local-first and supports lightweight collaboration.

It does not include accounts, permissions, cloud sync, or multi-tenant
isolation.

It should support:

- Local workspace storage.
- Configurable workspace directory.
- Shared-directory usage for small teams.
- Project package export.
- Project package import.

## MVP Scope

Included:

- Tauri desktop app.
- Multi-brand and multi-project support.
- Local lightweight collaboration and project import/export.
- Product image upload and image understanding.
- Reference image, reference video, and link inputs.
- Video frame extraction and reference style summary.
- Active public web research and user link parsing.
- Automatic product understanding, creative direction, storyboard, and prompt
  package generation.
- Jimeng and Grok video prompts.
- Banana 2, Jimeng Image, and GPT Image 2 image prompts.
- Shot-level editing, locking, and regeneration.
- Version history, rollback, and final-version marking.
- Modification reason capture.
- User, brand, and project memory routing.
- Markdown, Excel/CSV, and JSON export.
- Hermes multi-model providers and tools/skills/MCP mechanisms.

Excluded:

- Direct calls to Jimeng, Grok, Banana 2, Jimeng Image, or GPT Image 2 for
  generation.
- Accounts, permissions, multi-tenancy, and cloud sync.
- Word and PDF export.
- Logged-in platform crawling or dedicated scrapers for social/e-commerce
  platforms.
- A new Joi plugin system separate from Hermes.
- Large-scale Hermes Core minimization in the first phase.

## Development Roadmap

### Phase 0: Hermes Fork Validation

Verify Hermes Core can be forked, installed, and run locally. Identify runtime
entry points, model provider boundaries, tool registration, memory access, and
dependencies relevant to Joi Runtime.

### Phase 1: Joi Data Model And Local Project Store

Implement local schemas for Brand, Project, Asset, ResearchReport,
ProductUnderstanding, CreativeDirection, Storyboard, Shot, PromptPackage,
Version, and MemoryEntry.

### Phase 2: Tauri Desktop Workspace

Build the three-column Joi Desktop shell with brand/project navigation, asset
management, chat, structured result tabs, and export/import surfaces.

### Phase 3: Joi Domain Workflow

Implement the workflow state machine, automatic progression, checkpoint pauses,
user corrections, shot-level regeneration, lock behavior, version history, and
modification reason capture.

### Phase 4: Hermes Core Integration

Integrate model providers, vision, web/search/browser capabilities, memory,
MCP, and custom domain tools with the Joi Domain Layer.

### Phase 5: Research, Video Analysis, And Prompt Adapters

Implement research reports, reference video extraction, reference style
summaries, and prompt adapters for Jimeng, Grok, Banana 2, Jimeng Image, and
GPT Image 2.

### Phase 6: Export, Import, Memory Optimization, And Packaging

Implement Markdown, Excel/CSV, and JSON export/import; finish rollback and
final-version support; refine memory routing; package the Tauri desktop app.

## Open Risks

- Hermes Core is large. Keeping most of it preserves extensibility but requires
  clear module boundaries and dependency management.
- The first clone attempt timed out due repository size. Phase 0 should use a
  robust source acquisition strategy.
- Direct platform APIs are intentionally out of scope; prompt quality must be
  validated manually in target tools.
- Public web research quality depends on available search and browser tools.
- Reference video link processing may fail on protected or unsupported sites.

## Approval

This design reflects the decisions confirmed in conversation:

- Use Hermes Agent as the source for Joi Runtime, not as a mere Skill host.
- Build a Tauri desktop app.
- Fork Hermes Core and retain most capabilities initially.
- Use Hermes tools/skills/MCP/provider extension mechanisms.
- Build a product-layer-first fashion advertising workspace.
- Support multi-brand, multi-project, research, video reference analysis,
  structured storyboards, platform prompt packages, version history, and memory.
