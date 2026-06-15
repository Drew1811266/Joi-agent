import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";

import App from "./App";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (command: string, payload?: unknown) => invokeMock(command, payload),
}));

const mockStoryboardGenerationResult = {
  storyboard: {
    id: "storyboard-1",
    project_id: "project-1",
    title: "Spring Drop Film storyboard",
    duration_seconds: 15,
    created_at: "2026-06-15T00:00:00Z",
    updated_at: "2026-06-15T00:00:00Z",
  },
  shots: [
    {
      shot: {
        id: "shot-1",
        storyboard_id: "storyboard-1",
        shot_number: 1,
        duration_seconds: 3,
        description: "Model enters a clean studio frame wearing the trench.",
        model_action: "Model walks forward.",
        camera_movement: "slow push-in",
        scene: "minimal warm studio",
        lighting: "soft side light",
        subtitle_or_voiceover: "Light enough for changing weather",
        rationale: "Opening shot establishes product and brand mood.",
        is_locked: false,
        metadata_json: {
          format_version: "joi.shot_metadata.v1",
          garment_focus: "water-resistant cotton trench silhouette",
          transition: "cut on movement",
        },
        created_at: "2026-06-15T00:00:00Z",
        updated_at: "2026-06-15T00:00:00Z",
      },
      visual_description: "Model enters a clean studio frame wearing the trench.",
      garment_focus: "water-resistant cotton trench silhouette",
      transition: "cut on movement",
    },
    {
      shot: {
        id: "shot-2",
        storyboard_id: "storyboard-1",
        shot_number: 2,
        duration_seconds: 3,
        description: "Close fabric texture detail fills the frame.",
        model_action: "Model lifts sleeve edge.",
        camera_movement: "macro slide",
        scene: "studio insert",
        lighting: "grazing highlight",
        subtitle_or_voiceover: "Texture that moves",
        rationale: "Product proof shot.",
        is_locked: false,
        metadata_json: {
          format_version: "joi.shot_metadata.v1",
          garment_focus: "fabric texture",
          transition: "match cut",
        },
        created_at: "2026-06-15T00:00:00Z",
        updated_at: "2026-06-15T00:00:00Z",
      },
      visual_description: "Close fabric texture detail fills the frame.",
      garment_focus: "fabric texture",
      transition: "match cut",
    },
  ],
  total_duration_seconds: 15,
  agent_run: {
    id: "run-storyboard",
    project_id: "project-1",
    user_goal: "Make the opening tactile and premium.",
    status: "completed",
    runtime_kind: "hermes_core",
    runtime_mode: "local_storyboard_bridge",
    runtime_version: "0.16.0",
    roles_json: ["planner", "storyboard_writer", "reviewer"],
    plan_json: [],
    result_summary: "Generated 2 shot storyboard for Spring Drop Film.",
    created_at: "2026-06-15T00:00:00Z",
    updated_at: "2026-06-15T00:00:00Z",
  },
  agent_events: [],
};

const mockPromptProfiles = [
  {
    id: "jimeng_video",
    display_name: "Jimeng Video",
    modality: "video",
    default_negative_prompt: "avoid distorted garments",
    required_fields: ["shot", "garment", "camera"],
  },
  {
    id: "grok_video",
    display_name: "Grok Video",
    modality: "video",
    default_negative_prompt: "avoid distorted garments",
    required_fields: ["shot", "garment", "camera"],
  },
  {
    id: "banana_2_image",
    display_name: "Banana 2 Image",
    modality: "image",
    default_negative_prompt: "avoid distorted garments",
    required_fields: ["image_brief", "garment", "model"],
  },
  {
    id: "jimeng_image",
    display_name: "Jimeng Image",
    modality: "image",
    default_negative_prompt: "avoid distorted garments",
    required_fields: ["image_brief", "garment", "model"],
  },
  {
    id: "gpt_image_2",
    display_name: "GPT Image 2",
    modality: "image",
    default_negative_prompt: "avoid distorted garments",
    required_fields: ["image_brief", "garment", "model"],
  },
];

