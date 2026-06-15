use chrono::Utc;
use rusqlite::{params, Connection};
use serde_json::{json, Value};

use crate::error::{JoiError, JoiResult};
use crate::models::{
    new_id, AgentRun, AgentRunEvent, Asset, AssetKind, Brand, CreativeDirection, DeliveryReport,
    MemoryEntry, MemoryScope, MemoryStatus, ProductUnderstanding, Project, ProjectVersion,
    PromptModality, PromptPackage, PromptPlatform, QualityReview, ResearchReport, Shot, Storyboard,
};
use crate::validation::{validate_non_negative, validate_prompt_modality, validate_required_text};

pub struct Repository<'a> {
    connection: &'a Connection,
}

#[derive(Debug, Clone)]
pub struct BrandCreate {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct BrandUpdate {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ProjectCreate {
    pub brand_id: String,
    pub title: String,
    pub advertising_goal: String,
    pub duration_seconds: i64,
}

#[derive(Debug, Clone)]
pub struct ProjectUpdate {
    pub id: String,
    pub title: String,
    pub advertising_goal: String,
    pub duration_seconds: i64,
}

#[derive(Debug, Clone)]
pub struct AssetCreate {
    pub project_id: String,
    pub kind: String,
    pub display_name: String,
    pub relative_path: String,
    pub source_uri: String,
    pub mime_type: String,
    pub file_size_bytes: i64,
    pub sha256: String,
}

#[derive(Debug, Clone)]
pub struct ResearchReportCreate {
    pub project_id: String,
    pub summary: String,
    pub findings_json: Value,
    pub sources_json: Value,
}

#[derive(Debug, Clone)]
pub struct ProductUnderstandingCreate {
    pub project_id: String,
    pub product_name: String,
    pub category: String,
    pub audience: String,
    pub selling_points: Vec<String>,
    pub constraints: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct CreativeDirectionCreate {
    pub project_id: String,
    pub title: String,
    pub concept: String,
    pub tone: String,
    pub visual_style: String,
    pub scene_direction: String,
    pub rationale: String,
}

#[derive(Debug, Clone)]
pub struct StoryboardCreate {
    pub project_id: String,
    pub title: String,
    pub duration_seconds: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoryboardWithShots {
    pub storyboard: Storyboard,
    pub shots: Vec<Shot>,
}

#[derive(Debug, Clone)]
pub struct ShotCreate {
    pub storyboard_id: String,
    pub shot_number: i64,
    pub duration_seconds: i64,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ShotPlanCreate {
    pub storyboard_id: String,
    pub shot_number: i64,
    pub duration_seconds: i64,
    pub visual_description: String,
    pub model_action: String,
    pub garment_focus: String,
    pub camera_movement: String,
    pub scene: String,
    pub lighting: String,
    pub transition: String,
    pub subtitle_or_text: String,
    pub rationale: String,
    pub source_memory_ids: Vec<String>,
    pub source_research_report_ids: Vec<String>,
    pub generation_context: Value,
}

#[derive(Debug, Clone)]
pub struct ShotUpdate {
    pub id: String,
    pub duration_seconds: i64,
    pub visual_description: String,
    pub model_action: String,
    pub garment_focus: String,
    pub camera_movement: String,
    pub scene: String,
    pub lighting: String,
    pub transition: String,
    pub subtitle_or_text: String,
    pub rationale: String,
    pub is_locked: bool,
}

#[derive(Debug, Clone)]
pub struct PromptPackageCreate {
    pub project_id: String,
    pub shot_id: Option<String>,
    pub platform: String,
    pub modality: String,
    pub prompt_text: String,
    pub negative_prompt: String,
    pub parameters_json: Value,
}

#[derive(Debug, Clone)]
pub struct PromptPackageUpdate {
    pub id: String,
    pub prompt_text: String,
    pub negative_prompt: String,
    pub parameters_json: Value,
    pub is_locked: bool,
}

#[derive(Debug, Clone)]
pub struct DeliveryReportCreate {
    pub project_id: String,
    pub title: String,
    pub markdown: String,
    pub sections_json: Value,
    pub is_final_candidate: bool,
}

#[derive(Debug, Clone)]
pub struct DeliveryReportUpdate {
    pub id: String,
    pub title: String,
    pub markdown: String,
    pub sections_json: Value,
    pub is_final_candidate: bool,
}

#[derive(Debug, Clone)]
pub struct QualityReviewCreate {
    pub project_id: String,
    pub summary: String,
    pub score: i64,
    pub checklist_json: Value,
    pub suggestions_json: Value,
}

#[derive(Debug, Clone)]
pub struct MemoryEntryCreate {
    pub scope: String,
    pub brand_id: Option<String>,
    pub project_id: Option<String>,
    pub content: String,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct MemoryCandidateCreate {
    pub scope: String,
    pub brand_id: Option<String>,
    pub project_id: Option<String>,
    pub content: String,
    pub source: String,
    pub source_entity_type: String,
    pub source_entity_id: String,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct MemoryStatusUpdate {
    pub id: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct AgentRunCreate {
    pub project_id: String,
    pub user_goal: String,
    pub status: String,
    pub runtime_kind: String,
    pub runtime_mode: String,
    pub runtime_version: String,
    pub roles_json: Value,
    pub plan_json: Value,
    pub result_summary: String,
}

#[derive(Debug, Clone)]
pub struct AgentRunEventCreate {
    pub agent_run_id: String,
    pub sequence_number: i64,
    pub role: String,
    pub event_type: String,
    pub message: String,
    pub payload_json: Value,
}

impl<'a> Repository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn create_brand(&self, input: BrandCreate) -> JoiResult<Brand> {
        validate_required_text("Brand name", &input.name)?;
        let now = Utc::now();
        let brand = Brand {
            id: new_id(),
            name: input.name.trim().to_string(),
            description: input.description,
            style_keywords: json!([]),
            visual_preferences: json!({}),
            negative_preferences: json!([]),
            common_scenes: json!([]),
            model_preferences: json!({}),
            platform_preferences: json!({}),
            created_at: now,
            updated_at: now,
        };
        self.connection.execute(
            "INSERT INTO brands (
                id, name, description, style_keywords_json, visual_preferences_json,
                negative_preferences_json, common_scenes_json, model_preferences_json,
                platform_preferences_json, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                brand.id,
                brand.name,
                brand.description,
                brand.style_keywords.to_string(),
                brand.visual_preferences.to_string(),
                brand.negative_preferences.to_string(),
                brand.common_scenes.to_string(),
                brand.model_preferences.to_string(),
                brand.platform_preferences.to_string(),
                brand.created_at.to_rfc3339(),
                brand.updated_at.to_rfc3339()
            ],
        )?;
        Ok(brand)
    }

    pub fn list_brands(&self) -> JoiResult<Vec<Brand>> {
        let mut statement = self.connection.prepare(
            "SELECT id, name, description, style_keywords_json, visual_preferences_json,
                    negative_preferences_json, common_scenes_json, model_preferences_json,
                    platform_preferences_json, created_at, updated_at
             FROM brands ORDER BY created_at ASC",
        )?;
        let rows = statement.query_map([], map_brand)?;
        collect_rows(rows)
    }

