use chrono::Utc;
use rusqlite::Connection;
use serde_json::{json, Value};

use crate::error::{JoiError, JoiResult};
use crate::models::{new_id, ProjectVersion};
use crate::repositories::Repository;

#[derive(Debug, Clone)]
pub struct SaveSnapshotInput {
    pub project_id: String,
    pub label: String,
    pub change_reason: String,
    pub changed_entities: Vec<String>,
    pub created_by: String,
    pub is_final_candidate: bool,
}

pub struct ProjectSnapshotService<'a> {
    connection: &'a Connection,
}

impl<'a> ProjectSnapshotService<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn save_snapshot(&self, input: SaveSnapshotInput) -> JoiResult<ProjectVersion> {
        let repo = Repository::new(self.connection);
        let snapshot = self.build_snapshot(&input.project_id)?;
        let version = ProjectVersion {
            id: new_id(),
            project_id: input.project_id.clone(),
            version_number: repo.next_project_version_number(&input.project_id)?,
            label: input.label,
            change_reason: input.change_reason,
            changed_entities_json: json!(input.changed_entities),
            snapshot_json: snapshot,
            created_by: input.created_by,
            is_final_candidate: input.is_final_candidate,
            created_at: Utc::now(),
        };
        repo.create_project_version(version)
    }

    pub fn build_snapshot(&self, project_id: &str) -> JoiResult<Value> {
        let repo = Repository::new(self.connection);
        let project = repo.get_project(project_id)?;
        let brand = repo.get_brand(&project.brand_id)?;
        Ok(json!({
            "format_version": 1,
            "brand": brand,
            "project": project,
            "assets": repo.list_assets(project_id)?,
            "research_reports": repo.list_research_reports(project_id)?,
            "product_understandings": repo.list_product_understandings(project_id)?,
            "creative_directions": repo.list_creative_directions(project_id)?,
            "storyboards": repo.list_storyboards_with_shots(project_id)?,
            "prompt_packages": repo.list_prompt_packages(project_id)?,
            "memory_entries": repo.list_memory_entries_for_project(project_id)?,
        }))
    }

    pub fn restore_project_version(&self, project_id: &str, version_id: &str) -> JoiResult<()> {
        let repo = Repository::new(self.connection);
        let version = repo.get_project_version(version_id)?;
        if version.project_id != project_id {
            return Err(JoiError::Validation(
                "version does not belong to project".to_string(),
            ));
        }
        let title = project_title_from_snapshot(&version.snapshot_json)?;
        repo.update_project_title(project_id, &title)?;
        Ok(())
    }
}

fn project_title_from_snapshot(snapshot: &Value) -> JoiResult<String> {
    if snapshot.get("format_version").and_then(Value::as_i64) != Some(1) {
        return Err(JoiError::Validation(
            "snapshot version malformed: expected format_version 1".to_string(),
        ));
    }

    let title = snapshot
        .get("project")
        .and_then(|project| project.get("title"))
        .and_then(Value::as_str)
        .ok_or_else(|| {
            JoiError::Validation(
                "snapshot version malformed: project.title must be a string".to_string(),
            )
        })?;

    if title.trim().is_empty() {
        return Err(JoiError::Validation(
            "snapshot version malformed: project.title must not be empty".to_string(),
        ));
    }

    Ok(title.to_string())
}
