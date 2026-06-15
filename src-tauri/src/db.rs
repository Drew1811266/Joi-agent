use std::path::Path;

use rusqlite::Connection;

use crate::error::JoiResult;

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> JoiResult<Self> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(path)?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        Ok(Self { connection })
    }

    pub fn open_in_memory() -> JoiResult<Self> {
        let connection = Connection::open_in_memory()?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        Ok(Self { connection })
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    pub fn migrate(&self) -> JoiResult<()> {
        self.connection.execute_batch(SCHEMA)?;
        self.migrate_prompt_packages_optional_shot()?;
        Ok(())
    }

    pub fn table_names(&self) -> JoiResult<Vec<String>> {
        let mut statement = self.connection.prepare(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        let mut names = Vec::new();
        for row in rows {
            names.push(row?);
        }
        Ok(names)
    }

    fn migrate_prompt_packages_optional_shot(&self) -> JoiResult<()> {
        let shot_id_not_null = {
            let mut statement = self
                .connection
                .prepare("PRAGMA table_info(prompt_packages)")?;
            let rows = statement.query_map([], |row| {
                let name: String = row.get(1)?;
                let not_null: i64 = row.get(3)?;
                Ok((name, not_null))
            })?;
            let mut shot_id_not_null = false;
            for row in rows {
                let (name, not_null) = row?;
                if name == "shot_id" {
                    shot_id_not_null = not_null == 1;
                }
            }
            shot_id_not_null
        };

        if !shot_id_not_null {
            return Ok(());
        }

        self.connection.execute_batch(
            r#"
            DROP TRIGGER IF EXISTS trg_prompt_packages_shot_belongs_to_project_insert;
            DROP TRIGGER IF EXISTS trg_prompt_packages_shot_belongs_to_project_update;
            DROP INDEX IF EXISTS idx_prompt_packages_project_id;
            DROP INDEX IF EXISTS idx_prompt_packages_shot_id;
            ALTER TABLE prompt_packages RENAME TO prompt_packages_legacy;
            CREATE TABLE prompt_packages (
              id TEXT PRIMARY KEY,
              project_id TEXT NOT NULL,
              shot_id TEXT,
              platform TEXT NOT NULL,
              modality TEXT NOT NULL,
              prompt_text TEXT NOT NULL DEFAULT '',
              negative_prompt TEXT NOT NULL DEFAULT '',
              parameters_json TEXT NOT NULL DEFAULT '{}',
              is_locked INTEGER NOT NULL DEFAULT 0,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL,
              CHECK (shot_id IS NOT NULL OR modality = 'image'),
              FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
              FOREIGN KEY (shot_id) REFERENCES shots(id) ON DELETE CASCADE
            );
            INSERT INTO prompt_packages (
              id, project_id, shot_id, platform, modality, prompt_text, negative_prompt,
              parameters_json, is_locked, created_at, updated_at
            )
            SELECT id, project_id, shot_id, platform, modality, prompt_text, negative_prompt,
                   parameters_json, is_locked, created_at, updated_at
            FROM prompt_packages_legacy;
            DROP TABLE prompt_packages_legacy;
            "#,
        )?;
        self.connection.execute_batch(SCHEMA)?;
        Ok(())
    }
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS brands (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  style_keywords_json TEXT NOT NULL DEFAULT '[]',
  visual_preferences_json TEXT NOT NULL DEFAULT '{}',
  negative_preferences_json TEXT NOT NULL DEFAULT '[]',
  common_scenes_json TEXT NOT NULL DEFAULT '[]',
  model_preferences_json TEXT NOT NULL DEFAULT '{}',
  platform_preferences_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS projects (
  id TEXT PRIMARY KEY,
  brand_id TEXT NOT NULL,
  title TEXT NOT NULL,
  advertising_goal TEXT NOT NULL DEFAULT '',
  duration_seconds INTEGER NOT NULL DEFAULT 15,
  target_platforms_json TEXT NOT NULL DEFAULT '[]',
  workflow_stage TEXT NOT NULL DEFAULT 'created',
  current_version_id TEXT,
  final_version_id TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS assets (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  display_name TEXT NOT NULL,
  relative_path TEXT NOT NULL DEFAULT '',
  source_uri TEXT NOT NULL DEFAULT '',
  mime_type TEXT NOT NULL DEFAULT '',
  file_size_bytes INTEGER NOT NULL DEFAULT 0,
  sha256 TEXT NOT NULL DEFAULT '',
  metadata_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS research_reports (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  summary TEXT NOT NULL DEFAULT '',
  findings_json TEXT NOT NULL DEFAULT '[]',
  sources_json TEXT NOT NULL DEFAULT '[]',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS product_understandings (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  product_name TEXT NOT NULL DEFAULT '',
  category TEXT NOT NULL DEFAULT '',
  audience TEXT NOT NULL DEFAULT '',
  selling_points_json TEXT NOT NULL DEFAULT '[]',
  constraints_json TEXT NOT NULL DEFAULT '[]',
  notes TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS creative_directions (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  title TEXT NOT NULL,
  concept TEXT NOT NULL DEFAULT '',
  tone TEXT NOT NULL DEFAULT '',
  visual_style TEXT NOT NULL DEFAULT '',
  scene_direction TEXT NOT NULL DEFAULT '',
  rationale TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS storyboards (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  title TEXT NOT NULL DEFAULT '',
  duration_seconds INTEGER NOT NULL DEFAULT 15,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS shots (
  id TEXT PRIMARY KEY,
  storyboard_id TEXT NOT NULL,
  shot_number INTEGER NOT NULL,
  duration_seconds INTEGER NOT NULL DEFAULT 0,
  description TEXT NOT NULL DEFAULT '',
  model_action TEXT NOT NULL DEFAULT '',
  camera_movement TEXT NOT NULL DEFAULT '',
  scene TEXT NOT NULL DEFAULT '',
  lighting TEXT NOT NULL DEFAULT '',
  subtitle_or_voiceover TEXT NOT NULL DEFAULT '',
  rationale TEXT NOT NULL DEFAULT '',
  is_locked INTEGER NOT NULL DEFAULT 0,
  metadata_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (storyboard_id) REFERENCES storyboards(id) ON DELETE CASCADE,
  UNIQUE(storyboard_id, shot_number)
);

CREATE TABLE IF NOT EXISTS prompt_packages (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  shot_id TEXT,
  platform TEXT NOT NULL,
  modality TEXT NOT NULL,
  prompt_text TEXT NOT NULL DEFAULT '',
  negative_prompt TEXT NOT NULL DEFAULT '',
  parameters_json TEXT NOT NULL DEFAULT '{}',
  is_locked INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  CHECK (shot_id IS NOT NULL OR modality = 'image'),
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
  FOREIGN KEY (shot_id) REFERENCES shots(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS project_versions (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  version_number INTEGER NOT NULL,
  label TEXT NOT NULL DEFAULT '',
  change_reason TEXT NOT NULL DEFAULT '',
  changed_entities_json TEXT NOT NULL DEFAULT '[]',
  snapshot_json TEXT NOT NULL,
  created_by TEXT NOT NULL DEFAULT 'user',
  is_final_candidate INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
  UNIQUE(project_id, version_number)
);

CREATE TABLE IF NOT EXISTS memory_entries (
  id TEXT PRIMARY KEY,
  scope TEXT NOT NULL,
  brand_id TEXT,
  project_id TEXT,
  content TEXT NOT NULL,
  source TEXT NOT NULL DEFAULT '',
  source_entity_type TEXT NOT NULL DEFAULT '',
  source_entity_id TEXT NOT NULL DEFAULT '',
  confidence REAL NOT NULL DEFAULT 0.0,
  status TEXT NOT NULL DEFAULT 'proposed',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (brand_id) REFERENCES brands(id) ON DELETE CASCADE,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS agent_runs (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  user_goal TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL DEFAULT 'completed',
  runtime_kind TEXT NOT NULL DEFAULT 'hermes_core',
  runtime_mode TEXT NOT NULL DEFAULT 'local_planner_bridge',
  runtime_version TEXT NOT NULL DEFAULT '',
  roles_json TEXT NOT NULL DEFAULT '[]',
  plan_json TEXT NOT NULL DEFAULT '[]',
  result_summary TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS agent_run_events (
  id TEXT PRIMARY KEY,
  agent_run_id TEXT NOT NULL,
  sequence_number INTEGER NOT NULL,
  role TEXT NOT NULL DEFAULT '',
  event_type TEXT NOT NULL DEFAULT '',
  message TEXT NOT NULL DEFAULT '',
  payload_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL,
  FOREIGN KEY (agent_run_id) REFERENCES agent_runs(id) ON DELETE CASCADE,
  UNIQUE(agent_run_id, sequence_number)
);

CREATE TRIGGER IF NOT EXISTS trg_prompt_packages_shot_belongs_to_project_insert
BEFORE INSERT ON prompt_packages
FOR EACH ROW
WHEN NEW.shot_id IS NOT NULL
  AND EXISTS (SELECT 1 FROM shots WHERE shots.id = NEW.shot_id)
  AND EXISTS (SELECT 1 FROM projects WHERE projects.id = NEW.project_id)
  AND NOT EXISTS (
    SELECT 1
    FROM shots
    JOIN storyboards ON storyboards.id = shots.storyboard_id
    WHERE shots.id = NEW.shot_id
      AND storyboards.project_id = NEW.project_id
  )
BEGIN
  SELECT RAISE(ABORT, 'prompt package shot must belong to project');
END;

CREATE TRIGGER IF NOT EXISTS trg_prompt_packages_shot_belongs_to_project_update
BEFORE UPDATE OF project_id, shot_id ON prompt_packages
FOR EACH ROW
WHEN NEW.shot_id IS NOT NULL
  AND EXISTS (SELECT 1 FROM shots WHERE shots.id = NEW.shot_id)
  AND EXISTS (SELECT 1 FROM projects WHERE projects.id = NEW.project_id)
  AND NOT EXISTS (
    SELECT 1
    FROM shots
    JOIN storyboards ON storyboards.id = shots.storyboard_id
    WHERE shots.id = NEW.shot_id
      AND storyboards.project_id = NEW.project_id
  )
BEGIN
  SELECT RAISE(ABORT, 'prompt package shot must belong to project');
END;

CREATE INDEX IF NOT EXISTS idx_projects_brand_id ON projects(brand_id);
CREATE INDEX IF NOT EXISTS idx_assets_project_id ON assets(project_id);
CREATE INDEX IF NOT EXISTS idx_research_reports_project_id ON research_reports(project_id);
CREATE INDEX IF NOT EXISTS idx_product_understandings_project_id ON product_understandings(project_id);
CREATE INDEX IF NOT EXISTS idx_creative_directions_project_id ON creative_directions(project_id);
CREATE INDEX IF NOT EXISTS idx_storyboards_project_id ON storyboards(project_id);
CREATE INDEX IF NOT EXISTS idx_shots_storyboard_id ON shots(storyboard_id);
CREATE INDEX IF NOT EXISTS idx_prompt_packages_project_id ON prompt_packages(project_id);
CREATE INDEX IF NOT EXISTS idx_prompt_packages_shot_id ON prompt_packages(shot_id);
CREATE INDEX IF NOT EXISTS idx_project_versions_project_id ON project_versions(project_id);
CREATE INDEX IF NOT EXISTS idx_memory_entries_scope ON memory_entries(scope);
CREATE INDEX IF NOT EXISTS idx_memory_entries_brand_id ON memory_entries(brand_id);
CREATE INDEX IF NOT EXISTS idx_memory_entries_project_id ON memory_entries(project_id);
CREATE INDEX IF NOT EXISTS idx_agent_runs_project_id ON agent_runs(project_id);
CREATE INDEX IF NOT EXISTS idx_agent_run_events_agent_run_id ON agent_run_events(agent_run_id);
"#;