    pub fn get_brand(&self, id: &str) -> JoiResult<Brand> {
        self.connection
            .query_row(
                "SELECT id, name, description, style_keywords_json, visual_preferences_json,
                        negative_preferences_json, common_scenes_json, model_preferences_json,
                        platform_preferences_json, created_at, updated_at
                 FROM brands WHERE id = ?1",
                params![id],
                map_brand,
            )
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => JoiError::NotFound(format!("brand {}", id)),
                other => other.into(),
            })
    }

    pub fn update_brand(&self, input: BrandUpdate) -> JoiResult<Brand> {
        validate_required_text("Brand name", &input.name)?;
        let now = Utc::now();
        let affected = self.connection.execute(
            "UPDATE brands SET name = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
            params![
                input.name.trim(),
                input.description,
                now.to_rfc3339(),
                input.id
            ],
        )?;
        if affected == 0 {
            return Err(JoiError::NotFound(format!("brand {}", input.id)));
        }
        self.get_brand(&input.id)
    }

    pub fn create_project(&self, input: ProjectCreate) -> JoiResult<Project> {
        validate_required_text("Project title", &input.title)?;
        validate_non_negative("Project duration", input.duration_seconds)?;
        self.get_brand(&input.brand_id)?;
        let now = Utc::now();
        let project = Project {
            id: new_id(),
            brand_id: input.brand_id,
            title: input.title.trim().to_string(),
            advertising_goal: input.advertising_goal,
            duration_seconds: input.duration_seconds,
            target_platforms: json!([]),
            workflow_stage: "created".to_string(),
            current_version_id: None,
            final_version_id: None,
            created_at: now,
            updated_at: now,
        };
        self.connection.execute(
            "INSERT INTO projects (
                id, brand_id, title, advertising_goal, duration_seconds, target_platforms_json,
                workflow_stage, current_version_id, final_version_id, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                project.id,
                project.brand_id,
                project.title,
                project.advertising_goal,
                project.duration_seconds,
                project.target_platforms.to_string(),
                project.workflow_stage,
                project.current_version_id,
                project.final_version_id,
                project.created_at.to_rfc3339(),
                project.updated_at.to_rfc3339()
            ],
        )?;
        Ok(project)
    }

    pub fn list_projects(&self, brand_id: Option<&str>) -> JoiResult<Vec<Project>> {
        if let Some(brand_id) = brand_id {
            let mut statement = self.connection.prepare(
                "SELECT id, brand_id, title, advertising_goal, duration_seconds, target_platforms_json,
                        workflow_stage, current_version_id, final_version_id, created_at, updated_at
                 FROM projects WHERE brand_id = ?1 ORDER BY created_at ASC",
            )?;
            let rows = statement.query_map(params![brand_id], map_project)?;
            collect_rows(rows)
        } else {
            let mut statement = self.connection.prepare(
                "SELECT id, brand_id, title, advertising_goal, duration_seconds, target_platforms_json,
                        workflow_stage, current_version_id, final_version_id, created_at, updated_at
                 FROM projects ORDER BY created_at ASC",
            )?;
            let rows = statement.query_map([], map_project)?;
            collect_rows(rows)
        }
    }

    pub fn get_project(&self, id: &str) -> JoiResult<Project> {
        self.connection
            .query_row(
                "SELECT id, brand_id, title, advertising_goal, duration_seconds, target_platforms_json,
                        workflow_stage, current_version_id, final_version_id, created_at, updated_at
                 FROM projects WHERE id = ?1",
                params![id],
                map_project,
            )
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => {
                    JoiError::NotFound(format!("project {}", id))
                }
                other => other.into(),
            })
    }

    pub fn update_project(&self, input: ProjectUpdate) -> JoiResult<Project> {
        validate_required_text("Project title", &input.title)?;
        validate_non_negative("Project duration", input.duration_seconds)?;
        let now = Utc::now();
        let affected = self.connection.execute(
            "UPDATE projects
             SET title = ?1, advertising_goal = ?2, duration_seconds = ?3, updated_at = ?4
             WHERE id = ?5",
            params![
                input.title.trim(),
                input.advertising_goal,
                input.duration_seconds,
                now.to_rfc3339(),
                input.id
            ],
        )?;
        if affected == 0 {
            return Err(JoiError::NotFound(format!("project {}", input.id)));
        }
        self.get_project(&input.id)
    }

    pub fn update_project_title(&self, project_id: &str, title: &str) -> JoiResult<Project> {
        validate_required_text("Project title", title)?;
        let now = Utc::now();
        let affected = self.connection.execute(
            "UPDATE projects SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![title.trim(), now.to_rfc3339(), project_id],
        )?;
        if affected == 0 {
            return Err(JoiError::NotFound(format!("project {}", project_id)));
        }
        self.get_project(project_id)
    }

