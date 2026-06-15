import { invoke } from "@tauri-apps/api/core";

import type {
  AgentPlanInput,
  AgentPlanResult,
  AgentRunWithEvents,
  AgentRuntimeStatus,
  Asset,
  BriefUnderstandingInput,
  BriefUnderstandingResult,
  Brand,
  BrandInput,
  BrandUpdateInput,
  CreativeDirection,
  HealthResponse,
  MemoryEntry,
  MemoryEntryInput,
  MemoryListInput,
  Project,
  ProjectInput,
  ProjectUpdateInput,
  ProjectVersion,
  ProductUnderstanding,
  ReferenceAssetInput,
  SnapshotInput,
} from "../types/joi";

export function formatError(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string") {
    return error;
  }
  try {
    return JSON.stringify(error);
  } catch {
    return "Unknown error";
  }
}

export function healthCheck(): Promise<HealthResponse> {
  return invoke<HealthResponse>("joi_health_check");
}

export function createBrand(input: BrandInput): Promise<Brand> {
  return invoke<Brand>("joi_create_brand", { input });
}

export function listBrands(): Promise<Brand[]> {
  return invoke<Brand[]>("joi_list_brands");
}

export function updateBrand(input: BrandUpdateInput): Promise<Brand> {
  return invoke<Brand>("joi_update_brand", { input });
}

export function createProject(input: ProjectInput): Promise<Project> {
  return invoke<Project>("joi_create_project", { input });
}

export function listProjects(brandId: string | null): Promise<Project[]> {
  return invoke<Project[]>("joi_list_projects", { brand_id: brandId });
}

export function updateProject(input: ProjectUpdateInput): Promise<Project> {
  return invoke<Project>("joi_update_project", { input });
}

export function listAssets(projectId: string): Promise<Asset[]> {
  return invoke<Asset[]>("joi_list_assets", { project_id: projectId });
}

export function createReferenceAsset(input: ReferenceAssetInput): Promise<Asset> {
  return invoke<Asset>("joi_create_reference_asset", { input });
}

export function listProjectVersions(projectId: string): Promise<ProjectVersion[]> {
  return invoke<ProjectVersion[]>("joi_list_project_versions", { project_id: projectId });
}

export function saveProjectSnapshot(input: SnapshotInput): Promise<ProjectVersion> {
  return invoke<ProjectVersion>("joi_save_project_snapshot", { input });
}

export function createMemoryEntry(input: MemoryEntryInput): Promise<MemoryEntry> {
  return invoke<MemoryEntry>("joi_create_memory_entry", { input });
}

export function listMemoryEntries(input: MemoryListInput): Promise<MemoryEntry[]> {
  return invoke<MemoryEntry[]>("joi_list_memory_entries", { input });
}

export function generateBriefUnderstanding(
  input: BriefUnderstandingInput,
): Promise<BriefUnderstandingResult> {
  return invoke<BriefUnderstandingResult>("joi_generate_brief_understanding", { input });
}

export function listProductUnderstandings(projectId: string): Promise<ProductUnderstanding[]> {
  return invoke<ProductUnderstanding[]>("joi_list_product_understandings", { project_id: projectId });
}

export function listCreativeDirections(projectId: string): Promise<CreativeDirection[]> {
  return invoke<CreativeDirection[]>("joi_list_creative_directions", { project_id: projectId });
}

export function getAgentRuntimeStatus(): Promise<AgentRuntimeStatus> {
  return invoke<AgentRuntimeStatus>("joi_get_agent_runtime_status");
}

export function startAgentPlan(input: AgentPlanInput): Promise<AgentPlanResult> {
  return invoke<AgentPlanResult>("joi_start_agent_plan", { input });
}

export function getAgentRun(id: string): Promise<AgentRunWithEvents> {
  return invoke<AgentRunWithEvents>("joi_get_agent_run", { id });
}

export function listAgentRuns(projectId: string): Promise<AgentRunWithEvents[]> {
  return invoke<AgentRunWithEvents[]>("joi_list_agent_runs", { project_id: projectId });
}
