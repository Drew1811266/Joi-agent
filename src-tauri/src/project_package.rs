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
    pub delivery_report_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectExportResult {
    pub project_json_path: PathBuf,
    pub assets_dir: PathBuf,
    pub delivery_report_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ProjectImportInput {
    pub project_json_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectImportResult {
    pub project_id: String,
}

struct ProjectImportFields {
    brand_name: String,
    brand_description: String,
    project_title: String,
    advertising_goal: String,
    duration_seconds: i64,
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
        let delivery_report = match input.delivery_report_id.as_deref() {
            Some(report_id) => {
                let report = repo.get_delivery_report(report_id)?;
                if report.project_id != project.id {
                    return Err(JoiError::Package(format!(
                        "project package delivery report {} does not belong to project {}",
                        report.id, project.id
                    )));
                }
                Some(report)
            }
            None => None,
        };
        let delivery_report_path = delivery_report
            .as_ref()
            .map(|_| input.export_dir.join(format!("{slug}-delivery-report.md")));
        if project_json_path.exists() {
            return Err(JoiError::Package(format!(
                "project package already exists: {}",
                project_json_path.display()
            )));
        }
        if let Some(path) = delivery_report_path.as_ref() {
            if path.exists() {
                return Err(JoiError::Package(format!(
                    "delivery report export already exists: {}",
                    path.display()
                )));
            }
        }

        std::fs::create_dir_all(&input.export_dir)?;
        std::fs::create_dir_all(&assets_dir)?;

        let snapshot = ProjectSnapshotService::new(self.connection).build_snapshot(&project.id)?;
        copy_managed_project_assets(&repo, &self.asset_root, &project.id, &assets_dir)?;

        let mut package = json!({
            "format_version": 1,
            "exported_by": "Joi Agent",
            "project_id": project.id.clone(),
            "snapshot": snapshot,
            "assets_folder": assets_folder,
        });
        if let (Some(report), Some(path)) =
            (delivery_report.as_ref(), delivery_report_path.as_ref())
        {
            let markdown_file = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string();
            package["delivery_report"] = json!({
                "id": report.id,
                "title": report.title,
                "markdown_file": markdown_file,
            });
            std::fs::write(path, report.markdown.as_bytes())?;
        }
        std::fs::write(&project_json_path, serde_json::to_vec_pretty(&package)?)?;

        Ok(ProjectExportResult {
            project_json_path,
            assets_dir,
            delivery_report_path,
        })
    }

    pub fn import_project(&self, input: ProjectImportInput) -> JoiResult<ProjectImportResult> {
        let package = read_project_package(&input.project_json_path)?;
        let fields = parse_project_import_fields(&package)?;
        let repo = Repository::new(self.connection);
        let brand = repo.create_brand(BrandCreate {
            name: fields.brand_name,
            description: fields.brand_description,
        })?;
        let project = repo.create_project(ProjectCreate {
            brand_id: brand.id,
            title: fields.project_title,
            advertising_goal: fields.advertising_goal,
            duration_seconds: fields.duration_seconds,
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

fn parse_project_import_fields(package: &Value) -> JoiResult<ProjectImportFields> {
    validate_package_format(package)?;
    let snapshot = required_object(package, "snapshot")?;
    validate_format_version(snapshot, "snapshot.format_version")?;
    let brand = required_object(snapshot, "brand")?;
    let project = required_object(snapshot, "project")?;

    Ok(ProjectImportFields {
        brand_name: required_non_blank_string(brand, "name", "brand.name")?,
        brand_description: optional_string(brand, "description", "brand.description", "")?,
        project_title: required_non_blank_string(project, "title", "project.title")?,
        advertising_goal: optional_string(
            project,
            "advertising_goal",
            "project.advertising_goal",
            "",
        )?,
        duration_seconds: optional_positive_integer(
            project,
            "duration_seconds",
            "project.duration_seconds",
            15,
        )?,
    })
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

fn validate_format_version(parent: &Value, field: &str) -> JoiResult<()> {
    let format_version = parent
        .get("format_version")
        .and_then(Value::as_i64)
        .ok_or_else(|| JoiError::Package(format!("{field} must be 1")))?;
    if format_version != 1 {
        return Err(JoiError::Package(format!(
            "unsupported {field}: {format_version}"
        )));
    }
    Ok(())
}

fn required_object<'a>(parent: &'a Value, key: &str) -> JoiResult<&'a Value> {
    let value = parent
        .get(key)
        .ok_or_else(|| JoiError::Package(format!("{key} must be an object")))?;
    if !value.is_object() {
        return Err(JoiError::Package(format!("{key} must be an object")));
    }
    Ok(value)
}

fn required_non_blank_string(parent: &Value, key: &str, field: &str) -> JoiResult<String> {
    let value = parent
        .get(key)
        .ok_or_else(|| JoiError::Package(format!("{field} must be a non-empty string")))?;
    let text = value
        .as_str()
        .ok_or_else(|| JoiError::Package(format!("{field} must be a non-empty string")))?;
    if text.trim().is_empty() {
        return Err(JoiError::Package(format!(
            "{field} must be a non-empty string"
        )));
    }
    Ok(text.trim().to_string())
}

fn optional_string(parent: &Value, key: &str, field: &str, default: &str) -> JoiResult<String> {
    match parent.get(key) {
        Some(value) => value
            .as_str()
            .map(ToString::to_string)
            .ok_or_else(|| JoiError::Package(format!("{field} must be a string"))),
        None => Ok(default.to_string()),
    }
}

fn optional_positive_integer(
    parent: &Value,
    key: &str,
    field: &str,
    default: i64,
) -> JoiResult<i64> {
    match parent.get(key) {
        Some(value) => {
            let number = value
                .as_i64()
                .ok_or_else(|| JoiError::Package(format!("{field} must be a positive integer")))?;
            if number <= 0 {
                return Err(JoiError::Package(format!(
                    "{field} must be a positive integer"
                )));
            }
            Ok(number)
        }
        None => Ok(default),
    }
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