    pub fn create_project_version(&self, version: ProjectVersion) -> JoiResult<ProjectVersion> {
        self.connection.execute(
            "INSERT INTO project_versions (
                id, project_id, version_number, label, change_reason, changed_entities_json,
                snapshot_json, created_by, is_final_candidate, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                version.id,
                version.project_id,
                version.version_number,
                version.label,
                version.change_reason,
                version.changed_entities_json.to_string(),
                version.snapshot_json.to_string(),
                version.created_by,
                if version.is_final_candidate { 1 } else { 0 },
                version.created_at.to_rfc3339()
            ],
        )?;
        Ok(version)
    }

    pub fn list_project_versions(&self, project_id: &str) -> JoiResult<Vec<ProjectVersion>> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, version_number, label, change_reason, changed_entities_json,
                    snapshot_json, created_by, is_final_candidate, created_at
             FROM project_versions WHERE project_id = ?1 ORDER BY version_number ASC",
        )?;
        let rows = statement.query_map(params![project_id], map_project_version)?;
        collect_rows(rows)
    }

    pub fn get_project_version(&self, version_id: &str) -> JoiResult<ProjectVersion> {
        self.connection
            .query_row(
                "SELECT id, project_id, version_number, label, change_reason, changed_entities_json,
                        snapshot_json, created_by, is_final_candidate, created_at
                 FROM project_versions WHERE id = ?1",
                params![version_id],
                map_project_version,
            )
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => {
                    JoiError::NotFound(format!("project version {}", version_id))
                }
                other => other.into(),
            })
    }

    pub fn next_project_version_number(&self, project_id: &str) -> JoiResult<i64> {
        let version_number = self.connection.query_row(
            "SELECT COALESCE(MAX(version_number), 0) + 1 FROM project_versions WHERE project_id = ?1",
            params![project_id],
            |row| row.get(0),
        )?;
        Ok(version_number)
    }

    pub fn create_asset(&self, input: AssetCreate) -> JoiResult<Asset> {
        let kind = AssetKind::try_from(input.kind.as_str())?;
        self.get_project(&input.project_id)?;
        let now = Utc::now();
        let asset = Asset {
            id: new_id(),
            project_id: input.project_id,
            kind: kind.as_str().to_string(),
            display_name: input.display_name,
            relative_path: input.relative_path,
            source_uri: input.source_uri,
            mime_type: input.mime_type,
            file_size_bytes: input.file_size_bytes,
            sha256: input.sha256,
            metadata_json: json!({}),
            created_at: now,
            updated_at: now,
        };
        self.connection.execute(
            "INSERT INTO assets (
                id, project_id, kind, display_name, relative_path, source_uri, mime_type,
                file_size_bytes, sha256, metadata_json, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                asset.id,
                asset.project_id,
                asset.kind,
                asset.display_name,
                asset.relative_path,
                asset.source_uri,
                asset.mime_type,
                asset.file_size_bytes,
                asset.sha256,
                asset.metadata_json.to_string(),
                asset.created_at.to_rfc3339(),
                asset.updated_at.to_rfc3339()
            ],
        )?;
        Ok(asset)
    }

    pub fn create_research_report(&self, input: ResearchReportCreate) -> JoiResult<ResearchReport> {
        self.get_project(&input.project_id)?;
        let now = Utc::now();
        let report = ResearchReport {
            id: new_id(),
            project_id: input.project_id,
            summary: input.summary,
            findings_json: input.findings_json,
            sources_json: input.sources_json,
            created_at: now,
            updated_at: now,
        };
        self.connection.execute(
            "INSERT INTO research_reports (
                id, project_id, summary, findings_json, sources_json, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                report.id,
                report.project_id,
                report.summary,
                report.findings_json.to_string(),
                report.sources_json.to_string(),
                report.created_at.to_rfc3339(),
                report.updated_at.to_rfc3339()
            ],
        )?;
        Ok(report)
    }

    pub fn create_product_understanding(
        &self,
        input: ProductUnderstandingCreate,
    ) -> JoiResult<ProductUnderstanding> {
        self.get_project(&input.project_id)?;
        let now = Utc::now();
        let understanding = ProductUnderstanding {
            id: new_id(),
            project_id: input.project_id,
            product_name: input.product_name.trim().to_string(),
            category: input.category.trim().to_string(),
            audience: input.audience.trim().to_string(),
            selling_points_json: json!(input.selling_points),
            constraints_json: json!(input.constraints),
            notes: input.notes,
            created_at: now,
            updated_at: now,
        };
        self.connection.execute(
            "INSERT INTO product_understandings (
                id, project_id, product_name, category, audience, selling_points_json,
                constraints_json, notes, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                understanding.id,
                understanding.project_id,
                understanding.product_name,
                understanding.category,
                understanding.audience,
                understanding.selling_points_json.to_string(),
                understanding.constraints_json.to_string(),
                understanding.notes,
                understanding.created_at.to_rfc3339(),
                understanding.updated_at.to_rfc3339()
            ],
        )?;
        Ok(understanding)
    }

    pub fn create_creative_direction(
        &self,
        input: CreativeDirectionCreate,
    ) -> JoiResult<CreativeDirection> {
        validate_required_text("Creative direction title", &input.title)?;
        self.get_project(&input.project_id)?;
        let now = Utc::now();
        let direction = CreativeDirection {
            id: new_id(),
            project_id: input.project_id,
            title: input.title.trim().to_string(),
            concept: input.concept,
            tone: input.tone,
            visual_style: input.visual_style,
            scene_direction: input.scene_direction,
            rationale: input.rationale,
            created_at: now,
            updated_at: now,
        };
        self.connection.execute(
            "INSERT INTO creative_directions (
                id, project_id, title, concept, tone, visual_style, scene_direction, rationale,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                direction.id,
                direction.project_id,
                direction.title,
                direction.concept,
                direction.tone,
                direction.visual_style,
                direction.scene_direction,
                direction.rationale,
                direction.created_at.to_rfc3339(),
                direction.updated_at.to_rfc3339()
            ],
        )?;
        Ok(direction)
    }

    pub fn create_storyboard(&self, input: StoryboardCreate) -> JoiResult<Storyboard> {
        validate_non_negative("Storyboard duration", input.duration_seconds)?;
        self.get_project(&input.project_id)?;
        let now = Utc::now();
        let storyboard = Storyboard {
            id: new_id(),
            project_id: input.project_id,
            title: input.title,
            duration_seconds: input.duration_seconds,
            created_at: now,
            updated_at: now,
        };
        self.connection.execute(
            "INSERT INTO storyboards (
                id, project_id, title, duration_seconds, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                storyboard.id,
                storyboard.project_id,
                storyboard.title,
                storyboard.duration_seconds,
                storyboard.created_at.to_rfc3339(),
                storyboard.updated_at.to_rfc3339()
            ],
        )?;
        Ok(storyboard)
    }

    pub fn get_storyboard(&self, id: &str) -> JoiResult<Storyboard> {
        self.connection
            .query_row(
                "SELECT id, project_id, title, duration_seconds, created_at, updated_at
                 FROM storyboards WHERE id = ?1",
                params![id],
                map_storyboard,
            )
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => {
                    JoiError::NotFound(format!("storyboard {}", id))
                }
                other => other.into(),
            })
    }

    pub fn create_shot(&self, input: ShotCreate) -> JoiResult<Shot> {
        if input.shot_number <= 0 {
            return Err(JoiError::Validation(
                "Shot number must be positive".to_string(),
            ));
        }
        validate_non_negative("Shot duration", input.duration_seconds)?;
        let now = Utc::now();
        let shot = Shot {
            id: new_id(),
            storyboard_id: input.storyboard_id,
            shot_number: input.shot_number,
            duration_seconds: input.duration_seconds,
            description: input.description,
            model_action: String::new(),
            camera_movement: String::new(),
            scene: String::new(),
            lighting: String::new(),
            subtitle_or_voiceover: String::new(),
            rationale: String::new(),
            is_locked: false,
            metadata_json: json!({}),
            created_at: now,
            updated_at: now,
        };
        self.connection.execute(
            "INSERT INTO shots (
                id, storyboard_id, shot_number, duration_seconds, description, model_action,
                camera_movement, scene, lighting, subtitle_or_voiceover, rationale, is_locked,
                metadata_json, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                shot.id,
                shot.storyboard_id,
                shot.shot_number,
                shot.duration_seconds,
                shot.description,
                shot.model_action,
                shot.camera_movement,
                shot.scene,
                shot.lighting,
                shot.subtitle_or_voiceover,
                shot.rationale,
                0,
                shot.metadata_json.to_string(),
                shot.created_at.to_rfc3339(),
                shot.updated_at.to_rfc3339()
            ],
        )?;
        Ok(shot)
    }

    pub fn create_shot_plan(&self, input: ShotPlanCreate) -> JoiResult<Shot> {
        if input.shot_number <= 0 {
            return Err(JoiError::Validation(
                "Shot number must be positive".to_string(),
            ));
        }
        validate_positive("Shot duration", input.duration_seconds)?;
        validate_required_text("Shot visual description", &input.visual_description)?;
        validate_required_text("Shot model action", &input.model_action)?;
        validate_required_text("Shot garment focus", &input.garment_focus)?;
        validate_required_text("Shot camera movement", &input.camera_movement)?;
        validate_required_text("Shot scene", &input.scene)?;
        validate_required_text("Shot rationale", &input.rationale)?;
        self.get_storyboard(&input.storyboard_id)?;

        let now = Utc::now();
        let metadata_json = shot_plan_metadata(&input);
        let shot = Shot {
            id: new_id(),
            storyboard_id: input.storyboard_id,
            shot_number: input.shot_number,
            duration_seconds: input.duration_seconds,
            description: input.visual_description.trim().to_string(),
            model_action: input.model_action.trim().to_string(),
            camera_movement: input.camera_movement.trim().to_string(),
            scene: input.scene.trim().to_string(),
            lighting: input.lighting.trim().to_string(),
            subtitle_or_voiceover: input.subtitle_or_text.trim().to_string(),
            rationale: input.rationale.trim().to_string(),
            is_locked: false,
            metadata_json,
            created_at: now,
            updated_at: now,
        };
        self.connection.execute(
            "INSERT INTO shots (
                id, storyboard_id, shot_number, duration_seconds, description, model_action,
                camera_movement, scene, lighting, subtitle_or_voiceover, rationale, is_locked,
                metadata_json, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                shot.id,
                shot.storyboard_id,
                shot.shot_number,
                shot.duration_seconds,
                shot.description,
                shot.model_action,
                shot.camera_movement,
                shot.scene,
                shot.lighting,
                shot.subtitle_or_voiceover,
                shot.rationale,
                if shot.is_locked { 1 } else { 0 },
                shot.metadata_json.to_string(),
                shot.created_at.to_rfc3339(),
                shot.updated_at.to_rfc3339()
            ],
        )?;
        Ok(shot)
    }

    pub fn get_shot(&self, id: &str) -> JoiResult<Shot> {
        self.connection
            .query_row(
                "SELECT id, storyboard_id, shot_number, duration_seconds, description, model_action,
                        camera_movement, scene, lighting, subtitle_or_voiceover, rationale, is_locked,
                        metadata_json, created_at, updated_at
                 FROM shots WHERE id = ?1",
                params![id],
                map_shot,
            )
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => JoiError::NotFound(format!("shot {}", id)),
                other => other.into(),
            })
    }

    pub fn update_shot(&self, input: ShotUpdate) -> JoiResult<Shot> {
        validate_positive("Shot duration", input.duration_seconds)?;
        validate_required_text("Shot visual description", &input.visual_description)?;
        validate_required_text("Shot model action", &input.model_action)?;
        validate_required_text("Shot garment focus", &input.garment_focus)?;
        validate_required_text("Shot camera movement", &input.camera_movement)?;
        validate_required_text("Shot scene", &input.scene)?;
        validate_required_text("Shot rationale", &input.rationale)?;

        let existing = self.get_shot(&input.id)?;
        let metadata_json = update_shot_metadata(
            existing.metadata_json,
            input.garment_focus.trim(),
            input.transition.trim(),
        );
        let now = Utc::now();
        let affected = self.connection.execute(
            "UPDATE shots
             SET duration_seconds = ?1,
                 description = ?2,
                 model_action = ?3,
                 camera_movement = ?4,
                 scene = ?5,
                 lighting = ?6,
                 subtitle_or_voiceover = ?7,
                 rationale = ?8,
                 is_locked = ?9,
                 metadata_json = ?10,
                 updated_at = ?11
             WHERE id = ?12",
            params![
                input.duration_seconds,
                input.visual_description.trim(),
                input.model_action.trim(),
                input.camera_movement.trim(),
                input.scene.trim(),
                input.lighting.trim(),
                input.subtitle_or_text.trim(),
                input.rationale.trim(),
                if input.is_locked { 1 } else { 0 },
                metadata_json.to_string(),
                now.to_rfc3339(),
                input.id
            ],
        )?;
        if affected == 0 {
            return Err(JoiError::NotFound(format!("shot {}", input.id)));
        }
        self.get_shot(&input.id)
    }

    pub fn create_prompt_package(&self, input: PromptPackageCreate) -> JoiResult<PromptPackage> {
        let platform = PromptPlatform::try_from(input.platform.as_str())?;
        let modality = PromptModality::try_from(input.modality.as_str())?;
        validate_prompt_modality(platform, modality)?;
        validate_required_text("Prompt text", &input.prompt_text)?;
        self.get_project(&input.project_id)?;
        if input.shot_id.is_none() && modality == PromptModality::Video {
            return Err(JoiError::Validation(
                "Video prompt packages require a shot".to_string(),
            ));
        }
        if let Some(shot_id) = input.shot_id.as_deref() {
            self.get_shot(shot_id)?;
        }
        let now = Utc::now();
        let prompt = PromptPackage {
            id: new_id(),
            project_id: input.project_id,
            shot_id: input.shot_id,
            platform: platform.as_str().to_string(),
            modality: modality.as_str().to_string(),
            prompt_text: input.prompt_text.trim().to_string(),
            negative_prompt: input.negative_prompt.trim().to_string(),
            parameters_json: input.parameters_json,
            is_locked: false,
            created_at: now,
            updated_at: now,
        };
        self.connection.execute(
            "INSERT INTO prompt_packages (
                id, project_id, shot_id, platform, modality, prompt_text, negative_prompt,
                parameters_json, is_locked, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                prompt.id,
                prompt.project_id,
                prompt.shot_id,
                prompt.platform,
                prompt.modality,
                prompt.prompt_text,
                prompt.negative_prompt,
                prompt.parameters_json.to_string(),
                0,
                prompt.created_at.to_rfc3339(),
                prompt.updated_at.to_rfc3339()
            ],
        )?;
        Ok(prompt)
    }

    pub fn get_prompt_package(&self, id: &str) -> JoiResult<PromptPackage> {
        self.connection
            .query_row(
                "SELECT id, project_id, shot_id, platform, modality, prompt_text, negative_prompt,
                        parameters_json, is_locked, created_at, updated_at
                 FROM prompt_packages WHERE id = ?1",
                params![id],
                map_prompt_package,
            )
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => {
                    JoiError::NotFound(format!("prompt package {}", id))
                }
                other => other.into(),
            })
    }

    pub fn update_prompt_package(&self, input: PromptPackageUpdate) -> JoiResult<PromptPackage> {
        validate_required_text("Prompt text", &input.prompt_text)?;
        let now = Utc::now();
        let affected = self.connection.execute(
            "UPDATE prompt_packages
             SET prompt_text = ?1, negative_prompt = ?2, parameters_json = ?3,
                 is_locked = ?4, updated_at = ?5
             WHERE id = ?6",
            params![
                input.prompt_text.trim(),
                input.negative_prompt.trim(),
                input.parameters_json.to_string(),
                if input.is_locked { 1 } else { 0 },
                now.to_rfc3339(),
                input.id
            ],
        )?;
        if affected == 0 {
            return Err(JoiError::NotFound(format!("prompt package {}", input.id)));
        }
        self.get_prompt_package(&input.id)
    }

    pub fn create_delivery_report(&self, input: DeliveryReportCreate) -> JoiResult<DeliveryReport> {
        validate_required_text("Delivery report title", &input.title)?;
        validate_required_text("Delivery report markdown", &input.markdown)?;
        validate_delivery_report_sections(&input.sections_json)?;
        self.get_project(&input.project_id)?;

        let now = Utc::now();
        let report = DeliveryReport {
            id: new_id(),
            project_id: input.project_id,
            title: input.title.trim().to_string(),
            markdown: input.markdown.trim().to_string(),
            sections_json: input.sections_json,
            is_final_candidate: input.is_final_candidate,
            created_at: now,
            updated_at: now,
        };
        self.connection.execute(
            "INSERT INTO delivery_reports (
                id, project_id, title, markdown, sections_json, is_final_candidate,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                report.id,
                report.project_id,
                report.title,
                report.markdown,
                report.sections_json.to_string(),
                if report.is_final_candidate { 1 } else { 0 },
                report.created_at.to_rfc3339(),
                report.updated_at.to_rfc3339()
            ],
        )?;
        Ok(report)
    }

    pub fn get_delivery_report(&self, id: &str) -> JoiResult<DeliveryReport> {
        self.connection
            .query_row(
                "SELECT id, project_id, title, markdown, sections_json, is_final_candidate,
                        created_at, updated_at
                 FROM delivery_reports WHERE id = ?1",
                params![id],
                map_delivery_report,
            )
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => {
                    JoiError::NotFound(format!("delivery report {}", id))
                }
                other => other.into(),
            })
    }

    pub fn update_delivery_report(&self, input: DeliveryReportUpdate) -> JoiResult<DeliveryReport> {
        validate_required_text("Delivery report title", &input.title)?;
        validate_required_text("Delivery report markdown", &input.markdown)?;
        validate_delivery_report_sections(&input.sections_json)?;
        let now = Utc::now();
        let affected = self.connection.execute(
            "UPDATE delivery_reports
             SET title = ?1, markdown = ?2, sections_json = ?3,
                 is_final_candidate = ?4, updated_at = ?5
             WHERE id = ?6",
            params![
                input.title.trim(),
                input.markdown.trim(),
                input.sections_json.to_string(),
                if input.is_final_candidate { 1 } else { 0 },
                now.to_rfc3339(),
                input.id
            ],
        )?;
        if affected == 0 {
            return Err(JoiError::NotFound(format!("delivery report {}", input.id)));
        }
        self.get_delivery_report(&input.id)
    }

    pub fn create_quality_review(&self, input: QualityReviewCreate) -> JoiResult<QualityReview> {
        self.get_project(&input.project_id)?;
        validate_required_text("Quality review summary", &input.summary)?;
        if !(0..=100).contains(&input.score) {
            return Err(JoiError::Validation(
                "Quality review score must be between 0 and 100".to_string(),
            ));
        }

        let now = Utc::now();
        let review = QualityReview {
            id: new_id(),
            project_id: input.project_id,
            summary: input.summary.trim().to_string(),
            score: input.score,
            checklist_json: input.checklist_json,
            suggestions_json: input.suggestions_json,
            created_at: now,
            updated_at: now,
        };
        self.connection.execute(
            "INSERT INTO quality_reviews (
                id, project_id, summary, score, checklist_json, suggestions_json,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                review.id,
                review.project_id,
                review.summary,
                review.score,
                review.checklist_json.to_string(),
                review.suggestions_json.to_string(),
                review.created_at.to_rfc3339(),
                review.updated_at.to_rfc3339()
            ],
        )?;
        Ok(review)
    }

    pub fn get_quality_review(&self, id: &str) -> JoiResult<QualityReview> {
        self.connection
            .query_row(
                "SELECT id, project_id, summary, score, checklist_json, suggestions_json,
                        created_at, updated_at
                 FROM quality_reviews WHERE id = ?1",
                params![id],
                map_quality_review,
            )
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => {
                    JoiError::NotFound(format!("quality review {}", id))
                }
                other => other.into(),
            })
    }

    pub fn update_quality_review_suggestions(
        &self,
        id: &str,
        suggestions_json: Value,
    ) -> JoiResult<QualityReview> {
        let now = Utc::now();
        let affected = self.connection.execute(
            "UPDATE quality_reviews
             SET suggestions_json = ?1, updated_at = ?2
             WHERE id = ?3",
            params![suggestions_json.to_string(), now.to_rfc3339(), id],
        )?;
        if affected == 0 {
            return Err(JoiError::NotFound(format!("quality review {}", id)));
        }
        self.get_quality_review(id)
    }

    pub fn create_memory_entry(&self, input: MemoryEntryCreate) -> JoiResult<MemoryEntry> {
        let scope = MemoryScope::try_from(input.scope.as_str())?;
        validate_required_text("Memory content", &input.content)?;
        self.validate_memory_target(
            scope,
            input.brand_id.as_deref(),
            input.project_id.as_deref(),
        )?;

        let now = Utc::now();
        let memory = MemoryEntry {
            id: new_id(),
            scope: scope.as_str().to_string(),
            brand_id: input.brand_id,
            project_id: input.project_id,
            content: input.content.trim().to_string(),
            source: input.source.trim().to_string(),
            source_entity_type: String::new(),
            source_entity_id: String::new(),
            confidence: 0.0,
            status: MemoryStatus::Proposed.as_str().to_string(),
            created_at: now,
            updated_at: now,
        };
        self.connection.execute(
            "INSERT INTO memory_entries (
                id, scope, brand_id, project_id, content, source, source_entity_type,
                source_entity_id, confidence, status, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                memory.id,
                memory.scope,
                memory.brand_id,
                memory.project_id,
                memory.content,
                memory.source,
                memory.source_entity_type,
                memory.source_entity_id,
                memory.confidence,
                memory.status,
                memory.created_at.to_rfc3339(),
                memory.updated_at.to_rfc3339()
            ],
        )?;
        Ok(memory)
    }

    pub fn create_memory_candidate(&self, input: MemoryCandidateCreate) -> JoiResult<MemoryEntry> {
        let scope = MemoryScope::try_from(input.scope.as_str())?;
        validate_required_text("Memory content", &input.content)?;
        if !(0.0..=1.0).contains(&input.confidence) {
            return Err(JoiError::Validation(
                "Memory confidence must be between 0.0 and 1.0".to_string(),
            ));
        }
        self.validate_memory_target(
            scope,
            input.brand_id.as_deref(),
            input.project_id.as_deref(),
        )?;

        let now = Utc::now();
        let memory = MemoryEntry {
            id: new_id(),
            scope: scope.as_str().to_string(),
            brand_id: input.brand_id,
            project_id: input.project_id,
            content: input.content.trim().to_string(),
            source: input.source.trim().to_string(),
            source_entity_type: input.source_entity_type.trim().to_string(),
            source_entity_id: input.source_entity_id.trim().to_string(),
            confidence: input.confidence,
            status: MemoryStatus::Proposed.as_str().to_string(),
            created_at: now,
            updated_at: now,
        };
        self.connection.execute(
            "INSERT INTO memory_entries (
                id, scope, brand_id, project_id, content, source, source_entity_type,
                source_entity_id, confidence, status, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                memory.id,
                memory.scope,
                memory.brand_id,
                memory.project_id,
                memory.content,
                memory.source,
                memory.source_entity_type,
                memory.source_entity_id,
                memory.confidence,
                memory.status,
                memory.created_at.to_rfc3339(),
                memory.updated_at.to_rfc3339()
            ],
        )?;
        Ok(memory)
    }

    pub fn get_memory_entry(&self, id: &str) -> JoiResult<MemoryEntry> {
        self.connection
            .query_row(
                "SELECT id, scope, brand_id, project_id, content, source, source_entity_type,
                        source_entity_id, confidence, status, created_at, updated_at
                 FROM memory_entries WHERE id = ?1",
                params![id],
                map_memory_entry,
            )
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => {
                    JoiError::NotFound(format!("memory {}", id))
                }
                other => other.into(),
            })
    }

    pub fn update_memory_entry_status(&self, input: MemoryStatusUpdate) -> JoiResult<MemoryEntry> {
        let status = MemoryStatus::try_from(input.status.as_str())?;
        let now = Utc::now();
        let affected = self.connection.execute(
            "UPDATE memory_entries SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.as_str(), now.to_rfc3339(), input.id],
        )?;
        if affected == 0 {
            return Err(JoiError::NotFound(format!("memory {}", input.id)));
        }
        self.get_memory_entry(&input.id)
    }

    fn validate_memory_target(
        &self,
        scope: MemoryScope,
        brand_id: Option<&str>,
        project_id: Option<&str>,
    ) -> JoiResult<()> {
        match scope {
            MemoryScope::User => {
                if brand_id.is_some() || project_id.is_some() {
                    return Err(JoiError::Validation(
                        "user memory must not include brand_id or project_id".to_string(),
                    ));
                }
            }
            MemoryScope::Brand => {
                if project_id.is_some() {
                    return Err(JoiError::Validation(
                        "brand memory must not include project_id".to_string(),
                    ));
                }
                let brand_id = brand_id.ok_or_else(|| {
                    JoiError::Validation("brand memory requires brand_id".to_string())
                })?;
                self.get_brand(brand_id)?;
            }
            MemoryScope::Project => {
                let project_id = project_id.ok_or_else(|| {
                    JoiError::Validation("project memory requires project_id".to_string())
                })?;
                let project = self.get_project(project_id)?;
                if let Some(brand_id) = brand_id {
                    if brand_id != project.brand_id {
                        return Err(JoiError::Validation(
                            "project memory brand_id must match project brand".to_string(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn create_agent_run(&self, input: AgentRunCreate) -> JoiResult<AgentRun> {
        validate_required_text("Agent goal", &input.user_goal)?;
        self.get_project(&input.project_id)?;
        let now = Utc::now();
        let run = AgentRun {
            id: new_id(),
            project_id: input.project_id,
            user_goal: input.user_goal.trim().to_string(),
            status: input.status,
            runtime_kind: input.runtime_kind,
            runtime_mode: input.runtime_mode,
            runtime_version: input.runtime_version,
            roles_json: input.roles_json,
            plan_json: input.plan_json,
            result_summary: input.result_summary,
            created_at: now,
            updated_at: now,
        };
        self.connection.execute(
            "INSERT INTO agent_runs (
                id, project_id, user_goal, status, runtime_kind, runtime_mode, runtime_version,
                roles_json, plan_json, result_summary, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                run.id,
                run.project_id,
                run.user_goal,
                run.status,
                run.runtime_kind,
                run.runtime_mode,
                run.runtime_version,
                run.roles_json.to_string(),
                run.plan_json.to_string(),
                run.result_summary,
                run.created_at.to_rfc3339(),
                run.updated_at.to_rfc3339()
            ],
        )?;
        Ok(run)
    }

    pub fn get_agent_run(&self, id: &str) -> JoiResult<AgentRun> {
        self.connection
            .query_row(
                "SELECT id, project_id, user_goal, status, runtime_kind, runtime_mode,
                        runtime_version, roles_json, plan_json, result_summary, created_at, updated_at
                 FROM agent_runs WHERE id = ?1",
                params![id],
                map_agent_run,
            )
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => {
                    JoiError::NotFound(format!("agent run {}", id))
                }
                other => other.into(),
            })
    }

    pub fn list_agent_runs(&self, project_id: &str) -> JoiResult<Vec<AgentRun>> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, user_goal, status, runtime_kind, runtime_mode,
                    runtime_version, roles_json, plan_json, result_summary, created_at, updated_at
             FROM agent_runs WHERE project_id = ?1 ORDER BY created_at DESC, id DESC",
        )?;
        let rows = statement.query_map(params![project_id], map_agent_run)?;
        collect_rows(rows)
    }

    pub fn create_agent_run_event(&self, input: AgentRunEventCreate) -> JoiResult<AgentRunEvent> {
        if input.sequence_number <= 0 {
            return Err(JoiError::Validation(
                "Agent event sequence number must be positive".to_string(),
            ));
        }
        self.get_agent_run(&input.agent_run_id)?;
        let now = Utc::now();
        let event = AgentRunEvent {
            id: new_id(),
            agent_run_id: input.agent_run_id,
            sequence_number: input.sequence_number,
            role: input.role,
            event_type: input.event_type,
            message: input.message,
            payload_json: input.payload_json,
            created_at: now,
        };
        self.connection.execute(
            "INSERT INTO agent_run_events (
                id, agent_run_id, sequence_number, role, event_type, message, payload_json, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                event.id,
                event.agent_run_id,
                event.sequence_number,
                event.role,
                event.event_type,
                event.message,
                event.payload_json.to_string(),
                event.created_at.to_rfc3339()
            ],
        )?;
        Ok(event)
    }

    pub fn list_agent_run_events(&self, agent_run_id: &str) -> JoiResult<Vec<AgentRunEvent>> {
        let mut statement = self.connection.prepare(
            "SELECT id, agent_run_id, sequence_number, role, event_type, message, payload_json, created_at
             FROM agent_run_events WHERE agent_run_id = ?1 ORDER BY sequence_number ASC",
        )?;
        let rows = statement.query_map(params![agent_run_id], map_agent_run_event)?;
        collect_rows(rows)
    }

    pub fn list_storyboards(&self, project_id: &str) -> JoiResult<Vec<Storyboard>> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, title, duration_seconds, created_at, updated_at
             FROM storyboards WHERE project_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = statement.query_map(params![project_id], map_storyboard)?;
        collect_rows(rows)
    }

    pub fn list_storyboards_with_typed_shots(
        &self,
        project_id: &str,
    ) -> JoiResult<Vec<StoryboardWithShots>> {
        let storyboards = self.list_storyboards(project_id)?;
        let mut values = Vec::with_capacity(storyboards.len());
        for storyboard in storyboards {
            let shots = self.list_shots(&storyboard.id)?;
            values.push(StoryboardWithShots { storyboard, shots });
        }
        Ok(values)
    }

    pub fn list_shots(&self, storyboard_id: &str) -> JoiResult<Vec<Shot>> {
        let mut statement = self.connection.prepare(
            "SELECT id, storyboard_id, shot_number, duration_seconds, description, model_action,
                    camera_movement, scene, lighting, subtitle_or_voiceover, rationale, is_locked,
                    metadata_json, created_at, updated_at
             FROM shots WHERE storyboard_id = ?1 ORDER BY shot_number ASC",
        )?;
        let rows = statement.query_map(params![storyboard_id], map_shot)?;
        collect_rows(rows)
    }

    pub fn list_prompt_packages(&self, project_id: &str) -> JoiResult<Vec<PromptPackage>> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, shot_id, platform, modality, prompt_text, negative_prompt,
                    parameters_json, is_locked, created_at, updated_at
             FROM prompt_packages WHERE project_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = statement.query_map(params![project_id], map_prompt_package)?;
        collect_rows(rows)
    }

    pub fn list_delivery_reports(&self, project_id: &str) -> JoiResult<Vec<DeliveryReport>> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, title, markdown, sections_json, is_final_candidate,
                    created_at, updated_at
             FROM delivery_reports WHERE project_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = statement.query_map(params![project_id], map_delivery_report)?;
        collect_rows(rows)
    }

    pub fn list_quality_reviews(&self, project_id: &str) -> JoiResult<Vec<QualityReview>> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, summary, score, checklist_json, suggestions_json,
                    created_at, updated_at
             FROM quality_reviews WHERE project_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = statement.query_map(params![project_id], map_quality_review)?;
        collect_rows(rows)
    }

    pub fn list_assets(&self, project_id: &str) -> JoiResult<Vec<Asset>> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, kind, display_name, relative_path, source_uri, mime_type,
                    file_size_bytes, sha256, metadata_json, created_at, updated_at
             FROM assets WHERE project_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = statement.query_map(params![project_id], map_asset)?;
        collect_rows(rows)
    }

    pub fn list_research_reports(&self, project_id: &str) -> JoiResult<Vec<ResearchReport>> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, summary, findings_json, sources_json, created_at, updated_at
             FROM research_reports WHERE project_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = statement.query_map(params![project_id], map_research_report)?;
        collect_rows(rows)
    }

    pub fn list_product_understandings(
        &self,
        project_id: &str,
    ) -> JoiResult<Vec<ProductUnderstanding>> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, product_name, category, audience, selling_points_json,
                    constraints_json, notes, created_at, updated_at
             FROM product_understandings WHERE project_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = statement.query_map(params![project_id], map_product_understanding)?;
        collect_rows(rows)
    }

    pub fn list_creative_directions(&self, project_id: &str) -> JoiResult<Vec<CreativeDirection>> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, title, concept, tone, visual_style, scene_direction, rationale,
                    created_at, updated_at
             FROM creative_directions WHERE project_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = statement.query_map(params![project_id], map_creative_direction)?;
        collect_rows(rows)
    }

    pub fn list_storyboards_with_shots(&self, project_id: &str) -> JoiResult<serde_json::Value> {
        let storyboards = self.list_storyboards(project_id)?;
        let mut values = Vec::with_capacity(storyboards.len());
        for storyboard in storyboards {
            let shots = self.list_shots(&storyboard.id)?;
            values.push(json!({
                "storyboard": storyboard,
                "shots": shots,
            }));
        }
        Ok(json!(values))
    }

    pub fn list_memory_entries(
        &self,
        scope: &str,
        brand_id: Option<&str>,
        project_id: Option<&str>,
    ) -> JoiResult<Vec<MemoryEntry>> {
        let scope = MemoryScope::try_from(scope)?;
        let mut statement = self.connection.prepare(
            "SELECT id, scope, brand_id, project_id, content, source, source_entity_type,
                    source_entity_id, confidence, status, created_at, updated_at
             FROM memory_entries
             WHERE scope = ?1
               AND (?2 IS NULL OR brand_id = ?2)
               AND (?3 IS NULL OR project_id = ?3)
             ORDER BY created_at ASC, id ASC",
        )?;
        let rows = statement.query_map(
            params![scope.as_str(), brand_id, project_id],
            map_memory_entry,
        )?;
        collect_rows(rows)
    }

    pub fn list_memory_entries_for_project(&self, project_id: &str) -> JoiResult<Vec<MemoryEntry>> {
        self.list_memory_entries(MemoryScope::Project.as_str(), None, Some(project_id))
    }
}

