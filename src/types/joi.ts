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
