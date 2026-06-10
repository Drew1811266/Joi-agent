use chrono::Utc;
use rusqlite::{params, Connection};
use serde_json::json;

use crate::error::{JoiError, JoiResult};
use crate::models::{new_id, Brand, Project};
use crate::validation::{validate_non_negative, validate_required_text};

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