fn collect_rows<T>(rows: impl Iterator<Item = rusqlite::Result<T>>) -> JoiResult<Vec<T>> {
    let mut values = Vec::new();
    for row in rows {
        values.push(row?);
    }
    Ok(values)
}

fn validate_positive(label: &str, value: i64) -> JoiResult<()> {
    if value <= 0 {
        return Err(JoiError::Validation(format!("{label} must be positive")));
    }
    Ok(())
}

fn shot_plan_metadata(input: &ShotPlanCreate) -> Value {
    json!({
        "format_version": "joi.shot_metadata.v1",
        "garment_focus": input.garment_focus.trim(),
        "transition": input.transition.trim(),
        "source_memory_ids": &input.source_memory_ids,
        "source_research_report_ids": &input.source_research_report_ids,
        "generation_context": &input.generation_context,
    })
}

fn update_shot_metadata(existing: Value, garment_focus: &str, transition: &str) -> Value {
    let mut metadata = match existing {
        Value::Object(map) => Value::Object(map),
        _ => json!({}),
    };
    let object = metadata
        .as_object_mut()
        .expect("object value created from map or empty object");
    object.insert("format_version".to_string(), json!("joi.shot_metadata.v1"));
    object.insert("garment_focus".to_string(), json!(garment_focus));
    object.insert("transition".to_string(), json!(transition));
    object
        .entry("source_memory_ids".to_string())
        .or_insert_with(|| json!([]));
    object
        .entry("source_research_report_ids".to_string())
        .or_insert_with(|| json!([]));
    object
        .entry("generation_context".to_string())
        .or_insert_with(|| json!({"stage": "0.16", "source": "user_edit"}));
    metadata
}

