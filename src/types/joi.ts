export type HealthResponse = {
  status: string;
  app_name: string;
  phase: string;
};

export type Brand = {
  id: string;
  name: string;
  description: string;
  style_keywords: string[];
  visual_preferences: string[];
  negative_preferences: string[];
  common_scenes: string[];
  model_preferences: string[];
  platform_preferences: string[];
  created_at: string;
  updated_at: string;
};

export type Project = {
  id: string;
  brand_id: string;
  title: string;
  advertising_goal: string;
  duration_seconds: number;
  target_platforms: string[];
  content_type: string;
  status: string;
  current_version_id: string | null;
  final_version_id: string | null;
  created_at: string;
  updated_at: string;
};

export type Asset = {
  id: string;
  project_id: string;
  kind: string;
  display_name: string;
  relative_path: string;
  source_uri: string;
  mime_type: string;
  file_size_bytes: number;
  sha256: string;
  metadata_json?: unknown;
  created_at: string;
  updated_at: string;
};

export type ProductUnderstanding = {
  id: string;
  project_id: string;
  product_name: string;
  category: string;
  audience: string;
  selling_points_json: unknown;
  constraints_json: unknown;
  notes: string;
  created_at: string;
  updated_at: string;
};

export type CreativeDirection = {
  id: string;
  project_id: string;
  title: string;
  concept: string;
  tone: string;
  visual_style: string;
  scene_direction: string;
  rationale: string;
  created_at: string;
  updated_at: string;
};

export type ResearchReport = {
  id: string;
  project_id: string;
  summary: string;
  findings_json: unknown;
  sources_json: unknown;
  created_at: string;
  updated_at: string;
};

export type Storyboard = {
  id: string;
  project_id: string;
  title: string;
  duration_seconds: number;
  created_at: string;
  updated_at: string;
};

export type Shot = {
  id: string;
  storyboard_id: string;
  shot_number: number;
  duration_seconds: number;
  description: string;
  model_action: string;
  camera_movement: string;
  scene: string;
  lighting: string;
  subtitle_or_voiceover: string;
  rationale: string;
  is_locked: boolean;
  metadata_json: unknown;
  created_at: string;
  updated_at: string;
};

export type StoryboardWithShots = {
  storyboard: Storyboard;
  shots: Shot[];
};

export type PromptPackage = {
  id: string;
  project_id: string;
  shot_id: string | null;
  platform: string;
  modality: string;
  prompt_text: string;
  negative_prompt: string;
  parameters_json: unknown;
  is_locked: boolean;
  created_at: string;
  updated_at: string;
};

export type DeliveryReport = {
  id: string;
  project_id: string;
  title: string;
  markdown: string;
  sections_json: unknown;
  is_final_candidate: boolean;
  created_at: string;
  updated_at: string;
};

export type ProjectVersion = {
  id: string;
  project_id: string;
  version_number: number;
  label: string;
  change_reason: string;
  changed_entities: string[];
  snapshot_json: unknown;
  created_by: string;
  is_final_candidate: boolean;
  created_at: string;
};

export type MemoryEntry = {
  id: string;
  scope: string;
  brand_id: string | null;
  project_id: string | null;
  content: string;
  source: string;
  source_entity_type: string;
  source_entity_id: string;
  confidence: number;
  status: string;
  created_at: string;
  updated_at: string;
};

export type BrandInput = {
  name: string;
  description: string;
};

export type BrandUpdateInput = BrandInput & {
  id: string;
};

export type ProjectInput = {
  brand_id: string;
  title: string;
  advertising_goal: string;
  duration_seconds: number;
};

export type ProjectUpdateInput = {
  id: string;
  title: string;
  advertising_goal: string;
  duration_seconds: number;
};

export type MemoryEntryInput = {
  scope: string;
  brand_id: string | null;
  project_id: string | null;
  content: string;
  source: string;
};

export type MemoryListInput = {
  scope: string;
  brand_id: string | null;
  project_id: string | null;
};

export type MemoryCurationInput = {
  project_id: string;
  feedback_text: string;
  include_research_reports: boolean;
};

export type MemoryCandidateResult = {
  entry: MemoryEntry;
  reason: string;
  has_conflict: boolean;
  conflict_memory_ids: string[];
};

export type MemoryCurationResult = {
  candidates: MemoryCandidateResult[];
  agent_run: AgentRun;
  agent_events: AgentRunEvent[];
};

export type MemoryStatusInput = {
  id: string;
  status: "proposed" | "accepted" | "rejected";
};

export type SnapshotInput = {
  project_id: string;
  label: string | null;
  change_reason: string | null;
};

export type BriefUnderstandingInput = {
  project_id: string;
  brief_text: string;
  product_name: string;
  category: string;
  audience: string;
  target_platforms: string[];
  selling_points_text: string;
  visual_direction: string;
  constraints_text: string;
  reference_asset_ids: string[];
};

export type BriefUnderstandingResult = {
  product_understanding: ProductUnderstanding;
  creative_direction: CreativeDirection | null;
  brief_summary: string;
  brand_summary: string;
  visual_direction: string;
  selling_points: string[];
  constraints: string[];
  missing_questions: string[];
};

export type ReferenceAssetInput = {
  project_id: string;
  kind: string;
  display_name: string;
  source_uri: string;
};

