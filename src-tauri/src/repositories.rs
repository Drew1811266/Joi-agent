use chrono::Utc;
use rusqlite::{params, Connection};
use serde_json::json;

use crate::error::{JoiError, JoiResult};
use crate::models::{
    new_id, Brand, CreativeDirection, ProductUnderstanding, Project, PromptModality, PromptPackage,
    PromptPlatform, ResearchReport, Shot, Storyboard,
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
pub struct ProjectCreate {
    pub brand_id: String,
    pub title: String,
    pub advertising_goal: String,
    pub duration_seconds: i64,
}

#[derive(Debug, Clone)]
pub struct ResearchReportCreate {
    pub project_id: String,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct ProductUnderstandingCreate {
    pub project_id: String,
    pub product_name: String,
    pub category: String,
}

#[derive(Debug, Clone)]
pub struct CreativeDirectionCreate {
    pub project_id: String,
    pub title: String,
    pub concept: String,
}

#[derive(Debug, Clone)]
pub struct StoryboardCreate {
    pub project_id: String,
    pub title: String,
    pub duration_seconds: i64,
}

#[derive(Debug, Clone)]
pub struct ShotCreate {
    pub storyboard_id: String,
    pub shot_number: i64,
    pub duration_seconds: i64,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct PromptPackageCreate {
    pub project_id: String,
    pub shot_id: String,
    pub platform: String,
    pub modality: String,
    pub prompt_text: String,
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

    pub fn create_research_report(&self, input: ResearchReportCreate) -> JoiResult<ResearchReport> {
        self.get_project(&input.project_id)?;
        let now = Utc::now();
        let report = ResearchReport {
            id: new_id(),
            project_id: input.project_id,
            summary: input.summary,
            findings_json: json!([]),
            sources_json: json!([]),
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
            product_name: input.product_name,
            category: input.category,
            audience: String::new(),
            selling_points_json: json!([]),
            constraints_json: json!([]),
            notes: String::new(),
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
            tone: String::new(),
            visual_style: String::new(),
            scene_direction: String::new(),
            rationale: String::new(),
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

    pub fn create_shot(&self, input: ShotCreate) -> JoiResult<Shot> {
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

    pub fn create_prompt_package(&self, input: PromptPackageCreate) -> JoiResult<PromptPackage> {
        let platform = PromptPlatform::try_from(input.platform.as_str())?;
        let modality = PromptModality::try_from(input.modality.as_str())?;
        validate_prompt_modality(platform, modality)?;
        self.get_project(&input.project_id)?;
        let now = Utc::now();
        let prompt = PromptPackage {
            id: new_id(),
            project_id: input.project_id,
            shot_id: input.shot_id,
            platform: platform.as_str().to_string(),
            modality: modality.as_str().to_string(),
            prompt_text: input.prompt_text,
            negative_prompt: String::new(),
            parameters_json: json!({}),
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

    pub fn list_storyboards(&self, project_id: &str) -> JoiResult<Vec<Storyboard>> {
        let mut statement = self.connection.prepare(
            "SELECT id, project_id, title, duration_seconds, created_at, updated_at
             FROM storyboards WHERE project_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = statement.query_map(params![project_id], map_storyboard)?;
        collect_rows(rows)
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
}

fn collect_rows<T>(rows: impl Iterator<Item = rusqlite::Result<T>>) -> JoiResult<Vec<T>> {
    let mut values = Vec::new();
    for row in rows {
        values.push(row?);
    }
    Ok(values)
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
        is_locked: row.get::<_, i64>(11)? == 1,
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
        is_locked: row.get::<_, i64>(8)? == 1,
        created_at: parse_time(row.get(9)?, 9)?,
        updated_at: parse_time(row.get(10)?, 10)?,
    })
}