fn validate_delivery_report_sections(value: &Value) -> JoiResult<()> {
    if value.get("format_version").and_then(Value::as_str)
        != Some("joi.delivery_report_sections.v1")
    {
        return Err(JoiError::Validation(
            "Delivery report sections must use format_version joi.delivery_report_sections.v1"
                .to_string(),
        ));
    }
    if !value.get("sections").is_some_and(Value::is_array) {
        return Err(JoiError::Validation(
            "Delivery report sections must include a sections array".to_string(),
        ));
    }
    Ok(())
}

fn parse_time(value: String, column_index: usize) -> rusqlite::Result<chrono::DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(&value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(
                column_index,
                rusqlite::types::Type::Text,
                Box::new(err),
            )
        })
}

fn parse_json(value: String, column_index: usize) -> rusqlite::Result<serde_json::Value> {
    serde_json::from_str(&value).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(
            column_index,
            rusqlite::types::Type::Text,
            Box::new(err),
        )
    })
}

fn parse_bool(value: i64, column_index: usize) -> rusqlite::Result<bool> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(rusqlite::Error::FromSqlConversionFailure(
            column_index,
            rusqlite::types::Type::Integer,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid boolean value: {}", other),
            )),
        )),
    }
}

fn map_brand(row: &rusqlite::Row<'_>) -> rusqlite::Result<Brand> {
    Ok(Brand {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        style_keywords: parse_json(row.get(3)?, 3)?,
        visual_preferences: parse_json(row.get(4)?, 4)?,
        negative_preferences: parse_json(row.get(5)?, 5)?,
        common_scenes: parse_json(row.get(6)?, 6)?,
        model_preferences: parse_json(row.get(7)?, 7)?,
        platform_preferences: parse_json(row.get(8)?, 8)?,
        created_at: parse_time(row.get(9)?, 9)?,
        updated_at: parse_time(row.get(10)?, 10)?,
    })
}

