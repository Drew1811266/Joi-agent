import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";

import App from "./App";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (command: string, payload?: unknown) => invokeMock(command, payload),
}));

describe("Joi workspace shell", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    invokeMock.mockImplementation((command: string) => {
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
          return Promise.resolve([]);
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
});