function mockPromptGenerationResult(targetPlatforms: string[]) {
  const packages = targetPlatforms.map((platform, index) => {
    const profile = mockPromptProfiles.find((item) => item.id === platform) ?? mockPromptProfiles[0];
    const modality = profile.modality;
    const promptTitle =
      platform === "gpt_image_2"
        ? "GPT Image 2 prompt"
        : `${profile.display_name.replace(/\s+/g, " ")} prompt`;
    const promptText = `${promptTitle}\nCreate a refined fashion advertising ${modality} prompt.`;
    return {
      package: {
        id: `prompt-${platform}`,
        project_id: "project-1",
        shot_id: modality === "video" ? "shot-1" : null,
        platform,
        modality,
        prompt_text: promptText,
        negative_prompt: "avoid distorted garments, unreadable details",
        parameters_json: {
          format_version: "joi.prompt_package_parameters.v1",
          adapter_profile_id: platform,
        },
        is_locked: false,
        created_at: "2026-06-15T00:00:00Z",
        updated_at: "2026-06-15T00:00:00Z",
      },
      adapter_display_name: profile.display_name,
      shot_label: modality === "video" ? "Shot 1" : null,
      copy_text: promptText,
      missing_fields: [],
      completeness: [],
      sort_order: index + 1,
    };
  });
  return {
    packages,
    agent_run: {
      id: "run-prompts",
      project_id: "project-1",
      user_goal: "Generate model-specific prompt packages",
      status: "completed",
      runtime_kind: "hermes_core",
      runtime_mode: "local_prompt_adapter_bridge",
      runtime_version: "0.17.0",
      roles_json: ["prompt_adapter", "reviewer"],
      plan_json: [],
      result_summary: `Generated ${packages.length} prompt package(s).`,
      created_at: "2026-06-15T00:00:00Z",
      updated_at: "2026-06-15T00:00:00Z",
    },
    agent_events: [],
  };
}

function mockDeliveryReportResult(markdown = "# Spring Drop Film Delivery Report") {
  const report = {
    id: "delivery-report-1",
    project_id: "project-1",
    title: "Spring Drop Film Delivery Report",
    markdown,
    sections_json: {
      format_version: "joi.delivery_report_sections.v1",
      sections: [
        {
          id: "prompt_packages",
          title: "Prompt Packages",
          status: "complete",
          source_count: 2,
          warning: "",
        },
      ],
    },
    is_final_candidate: false,
    created_at: "2026-06-15T00:00:00Z",
    updated_at: "2026-06-15T00:00:00Z",
  };
  return {
    report,
    sections: report.sections_json.sections,
    package_preview: {
      project_json_file_name: "spring-drop-film.joi-project.json",
      assets_folder_name: "spring-drop-film-assets",
      delivery_report_file_name: "spring-drop-film-delivery-report.md",
      included_assets_count: 0,
      included_prompt_packages_count: 2,
      included_storyboards_count: 1,
      warnings: [],
    },
    agent_run: {
      id: "run-delivery",
      project_id: "project-1",
      user_goal: "Generate a delivery report for Spring Drop Film.",
      status: "completed",
      runtime_kind: "hermes_core",
      runtime_mode: "local_delivery_report_bridge",
      runtime_version: "0.18.0",
      roles_json: ["planner", "reviewer", "memory_curator"],
      plan_json: [],
      result_summary: "Generated delivery report.",
      created_at: "2026-06-15T00:00:00Z",
      updated_at: "2026-06-15T00:00:00Z",
    },
    agent_events: [],
  };
}