fn map_project(row: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get(0)?,
        brand_id: row.get(1)?,
        title: row.get(2)?,
        advertising_goal: row.get(3)?,
        duration_seconds: row.get(4)?,
        target_platforms: parse_json(row.get(5)?, 5)?,
        workflow_stage: row.get(6)?,
        current_version_id: row.get(7)?,
        final_version_id: row.get(8)?,
        created_at: parse_time(row.get(9)?, 9)?,
        updated_at: parse_time(row.get(10)?, 10)?,
    })
}

fn map_asset(row: &rusqlite::Row<'_>) -> rusqlite::Result<Asset> {
    Ok(Asset {
        id: row.get(0)?,
        project_id: row.get(1)?,
        kind: row.get(2)?,
        display_name: row.get(3)?,
        relative_path: row.get(4)?,
        source_uri: row.get(5)?,
        mime_type: row.get(6)?,
        file_size_bytes: row.get(7)?,
        sha256: row.get(8)?,
        metadata_json: parse_json(row.get(9)?, 9)?,
        created_at: parse_time(row.get(10)?, 10)?,
        updated_at: parse_time(row.get(11)?, 11)?,
    })
}

fn map_storyboard(row: &rusqlite::Row<'_>) -> rusqlite::Result<Storyboard> {
    Ok(Storyboard {
        id: row.get(0)?,
        project_id: row.get(1)?,
        title: row.get(2)?,
        duration_seconds: row.get(3)?,
        created_at: parse_time(row.get(4)?, 4)?,
        updated_at: parse_time(row.get(5)?, 5)?,
    })
}