export type ResearchSourceInput = {
  title: string;
  url: string;
  source_type: string;
  excerpt: string;
};

export type ResearchReportInput = {
  project_id: string;
  research_goal: string;
  market_focus: string;
  platform_focus: string[];
  source_materials: ResearchSourceInput[];
};

export type StoryboardGenerationInput = {
  project_id: string;
  user_direction: string;
  preferred_duration_seconds: number | null;
  preferred_shot_count: number | null;
};

export type StoryboardShotView = {
  shot: Shot;
  visual_description: string;
  garment_focus: string;
  transition: string;
};

export type StoryboardGenerationResult = {
  storyboard: Storyboard;
  shots: StoryboardShotView[];
  total_duration_seconds: number;
  agent_run: AgentRun;
  agent_events: AgentRunEvent[];
};

export type ShotUpdateInput = {
  id: string;
  duration_seconds: number;
  visual_description: string;
  model_action: string;
  garment_focus: string;
  camera_movement: string;
  scene: string;
  lighting: string;
  transition: string;
  subtitle_or_text: string;
  rationale: string;
  is_locked: boolean;
};

export type ShotRegenerationInput = {
  project_id: string;
  storyboard_id: string;
  shot_id: string;
  revision_note: string;
};

export type ShotRegenerationResult = {
  shot: StoryboardShotView;
  agent_run: AgentRun;
  agent_events: AgentRunEvent[];
};

export type PromptAdapterProfile = {
  id: string;
  display_name: string;
  modality: string;
  default_negative_prompt: string;
  required_fields: string[];
};

export type PromptGenerationInput = {
  project_id: string;
  shot_ids: string[];
  image_brief: string;
  target_platforms: string[];
  user_direction: string;
};

export type PromptCompletenessCheck = {
  field: string;
  label: string;
  present: boolean;
  message: string;
};

export type PromptPackageView = {
  package: PromptPackage;
  adapter_display_name: string;
  completeness: PromptCompletenessCheck[];
  missing_fields: string[];
  copy_text: string;
};

export type PromptGenerationResult = {
  packages: PromptPackageView[];
  agent_run: AgentRun;
  agent_events: AgentRunEvent[];
};

export type PromptPackageUpdateInput = {
  id: string;
  prompt_text: string;
  negative_prompt: string;
  parameters_json: unknown;
  is_locked: boolean;
};

export type DeliveryReportGenerationInput = {
  project_id: string;
  user_direction: string;
};

export type DeliveryReportSectionStatus = {
  id: string;
  title: string;
  status: "complete" | "partial" | "missing" | string;
  source_count: number;
  warning: string;
};

export type DeliveryPackagePreview = {
  project_json_file_name: string;
  assets_folder_name: string;
  delivery_report_file_name: string;
  included_assets_count: number;
  included_prompt_packages_count: number;
  included_storyboards_count: number;
  warnings: string[];
};

export type DeliveryReportGenerationResult = {
  report: DeliveryReport;
  sections: DeliveryReportSectionStatus[];
  package_preview: DeliveryPackagePreview;
  agent_run: AgentRun;
  agent_events: AgentRunEvent[];
};

export type DeliveryReportUpdateInput = {
  id: string;
  title: string;
  markdown: string;
  sections_json: unknown;
  is_final_candidate: boolean;
};

export type DeliveryPackagePreviewInput = {
  project_id: string;
  delivery_report_id: string | null;
};

export type ProjectExportCommandInput = {
  project_id: string;
  export_dir: string;
  delivery_report_id: string | null;
};

export type ProjectExportCommandResult = {
  project_json_path: string;
  assets_dir: string;
  delivery_report_path: string | null;
};

export type ResearchFinding = {
  title: string;
  insight: string;
  evidence: string;
  source_index: number;
  creative_implication: string;
};

export type ResearchSourceCitation = {
  index: number;
  title: string;
  url: string;
  source_type: string;
  excerpt: string;
};

export type AgentRuntimeStatus = {
  runtime_kind: string;
  runtime_mode: string;
  hermes_checkout_path: string;
  hermes_present: boolean;
  hermes_version: string;
  phase0_report_present: boolean;
  ready: boolean;
  message: string;
};

export type AgentRun = {
  id: string;
  project_id: string;
  user_goal: string;
  status: string;
  runtime_kind: string;
  runtime_mode: string;
  runtime_version: string;
  roles_json: unknown;
  plan_json: unknown;
  result_summary: string;
  created_at: string;
  updated_at: string;
};

export type AgentRunEvent = {
  id: string;
  agent_run_id: string;
  sequence_number: number;
  role: string;
  event_type: string;
  message: string;
  payload_json: unknown;
  created_at: string;
};

export type AgentPlanInput = {
  project_id: string;
  user_goal: string;
};

export type AgentPlanResult = {
  run: AgentRun;
  events: AgentRunEvent[];
};

export type AgentRunWithEvents = {
  run: AgentRun;
  events: AgentRunEvent[];
};

export type ResearchReportResult = {
  report: ResearchReport;
  findings: ResearchFinding[];
  sources: ResearchSourceCitation[];
  rationale: string;
  creative_implications: string[];
  agent_run: AgentRun;
  agent_events: AgentRunEvent[];
};