describe("Joi workspace shell", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockImplementation((command: string, payload?: unknown) => {
      const args = payload as { input?: Record<string, unknown> } | undefined;
      switch (command) {
        case "joi_health_check":
          return Promise.resolve({
            status: "ready",
            app_name: "Joi Agent",
            phase: "local workspace",
          });
        case "joi_list_brands":
          return Promise.resolve([
            {
              id: "brand-1",
              name: "Atelier Joi",
              description: "Editorial womenswear",
              style_keywords: [],
              visual_preferences: [],
              negative_preferences: [],
              common_scenes: [],
              model_preferences: [],
              platform_preferences: [],
              created_at: "2026-06-15T00:00:00Z",
              updated_at: "2026-06-15T00:00:00Z",
            },
          ]);
        case "joi_list_projects":
          return Promise.resolve([
            {
              id: "project-1",
              brand_id: "brand-1",
              title: "Spring Drop Film",
              advertising_goal: "Launch awareness",
              duration_seconds: 15,
              target_platforms: [],
              content_type: "fashion_ad",
              status: "draft",
              current_version_id: null,
              final_version_id: null,
              created_at: "2026-06-15T00:00:00Z",
              updated_at: "2026-06-15T00:00:00Z",
            },
          ]);
        case "joi_list_assets":
        case "joi_list_project_versions":
        case "joi_list_memory_entries":
        case "joi_list_product_understandings":
        case "joi_list_creative_directions":
        case "joi_list_research_reports":
        case "joi_list_delivery_reports":
          return Promise.resolve([]);
        case "joi_list_storyboards":
          return Promise.resolve([
            {
              storyboard: mockStoryboardGenerationResult.storyboard,
              shots: mockStoryboardGenerationResult.shots.map((item) => item.shot),
            },
          ]);
        case "joi_get_prompt_adapter_profiles":
          return Promise.resolve(mockPromptProfiles);
        case "joi_list_prompt_packages":
          return Promise.resolve([]);
        case "joi_get_agent_runtime_status":
          return Promise.resolve({
            runtime_kind: "hermes_core",
            runtime_mode: "local_planner_bridge",
            hermes_checkout_path: "D:/Software Project/Joi-agent/.external/hermes-agent",
            hermes_present: true,
            hermes_version: "0.16.0",
            phase0_report_present: true,
            ready: true,
            message: "Hermes Core bridge is ready for local planner mode.",
          });
        case "joi_list_agent_runs":
          return Promise.resolve([]);
        case "joi_start_agent_plan":
          return Promise.resolve({
            run: {
              id: "run-1",
              project_id: "project-1",
              user_goal: "Plan the next content workflow steps",
              status: "completed",
              runtime_kind: "hermes_core",
              runtime_mode: "local_planner_bridge",
              runtime_version: "0.16.0",
              roles_json: [
                "planner",
                "researcher",
                "storyboard_writer",
                "prompt_adapter",
                "reviewer",
                "memory_curator",
              ],
              plan_json: [{ role: "planner", title: "Confirm brief and material context" }],
              result_summary: "Created a local planner bridge run for Spring Drop Film.",
              created_at: "2026-06-15T00:00:00Z",
              updated_at: "2026-06-15T00:00:00Z",
            },
            events: [
              {
                id: "event-1",
                agent_run_id: "run-1",
                sequence_number: 1,
                role: "planner",
                event_type: "context_read",
                message: "Read saved context for Spring Drop Film.",
                payload_json: { project_title: "Spring Drop Film" },
                created_at: "2026-06-15T00:00:00Z",
              },
            ],
          });
        case "joi_save_project_snapshot":
          return Promise.resolve({
            id: "version-1",
            project_id: "project-1",
            version_number: 1,
            label: "Workspace snapshot",
            change_reason: "Saved from 0.11 workspace UI",
            changed_entities: [],
            snapshot_json: {},
            created_by: "user",
            is_final_candidate: false,
            created_at: "2026-06-15T00:00:00Z",
          });
        case "joi_create_memory_entry":
          return Promise.resolve({
            id: "memory-1",
            scope: "project",
            brand_id: "brand-1",
            project_id: "project-1",
            content: "Keep fabric texture visible",
            source: "user note",
            source_entity_type: "",
            source_entity_id: "",
            confidence: 0,
            status: "proposed",
            created_at: "2026-06-15T00:00:00Z",
            updated_at: "2026-06-15T00:00:00Z",
          });
        case "joi_generate_memory_candidates":
          return Promise.resolve({
            candidates: [
              {
                entry: {
                  id: "memory-candidate-1",
                  scope: "project",
                  brand_id: "brand-1",
                  project_id: "project-1",
                  content: "Use tactile close-ups as visual proof before the model movement.",
                  source: "research report",
                  source_entity_type: "research_report",
                  source_entity_id: "research-1",
                  confidence: 0.72,
                  status: "proposed",
                  created_at: "2026-06-15T00:00:00Z",
                  updated_at: "2026-06-15T00:00:00Z",
                },
                reason: "Source-backed research implication can guide future generation.",
                has_conflict: false,
                conflict_memory_ids: [],
              },
            ],
            agent_run: {
              id: "run-memory-1",
              project_id: "project-1",
              user_goal: "Curate practical long-term memory candidates",
              status: "completed",
              runtime_kind: "hermes_core",
              runtime_mode: "local_memory_bridge",
              runtime_version: "0.16.0",
              roles_json: ["memory_curator", "reviewer"],
              plan_json: [],
              result_summary: "Created 1 proposed memory candidate for Spring Drop Film.",
              created_at: "2026-06-15T00:00:00Z",
              updated_at: "2026-06-15T00:00:00Z",
            },
            agent_events: [],
          });
        case "joi_update_memory_status":
          return Promise.resolve({
            id: "memory-candidate-1",
            scope: "project",
            brand_id: "brand-1",
            project_id: "project-1",
            content: "Use tactile close-ups as visual proof before the model movement.",
            source: "research report",
            source_entity_type: "research_report",
            source_entity_id: "research-1",
            confidence: 0.72,
            status: "accepted",
            created_at: "2026-06-15T00:00:00Z",
            updated_at: "2026-06-15T00:00:00Z",
          });
        case "joi_create_brand":
          return Promise.resolve({
            id: "brand-2",
            name: "New Atelier",
            description: "Studio denim",
            style_keywords: [],
            visual_preferences: [],
            negative_preferences: [],
            common_scenes: [],
            model_preferences: [],
            platform_preferences: [],
            created_at: "2026-06-15T00:00:00Z",
            updated_at: "2026-06-15T00:00:00Z",
          });
        case "joi_create_project":
          return Promise.resolve({
            id: "project-2",
            brand_id: "brand-1",
            title: "Lookbook Motion",
            advertising_goal: "Convert collection interest",
            duration_seconds: 30,
            target_platforms: [],
            content_type: "fashion_ad",
            status: "draft",
            current_version_id: null,
            final_version_id: null,
            created_at: "2026-06-15T00:00:00Z",
            updated_at: "2026-06-15T00:00:00Z",
          });
        case "joi_generate_brief_understanding":
          return Promise.resolve({
            product_understanding: {
              id: "understanding-1",
              project_id: "project-1",
              product_name: "Lightweight trench",
              category: "outerwear",
              audience: "urban commuters",
              selling_points_json: ["water-resistant cotton", "soft structure"],
              constraints_json: ["avoid heavy winter styling"],
              notes: JSON.stringify({
                format_version: "joi.product_understanding_notes.v1",
                missing_questions: ["Which reference materials should Joi use as visual anchors?"],
              }),
              created_at: "2026-06-15T00:00:00Z",
              updated_at: "2026-06-15T00:00:00Z",
            },
            creative_direction: {
              id: "direction-1",
              project_id: "project-1",
              title: "Initial visual direction",
              concept: "clean studio walk with close fabric texture",
              tone: "user-defined",
              visual_style: "clean studio walk with close fabric texture",
              scene_direction: "",
              rationale: "Generated from 0.12 brief and material understanding input.",
              created_at: "2026-06-15T00:00:00Z",
              updated_at: "2026-06-15T00:00:00Z",
            },
            brief_summary: "15 second outerwear launch ad",
            brand_summary: "Atelier Joi: Editorial womenswear",
            visual_direction: "clean studio walk with close fabric texture",
            selling_points: ["water-resistant cotton", "soft structure"],
            constraints: ["avoid heavy winter styling"],
            missing_questions: ["Which reference materials should Joi use as visual anchors?"],
          });
        case "joi_create_reference_asset":
          return Promise.resolve({
            id: "asset-1",
            project_id: "project-1",
            kind: "link",
            display_name: "Studio trench reference",
            relative_path: "",
            source_uri: "https://example.com/trench-look",
            mime_type: "text/uri-list",
            file_size_bytes: 0,
            sha256: "",
            metadata_json: {},
            created_at: "2026-06-15T00:00:00Z",
            updated_at: "2026-06-15T00:00:00Z",
          });
        case "joi_generate_research_report":
          return Promise.resolve({
            report: {
              id: "research-1",
              project_id: "project-1",
              summary: "Research for Spring Drop Film: 1 source-backed finding.",
              findings_json: [],
              sources_json: [],
              created_at: "2026-06-15T00:00:00Z",
              updated_at: "2026-06-15T00:00:00Z",
            },
            findings: [
              {
                title: "Texture proof point",
                insight: "Texture details support premium positioning.",
                evidence: "Texture details support premium positioning.",
                source_index: 1,
                creative_implication: "Use tactile close-ups as visual proof before the model movement.",
              },
            ],
            sources: [
              {
                index: 1,
                title: "Reference note",
                url: "https://example.com/reference",
                source_type: "reference",
                excerpt: "Texture details support premium positioning.",
              },
            ],
            rationale: "Research for Spring Drop Film uses 1 source material.",
            creative_implications: ["Use tactile close-ups as visual proof before the model movement."],
            agent_run: {
              id: "run-research-1",
              project_id: "project-1",
              user_goal: "Find reference angles",
              status: "completed",
              runtime_kind: "hermes_core",
              runtime_mode: "local_research_bridge",
              runtime_version: "0.16.0",
              roles_json: ["researcher", "planner", "reviewer"],
              plan_json: [],
              result_summary: "Research for Spring Drop Film: 1 source-backed finding.",
              created_at: "2026-06-15T00:00:00Z",
              updated_at: "2026-06-15T00:00:00Z",
            },
            agent_events: [],
          });
        case "joi_generate_storyboard":
          return Promise.resolve(mockStoryboardGenerationResult);
        case "joi_update_shot":
          return Promise.resolve({
            shot: {
              ...mockStoryboardGenerationResult.shots[0].shot,
              description: args?.input?.visual_description,
            },
            visual_description: args?.input?.visual_description,
            garment_focus: args?.input?.garment_focus,
            transition: args?.input?.transition,
          });
        case "joi_regenerate_shot":
          return Promise.resolve({
            shot: {
              ...mockStoryboardGenerationResult.shots[1],
              visual_description: "Regenerated macro fabric insert.",
              garment_focus: "fabric texture",
            },
            agent_run: {
              ...mockStoryboardGenerationResult.agent_run,
              id: "run-storyboard-regen",
              runtime_mode: "local_storyboard_regeneration_bridge",
            },
            agent_events: [],
          });
        case "joi_generate_prompt_packages":
          return Promise.resolve(
            mockPromptGenerationResult((args?.input?.target_platforms as string[]) ?? []),
          );
        case "joi_update_prompt_package":
          return Promise.resolve({
            package: {
              id: args?.input?.id ?? "prompt-jimeng_video",
              project_id: "project-1",
              shot_id: "shot-1",
              platform: "jimeng_video",
              modality: "video",
              prompt_text: args?.input?.prompt_text ?? "Edited prompt package text.",
              negative_prompt: args?.input?.negative_prompt ?? "",
              parameters_json: args?.input?.parameters_json ?? {},
              is_locked: args?.input?.is_locked ?? false,
              created_at: "2026-06-15T00:00:00Z",
              updated_at: "2026-06-15T00:00:00Z",
            },
            adapter_display_name: "Jimeng Video",
            shot_label: "Shot 1",
            copy_text: args?.input?.prompt_text ?? "Edited prompt package text.",
            missing_fields: [],
            completeness: [],
            sort_order: 1,
          });
        case "joi_generate_delivery_report":
          return Promise.resolve(mockDeliveryReportResult());
        case "joi_update_delivery_report":
          return Promise.resolve({
            ...mockDeliveryReportResult(args?.input?.markdown as string).report,
            title: args?.input?.title ?? "Spring Drop Film Delivery Report",
            is_final_candidate: args?.input?.is_final_candidate ?? false,
          });
        case "joi_preview_delivery_package":
          return Promise.resolve(mockDeliveryReportResult().package_preview);
        case "joi_export_project":
          return Promise.resolve({
            project_json_path: "D:/tmp/joi-export/spring-drop-film.joi-project.json",
            assets_dir: "D:/tmp/joi-export/spring-drop-film-assets",
            delivery_report_path: "D:/tmp/joi-export/spring-drop-film-delivery-report.md",
          });
        default:
          return Promise.resolve(null);
      }
    });
  });

  test("renders the three-column workspace with live project context", async () => {
    render(<App />);

    expect(await screen.findByRole("button", { name: /Atelier Joi/ })).toBeInTheDocument();
    expect(await screen.findByRole("heading", { name: "Spring Drop Film" })).toBeInTheDocument();

    expect(screen.getByRole("navigation", { name: "Workspace navigation" })).toBeInTheDocument();
    expect(screen.getByRole("main", { name: "Project workspace" })).toBeInTheDocument();
    expect(screen.getByRole("complementary", { name: "Agent workspace" })).toBeInTheDocument();

    expect(screen.getByRole("button", { name: /new brand/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /new project/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /save snapshot/i })).toBeInTheDocument();

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("joi_health_check", undefined);
      expect(invokeMock).toHaveBeenCalledWith("joi_get_agent_runtime_status", undefined);
      expect(invokeMock).toHaveBeenCalledWith("joi_list_brands", undefined);
      expect(invokeMock).toHaveBeenCalledWith("joi_list_projects", { brand_id: "brand-1" });
    });
  });

  test("saves a snapshot for the selected project", async () => {
    render(<App />);

    await screen.findByRole("heading", { name: "Spring Drop Film" });
    fireEvent.click(screen.getByRole("button", { name: /save snapshot/i }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("joi_save_project_snapshot", {
        input: {
          project_id: "project-1",
          label: "Workspace snapshot",
          change_reason: "Saved from 0.11 workspace UI",
        },
      });
    });
  });

  test("creates project memory from the memory workspace", async () => {
    render(<App />);

    await screen.findByRole("heading", { name: "Spring Drop Film" });
    fireEvent.click(screen.getByRole("button", { name: "Memory" }));
    fireEvent.change(screen.getByLabelText("Project memory"), {
      target: { value: "Keep fabric texture visible" },
    });
    fireEvent.change(screen.getByLabelText("Source"), {
      target: { value: "user note" },
    });
    fireEvent.click(screen.getByRole("button", { name: /add memory/i }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("joi_create_memory_entry", {
        input: {
          scope: "project",
          brand_id: "brand-1",
          project_id: "project-1",
          content: "Keep fabric texture visible",
          source: "user note",
        },
      });
    });
  });

  test("generates and accepts memory candidates from the memory workspace", async () => {
    render(<App />);

    await screen.findByRole("heading", { name: "Spring Drop Film" });
    fireEvent.click(screen.getByRole("button", { name: "Memory" }));
    fireEvent.change(screen.getByLabelText("Feedback for memory"), {
      target: { value: "Keep tactile product proof in the opening shot." },
    });
    expect(screen.getByLabelText("Use research reports")).toBeChecked();
    fireEvent.click(screen.getByRole("button", { name: /generate memory candidates/i }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("joi_generate_memory_candidates", {
        input: {
          project_id: "project-1",
          feedback_text: "Keep tactile product proof in the opening shot.",
          include_research_reports: true,
        },
      });
    });
    expect(
      await screen.findByText("Use tactile close-ups as visual proof before the model movement."),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Accept" }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("joi_update_memory_status", {
        input: {
          id: "memory-candidate-1",
          status: "accepted",
        },
      });
    });
  });

  test("generates edits and regenerates storyboard shots", async () => {
    render(<App />);

    await screen.findByRole("heading", { name: "Spring Drop Film" });
    fireEvent.click(screen.getByRole("button", { name: "Storyboard" }));
    fireEvent.change(screen.getByLabelText("Storyboard direction"), {
      target: { value: "Make the opening tactile and premium." },
    });
    fireEvent.click(screen.getByRole("button", { name: /generate storyboard/i }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("joi_generate_storyboard", {
        input: {
          project_id: "project-1",
          user_direction: "Make the opening tactile and premium.",
          preferred_duration_seconds: 15,
          preferred_shot_count: 5,
        },
      });
    });
    expect(await screen.findByText("water-resistant cotton trench silhouette")).toBeInTheDocument();

    fireEvent.click(screen.getAllByRole("button", { name: /edit shot/i })[0]);
    fireEvent.change(screen.getByLabelText("Visual description"), {
      target: { value: "Edited opening product entrance." },
    });
    fireEvent.click(screen.getByRole("button", { name: /save shot/i }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "joi_update_shot",
        expect.objectContaining({
          input: expect.objectContaining({
            id: "shot-1",
            visual_description: "Edited opening product entrance.",
          }),
        }),
      );
    });

    fireEvent.change(screen.getByLabelText("Regeneration note"), {
      target: { value: "Make shot 2 a clearer macro fabric insert." },
    });
    fireEvent.click(screen.getAllByRole("button", { name: /regenerate shot/i })[1]);
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "joi_regenerate_shot",
        expect.objectContaining({
          input: expect.objectContaining({
            project_id: "project-1",
            storyboard_id: "storyboard-1",
            shot_id: "shot-2",
          }),
        }),
      );
    });
  });

  test("generates edits and copies prompt packages", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });
    render(<App />);

    await screen.findByRole("heading", { name: "Spring Drop Film" });
    fireEvent.click(screen.getByRole("button", { name: "Prompts" }));
    expect(await screen.findByText("Jimeng Video")).toBeInTheDocument();

    fireEvent.click(screen.getByLabelText("Shot 1"));
    fireEvent.change(screen.getByLabelText("Prompt direction"), {
      target: { value: "Keep output production-ready." },
    });
    fireEvent.click(screen.getByRole("button", { name: /generate video prompts/i }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("joi_generate_prompt_packages", {
        input: expect.objectContaining({
          project_id: "project-1",
          shot_ids: ["shot-1"],
          target_platforms: ["jimeng_video", "grok_video"],
          user_direction: "Keep output production-ready.",
        }),
      });
    });
    expect(await screen.findByText("Jimeng Video prompt")).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("Image brief"), {
      target: { value: "Full-body ecommerce model photo, warm studio." },
    });
    fireEvent.click(screen.getByRole("button", { name: /generate image prompts/i }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("joi_generate_prompt_packages", {
        input: expect.objectContaining({
          project_id: "project-1",
          shot_ids: [],
          image_brief: "Full-body ecommerce model photo, warm studio.",
          target_platforms: ["banana_2_image", "jimeng_image", "gpt_image_2"],
        }),
      });
    });
    expect(await screen.findByText("GPT Image 2 prompt")).toBeInTheDocument();

    fireEvent.change(screen.getAllByLabelText("Prompt text")[0], {
      target: { value: "Edited prompt package text." },
    });
    fireEvent.click(screen.getAllByRole("button", { name: /save prompt/i })[0]);
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "joi_update_prompt_package",
        expect.objectContaining({
          input: expect.objectContaining({
            prompt_text: "Edited prompt package text.",
          }),
        }),
      );
    });

    fireEvent.click(screen.getAllByRole("button", { name: /copy prompt/i })[0]);
    await waitFor(() => {
      expect(writeText).toHaveBeenCalled();
    });
  });

  test("keeps new brand and project actions in create mode", async () => {
    render(<App />);

    await screen.findByRole("heading", { name: "Spring Drop Film" });

    fireEvent.click(screen.getByRole("button", { name: /new brand/i }));
    await waitFor(() => {
      expect(screen.getByLabelText("Brand name")).toHaveValue("");
    });
    fireEvent.change(screen.getByLabelText("Brand name"), {
      target: { value: "New Atelier" },
    });
    fireEvent.change(screen.getByLabelText("Description"), {
      target: { value: "Studio denim" },
    });
    fireEvent.click(screen.getByRole("button", { name: /create brand/i }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("joi_create_brand", {
        input: {
          name: "New Atelier",
          description: "Studio denim",
        },
      });
    });

    fireEvent.click(screen.getByRole("button", { name: /Atelier Joi/ }));
    fireEvent.click(screen.getByRole("button", { name: /new project/i }));
    await waitFor(() => {
      expect(screen.getByLabelText("Project title")).toHaveValue("");
    });
    fireEvent.change(screen.getByLabelText("Project title"), {
      target: { value: "Lookbook Motion" },
    });
    fireEvent.change(screen.getByLabelText("Advertising goal"), {
      target: { value: "Convert collection interest" },
    });
    fireEvent.change(screen.getByLabelText("Duration seconds"), {
      target: { value: "30" },
    });
    fireEvent.click(screen.getByRole("button", { name: /create project/i }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("joi_create_project", {
        input: {
          brand_id: "brand-1",
          title: "Lookbook Motion",
          advertising_goal: "Convert collection interest",
          duration_seconds: 30,
        },
      });
    });
  });

  test("generates brief and material understanding from the Brief workspace", async () => {
    render(<App />);

    await screen.findByRole("heading", { name: "Spring Drop Film" });
    fireEvent.click(screen.getByRole("button", { name: "Brief" }));
    fireEvent.change(screen.getByLabelText("Project brief"), {
      target: { value: "15 second outerwear launch ad" },
    });
    fireEvent.change(screen.getByLabelText("Product name"), {
      target: { value: "Lightweight trench" },
    });
    fireEvent.change(screen.getByLabelText("Product category"), {
      target: { value: "outerwear" },
    });
    fireEvent.change(screen.getByLabelText("Audience"), {
      target: { value: "urban commuters" },
    });
    fireEvent.change(screen.getByLabelText("Target platforms"), {
      target: { value: "jimeng_video, grok_video" },
    });
    fireEvent.change(screen.getByLabelText("Selling points"), {
      target: { value: "water-resistant cotton, soft structure" },
    });
    fireEvent.change(screen.getByLabelText("Visual direction"), {
      target: { value: "clean studio walk with close fabric texture" },
    });
    fireEvent.change(screen.getByLabelText("Constraints"), {
      target: { value: "avoid heavy winter styling" },
    });
    fireEvent.click(screen.getByRole("button", { name: /generate understanding/i }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("joi_generate_brief_understanding", {
        input: {
          project_id: "project-1",
          brief_text: "15 second outerwear launch ad",
          product_name: "Lightweight trench",
          category: "outerwear",
          audience: "urban commuters",
          target_platforms: ["jimeng_video", "grok_video"],
          selling_points_text: "water-resistant cotton, soft structure",
          visual_direction: "clean studio walk with close fabric texture",
          constraints_text: "avoid heavy winter styling",
          reference_asset_ids: [],
        },
      });
    });
    expect(
      await screen.findByText("Which reference materials should Joi use as visual anchors?"),
    ).toBeInTheDocument();
  });

  test("keeps reference material submission disabled until required fields are filled", async () => {
    render(<App />);

    await screen.findByRole("heading", { name: "Spring Drop Film" });
    fireEvent.click(screen.getByRole("button", { name: "Brief" }));

    const addReference = screen.getByRole("button", { name: /add reference/i });
    expect(addReference).toBeDisabled();

    fireEvent.change(screen.getByLabelText("Reference name"), {
      target: { value: "Studio trench reference" },
    });
    expect(addReference).toBeDisabled();

    fireEvent.change(screen.getByLabelText("Reference URL"), {
      target: { value: "https://example.com/trench-look" },
    });
    expect(addReference).toBeEnabled();
  });

  test("creates link reference material from the Brief workspace", async () => {
    render(<App />);

    await screen.findByRole("heading", { name: "Spring Drop Film" });
    fireEvent.click(screen.getByRole("button", { name: "Brief" }));

    fireEvent.change(screen.getByLabelText("Reference name"), {
      target: { value: "Studio trench reference" },
    });
    fireEvent.change(screen.getByLabelText("Reference URL"), {
      target: { value: "https://example.com/trench-look" },
    });
    fireEvent.click(screen.getByRole("button", { name: /add reference/i }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("joi_create_reference_asset", {
        input: {
          project_id: "project-1",
          kind: "link",
          display_name: "Studio trench reference",
          source_uri: "https://example.com/trench-look",
        },
      });
    });
  });

  test("generates a research report from the Research workspace", async () => {
    render(<App />);

    await screen.findByRole("heading", { name: "Spring Drop Film" });
    fireEvent.click(screen.getByRole("button", { name: "Research" }));
    fireEvent.change(screen.getByLabelText("Research goal"), {
      target: { value: "Find reference angles" },
    });
    fireEvent.change(screen.getByLabelText("Market focus"), {
      target: { value: "outerwear" },
    });
    fireEvent.change(screen.getByLabelText("Platform focus"), {
      target: { value: "jimeng_video, grok_video" },
    });
    fireEvent.change(screen.getByLabelText("Source title"), {
      target: { value: "Reference note" },
    });
    fireEvent.change(screen.getByLabelText("Source URL"), {
      target: { value: "https://example.com/reference" },
    });
    fireEvent.change(screen.getByLabelText("Source excerpt"), {
      target: { value: "Texture details support premium positioning." },
    });
    fireEvent.click(screen.getByRole("button", { name: /generate research report/i }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("joi_generate_research_report", {
        input: {
          project_id: "project-1",
          research_goal: "Find reference angles",
          market_focus: "outerwear",
          platform_focus: ["jimeng_video", "grok_video"],
          source_materials: [
            {
              title: "Reference note",
              url: "https://example.com/reference",
              source_type: "reference",
              excerpt: "Texture details support premium positioning.",
            },
          ],
        },
      });
    });
    expect(await screen.findByText("Texture proof point")).toBeInTheDocument();
  });

  test("generates edits previews and exports delivery reports", async () => {
    render(<App />);

    await screen.findByRole("heading", { name: "Spring Drop Film" });
    fireEvent.click(screen.getByRole("button", { name: "Delivery" }));
    fireEvent.change(screen.getByLabelText("Report direction"), {
      target: { value: "Prepare final handoff." },
    });
    fireEvent.click(screen.getByRole("button", { name: /generate delivery report/i }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("joi_generate_delivery_report", {
        input: {
          project_id: "project-1",
          user_direction: "Prepare final handoff.",
        },
      });
    });
    expect(await screen.findByText("spring-drop-film-delivery-report.md")).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("Markdown report"), {
      target: { value: "# Edited Delivery Report" },
    });
    expect(screen.getByLabelText("Markdown report")).toHaveValue("# Edited Delivery Report");
    fireEvent.click(screen.getByRole("button", { name: /save report/i }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith(
        "joi_update_delivery_report",
        expect.objectContaining({
          input: expect.objectContaining({
            id: "delivery-report-1",
            markdown: "# Edited Delivery Report",
          }),
        }),
      );
    });

    fireEvent.click(screen.getByRole("button", { name: /preview package/i }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("joi_preview_delivery_package", {
        input: {
          project_id: "project-1",
          delivery_report_id: "delivery-report-1",
        },
      });
    });

    fireEvent.change(screen.getByLabelText("Export directory"), {
      target: { value: "D:/tmp/joi-export" },
    });
    fireEvent.click(screen.getByRole("button", { name: /export package/i }));
    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("joi_export_project", {
        input: {
          project_id: "project-1",
          export_dir: "D:/tmp/joi-export",
          delivery_report_id: "delivery-report-1",
        },
      });
    });
  });

  test("starts an agent plan from the Agent panel", async () => {
    render(<App />);

    await screen.findByRole("heading", { name: "Spring Drop Film" });
    expect(await screen.findByText("Hermes Core")).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("Agent goal"), {
      target: { value: "Plan the next content workflow steps" },
    });
    fireEvent.click(screen.getByRole("button", { name: /start plan/i }));

    await waitFor(() => {
      expect(invokeMock).toHaveBeenCalledWith("joi_start_agent_plan", {
        input: {
          project_id: "project-1",
          user_goal: "Plan the next content workflow steps",
        },
      });
    });
    expect(await screen.findByText("Read saved context for Spring Drop Film.")).toBeInTheDocument();
  });
});