fn map_shot(row: &rusqlite::Row<'_>) -> rusqlite::Result<Shot> {
    Ok(Shot {
        id: row.get(0)?,
        storyboard_id: row.get(1)?,
        shot_number: row.get(2)?,
        duration_seconds: row.get(3)?,
        description: row.get(4)?,
        model_action: row.get(5)?,
        camera_movement: row.get(6)?,
        scene: row.get(7)?,
        lighting: row.get(8)?,
        subtitle_or_voiceover: row.get(9)?,
        rationale: row.get(10)?,
        is_locked: parse_bool(row.get(11)?, 11)?,
        metadata_json: parse_json(row.get(12)?, 12)?,
        created_at: parse_time(row.get(13)?, 13)?,
        updated_at: parse_time(row.get(14)?, 14)?,
    })
}

fn map_prompt_package(row: &rusqlite::Row<'_>) -> rusqlite::Result<PromptPackage> {
    Ok(PromptPackage {
        id: row.get(0)?,
        project_id: row.get(1)?,
        shot_id: row.get(2)?,
        platform: row.get(3)?,
        modality: row.get(4)?,
        prompt_text: row.get(5)?,
        negative_prompt: row.get(6)?,
        parameters_json: parse_json(row.get(7)?, 7)?,
        is_locked: parse_bool(row.get(8)?, 8)?,
        created_at: parse_time(row.get(9)?, 9)?,
        updated_at: parse_time(row.get(10)?, 10)?,
    })
}

