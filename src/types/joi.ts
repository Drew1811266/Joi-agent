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
