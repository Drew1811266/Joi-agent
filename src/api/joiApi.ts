import { invoke } from "@tauri-apps/api/core";

import type {
  ApplyReviewSuggestionInput,
  ApplyReviewSuggestionResult,
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
  DeliveryPackagePreview,
  DeliveryPackagePreviewInput,
  DeliveryReport,
  DeliveryReportGenerationInput,
  DeliveryReportGenerationResult,
  DeliveryReportUpdateInput,
  HealthResponse,
  MemoryCurationInput,
  MemoryCurationResult,
  MemoryEntry,
  MemoryEntryInput,
  MemoryListInput,
  MemoryStatusInput,
  PromptAdapterProfile,
  PromptGenerationInput,
  PromptGenerationResult,
  PromptPackageUpdateInput,
  PromptPackageView,
  Project,
  ProjectExportCommandInput,
  ProjectExportCommandResult,
  ProjectInput,
  ProjectUpdateInput,
  ProjectVersion,
  ProductUnderstanding,
  QualityReview,
  QualityReviewGenerationInput,
  QualityReviewGenerationResult,
  ReferenceAssetInput,
  ResearchReport,
  ResearchReportInput,
  ResearchReportResult,
  SnapshotInput,
  ShotRegenerationInput,
  ShotRegenerationResult,
  ShotUpdateInput,
  StoryboardGenerationInput,
  StoryboardGenerationResult,
  StoryboardShotView,
  StoryboardWithShots,
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

export function generateMemoryCandidates(input: MemoryCurationInput): Promise<MemoryCurationResult> {
  return invoke<MemoryCurationResult>("joi_generate_memory_candidates", { input });
}

export function updateMemoryStatus(input: MemoryStatusInput): Promise<MemoryEntry> {
  return invoke<MemoryEntry>("joi_update_memory_status", { input });
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

export function generateResearchReport(input: ResearchReportInput): Promise<ResearchReportResult> {
  return invoke<ResearchReportResult>("joi_generate_research_report", { input });
}

export function listResearchReports(projectId: string): Promise<ResearchReport[]> {
  return invoke<ResearchReport[]>("joi_list_research_reports", { project_id: projectId });
}

export function generateStoryboard(input: StoryboardGenerationInput): Promise<StoryboardGenerationResult> {
  return invoke<StoryboardGenerationResult>("joi_generate_storyboard", { input });
}

export function listStoryboards(projectId: string): Promise<StoryboardWithShots[]> {
  return invoke<StoryboardWithShots[]>("joi_list_storyboards", { project_id: projectId });
}

export function updateShot(input: ShotUpdateInput): Promise<StoryboardShotView> {
  return invoke<StoryboardShotView>("joi_update_shot", { input });
}

export function regenerateShot(input: ShotRegenerationInput): Promise<ShotRegenerationResult> {
  return invoke<ShotRegenerationResult>("joi_regenerate_shot", { input });
}

export function getPromptAdapterProfiles(): Promise<PromptAdapterProfile[]> {
  return invoke<PromptAdapterProfile[]>("joi_get_prompt_adapter_profiles");
}

export function generatePromptPackages(
  input: PromptGenerationInput,
): Promise<PromptGenerationResult> {
  return invoke<PromptGenerationResult>("joi_generate_prompt_packages", { input });
}

export function listPromptPackages(projectId: string): Promise<PromptPackageView[]> {
  return invoke<PromptPackageView[]>("joi_list_prompt_packages", { project_id: projectId });
}

export function updatePromptPackage(input: PromptPackageUpdateInput): Promise<PromptPackageView> {
  return invoke<PromptPackageView>("joi_update_prompt_package", { input });
}

export function generateQualityReview(
  input: QualityReviewGenerationInput,
): Promise<QualityReviewGenerationResult> {
  return invoke<QualityReviewGenerationResult>("joi_generate_quality_review", { input });
}

export function listQualityReviews(projectId: string): Promise<QualityReview[]> {
  return invoke<QualityReview[]>("joi_list_quality_reviews", { project_id: projectId });
}

export function applyQualityReviewSuggestion(
  input: ApplyReviewSuggestionInput,
): Promise<ApplyReviewSuggestionResult> {
  return invoke<ApplyReviewSuggestionResult>("joi_apply_quality_review_suggestion", { input });
}

export function generateDeliveryReport(
  input: DeliveryReportGenerationInput,
): Promise<DeliveryReportGenerationResult> {
  return invoke<DeliveryReportGenerationResult>("joi_generate_delivery_report", { input });
}

export function listDeliveryReports(projectId: string): Promise<DeliveryReport[]> {
  return invoke<DeliveryReport[]>("joi_list_delivery_reports", { project_id: projectId });
}

export function updateDeliveryReport(input: DeliveryReportUpdateInput): Promise<DeliveryReport> {
  return invoke<DeliveryReport>("joi_update_delivery_report", { input });
}

export function previewDeliveryPackage(
  input: DeliveryPackagePreviewInput,
): Promise<DeliveryPackagePreview> {
  return invoke<DeliveryPackagePreview>("joi_preview_delivery_package", { input });
}

export function exportProject(input: ProjectExportCommandInput): Promise<ProjectExportCommandResult> {
  return invoke<ProjectExportCommandResult>("joi_export_project", { input });
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