fn map_delivery_report(row: &rusqlite::Row<'_>) -> rusqlite::Result<DeliveryReport> {
    Ok(DeliveryReport {
        id: row.get(0)?,
        project_id: row.get(1)?,
        title: row.get(2)?,
        markdown: row.get(3)?,
        sections_json: parse_json(row.get(4)?, 4)?,
        is_final_candidate: parse_bool(row.get(5)?, 5)?,
        created_at: parse_time(row.get(6)?, 6)?,
        updated_at: parse_time(row.get(7)?, 7)?,
    })
}

fn map_quality_review(row: &rusqlite::Row<'_>) -> rusqlite::Result<QualityReview> {
    Ok(QualityReview {
        id: row.get(0)?,
        project_id: row.get(1)?,
        summary: row.get(2)?,
        score: row.get(3)?,
        checklist_json: parse_json(row.get(4)?, 4)?,
        suggestions_json: parse_json(row.get(5)?, 5)?,
        created_at: parse_time(row.get(6)?, 6)?,
        updated_at: parse_time(row.get(7)?, 7)?,
    })
}

fn map_research_report(row: &rusqlite::Row<'_>) -> rusqlite::Result<ResearchReport> {
    Ok(ResearchReport {
        id: row.get(0)?,
        project_id: row.get(1)?,
        summary: row.get(2)?,
        findings_json: parse_json(row.get(3)?, 3)?,
        sources_json: parse_json(row.get(4)?, 4)?,
        created_at: parse_time(row.get(5)?, 5)?,
        updated_at: parse_time(row.get(6)?, 6)?,
    })
}

fn map_product_understanding(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProductUnderstanding> {
    Ok(ProductUnderstanding {
        id: row.get(0)?,
        project_id: row.get(1)?,
        product_name: row.get(2)?,
        category: row.get(3)?,
        audience: row.get(4)?,
        selling_points_json: parse_json(row.get(5)?, 5)?,
        constraints_json: parse_json(row.get(6)?, 6)?,
        notes: row.get(7)?,
        created_at: parse_time(row.get(8)?, 8)?,
        updated_at: parse_time(row.get(9)?, 9)?,
    })
}

fn map_creative_direction(row: &rusqlite::Row<'_>) -> rusqlite::Result<CreativeDirection> {
    Ok(CreativeDirection {
        id: row.get(0)?,
        project_id: row.get(1)?,
        title: row.get(2)?,
        concept: row.get(3)?,
        tone: row.get(4)?,
        visual_style: row.get(5)?,
        scene_direction: row.get(6)?,
        rationale: row.get(7)?,
        created_at: parse_time(row.get(8)?, 8)?,
        updated_at: parse_time(row.get(9)?, 9)?,
    })
}

fn map_project_version(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProjectVersion> {
    Ok(ProjectVersion {
        id: row.get(0)?,
        project_id: row.get(1)?,
        version_number: row.get(2)?,
        label: row.get(3)?,
        change_reason: row.get(4)?,
        changed_entities_json: parse_json(row.get(5)?, 5)?,
        snapshot_json: parse_json(row.get(6)?, 6)?,
        created_by: row.get(7)?,
        is_final_candidate: parse_bool(row.get(8)?, 8)?,
        created_at: parse_time(row.get(9)?, 9)?,
    })
}

fn map_memory_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<MemoryEntry> {
    Ok(MemoryEntry {
        id: row.get(0)?,
        scope: row.get(1)?,
        brand_id: row.get(2)?,
        project_id: row.get(3)?,
        content: row.get(4)?,
        source: row.get(5)?,
        source_entity_type: row.get(6)?,
        source_entity_id: row.get(7)?,
        confidence: row.get(8)?,
        status: row.get(9)?,
        created_at: parse_time(row.get(10)?, 10)?,
        updated_at: parse_time(row.get(11)?, 11)?,
    })
}

fn map_agent_run(row: &rusqlite::Row<'_>) -> rusqlite::Result<AgentRun> {
    Ok(AgentRun {
        id: row.get(0)?,
        project_id: row.get(1)?,
        user_goal: row.get(2)?,
        status: row.get(3)?,
        runtime_kind: row.get(4)?,
        runtime_mode: row.get(5)?,
        runtime_version: row.get(6)?,
        roles_json: parse_json(row.get(7)?, 7)?,
        plan_json: parse_json(row.get(8)?, 8)?,
        result_summary: row.get(9)?,
        created_at: parse_time(row.get(10)?, 10)?,
        updated_at: parse_time(row.get(11)?, 11)?,
    })
}

fn map_agent_run_event(row: &rusqlite::Row<'_>) -> rusqlite::Result<AgentRunEvent> {
    Ok(AgentRunEvent {
        id: row.get(0)?,
        agent_run_id: row.get(1)?,
        sequence_number: row.get(2)?,
        role: row.get(3)?,
        event_type: row.get(4)?,
        message: row.get(5)?,
        payload_json: parse_json(row.get(6)?, 6)?,
        created_at: parse_time(row.get(7)?, 7)?,
    })
}
