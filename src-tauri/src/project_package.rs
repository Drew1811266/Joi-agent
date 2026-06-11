use std::path::{Path, PathBuf};

use rusqlite::Connection;
use serde_json::{json, Value};

use crate::assets::safe_join_asset_path;
use crate::error::{JoiError, JoiResult};
use crate::repositories::{BrandCreate, ProjectCreate, Repository};
use crate::snapshots::ProjectSnapshotService;

#[derive(Debug, Clone)]
pub struct ProjectExportInput {
    pub project_id: String,
    pub export_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectExportResult {
    pub project_json_path: PathBuf,
    pub assets_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ProjectImportInput {
    pub project_json_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectImportResult {
    pub project_id: String,
}

pub struct ProjectPackageService<'a> {
    connection: &'a Connection,
    asset_root: PathBuf,
}

impl<'a> ProjectPackageService<'a> {
    pub fn new(connection: &'a Connection, asset_root: PathBuf) -> Self {
        Self {
            connection,
            asset_root,
        }
    }

    pub fn export_project(&self, input: ProjectExportInput) -> JoiResult<ProjectExportResult> {
        let repo = Repository::new(self.connection);
        let project = repo.get_project(&input.project_id)?;
        let slug = slugify_project_title(&project.title);
        let assets_folder = format!("{slug}-assets");
        let project_json_path = input.export_dir.join(format!("{slug}.joi-project.json"));
        let assets_dir = input.export_dir.join(&assets_folder);
        if project_json_path.exists() {
            return Err(JoiError::Package(format!(
                "project package already exists: {}",
                project_json_path.display()
            )));
        }

        std::fs::create_dir_all(&input.export_dir)?;
        std::fs::create_dir_all(&assets_dir)?;

        let snapshot = ProjectSnapshotService::new(self.connection).build_snapshot(&project.id)?;
        copy_managed_project_assets(&repo, &self.asset_root, &project.id, &assets_dir)?;

        let package = json!({
            "format_version": 1,
            "exported_by": "Joi Agent",
            "project_id": project.id,
            "snapshot": snapshot,
            "assets_folder": assets_folder,
        });
        std::fs::write(&project_json_path, serde_json::to_vec_pretty(&package)?)?;

        Ok(ProjectExportResult {
            project_json_path,
            assets_dir,
        })
    }

    pub fn import_project(&self, input: ProjectImportInput) -> JoiResult<ProjectImportResult> {
        let package = read_project_package(&input.project_json_path)?;
        validate_package_format(&package)?;
        let snapshot = package
            .get("snapshot")
            .ok_or_else(|| JoiError::Package("project package missing snapshot".to_string()))?;

        let brand_snapshot = snapshot.get("brand").unwrap_or(&Value::Null);
        let project_snapshot = snapshot.get("project").unwrap_or(&Value::Null);
        let repo = Repository::new(self.connection);
        let brand = repo.create_brand(BrandCreate {
            name: string_or_default(brand_snapshot, "name", "Imported Brand"),
            description: string_or_default(brand_snapshot, "description", ""),
        })?;
        let project = repo.create_project(ProjectCreate {
            brand_id: brand.id,
            title: string_or_default(project_snapshot, "title", "Imported Project"),
            advertising_goal: string_or_default(project_snapshot, "advertising_goal", ""),
            duration_seconds: integer_or_default(project_snapshot, "duration_seconds", 15),
        })?;

        Ok(ProjectImportResult {
            project_id: project.id,
        })
    }
}

fn read_project_package(path: &Path) -> JoiResult<Value> {
    let bytes = std::fs::read(path)?;
    serde_json::from_slice(&bytes)
        .map_err(|err| JoiError::Package(format!("malformed project package JSON: {err}")))
}

fn validate_package_format(package: &Value) -> JoiResult<()> {
    let format_version = package
        .get("format_version")
        .and_then(Value::as_i64)
        .ok_or_else(|| JoiError::Package("project package missing format_version".to_string()))?;
    if format_version != 1 {
        return Err(JoiError::Package(format!(
            "unsupported project package format_version: {format_version}"
        )));
    }
    Ok(())
}

fn string_or_default(parent: &Value, key: &str, default: &str) -> String {
    parent
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or(default)
        .to_string()
}

fn integer_or_default(parent: &Value, key: &str, default: i64) -> i64 {
    parent.get(key).and_then(Value::as_i64).unwrap_or(default)
}

pub fn slugify_project_title(title: &str) -> String {
    let mut slug = String::new();
    let mut last_was_separator = true;

    for character in title.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            last_was_separator = false;
        } else if !last_was_separator {
            slug.push('-');
            last_was_separator = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    if slug.is_empty() {
        "joi-project".to_string()
    } else {
        slug
    }
}

fn copy_managed_project_assets(
    repo: &Repository<'_>,
    asset_root: &Path,
    project_id: &str,
    assets_dir: &Path,
) -> JoiResult<()> {
    for asset in repo.list_assets(project_id)? {
        let source = safe_join_asset_path(asset_root, &asset.relative_path)?;
        if !source.is_file() {
            return Err(JoiError::FileSystem(format!(
                "managed export asset is missing or not a file: asset {} at {}",
                asset.id,
                source.display()
            )));
        }

        let destination = assets_dir.join(export_asset_file_name(&asset.id, &asset.relative_path));
        if destination.exists() {
            return Err(JoiError::Package(format!(
                "export asset already exists: {}",
                destination.display()
            )));
        }

        std::fs::copy(&source, &destination)?;
    }

    Ok(())
}

fn export_asset_file_name(asset_id: &str, relative_path: &str) -> String {
    format!(
        "{}.{}",
        safe_filename_stem(asset_id),
        safe_extension(relative_path)
    )
}

fn safe_filename_stem(value: &str) -> String {
    let mut stem = String::new();
    let mut last_was_separator = true;

    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            stem.push(character.to_ascii_lowercase());
            last_was_separator = false;
        } else if matches!(character, '-' | '_') {
            stem.push(character);
            last_was_separator = true;
        } else if !last_was_separator {
            stem.push('-');
            last_was_separator = true;
        }
    }

    while stem.ends_with('-') || stem.ends_with('_') {
        stem.pop();
    }

    if stem.is_empty() {
        "asset".to_string()
    } else {
        stem
    }
}

fn safe_extension(relative_path: &str) -> String {
    let file_name = relative_path.rsplit('/').next().unwrap_or(relative_path);
    let extension = Path::new(file_name)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("bin");
    let extension: String = extension
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_lowercase())
        .collect();

    if extension.is_empty() {
        "bin".to_string()
    } else {
        extension
    }
}
