use std::path::{Component, Path, PathBuf};

use rusqlite::Connection;
use sha2::{Digest, Sha256};

use crate::error::{JoiError, JoiResult};
use crate::models::{new_id, Asset};
use crate::repositories::{AssetCreate, Repository};

#[derive(Debug, Clone)]
pub struct AssetImportInput {
    pub project_id: String,
    pub kind: String,
    pub source_path: PathBuf,
    pub display_name: String,
}

pub struct AssetService<'a> {
    connection: &'a Connection,
    asset_root: PathBuf,
}

impl<'a> AssetService<'a> {
    pub fn new(connection: &'a Connection, asset_root: PathBuf) -> Self {
        Self {
            connection,
            asset_root,
        }
    }

    pub fn import_local_file(&self, input: AssetImportInput) -> JoiResult<Asset> {
        if !input.source_path.is_file() {
            return Err(JoiError::FileSystem(format!(
                "source asset does not exist: {}",
                input.source_path.display()
            )));
        }

        let bytes = std::fs::read(&input.source_path)?;
        let sha256 = hex::encode(Sha256::digest(&bytes));
        let extension = input
            .source_path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("bin");
        let asset_id = new_id();
        let relative_path = format!(
            "projects/{}/assets/{}.{}",
            input.project_id, asset_id, extension
        );
        let destination = safe_join_asset_path(&self.asset_root, &relative_path)?;
        std::fs::copy(&input.source_path, &destination)?;
        let mime_type = mime_guess::from_path(&input.source_path)
            .first_or_octet_stream()
            .essence_str()
            .to_string();
        let file_size_bytes = std::fs::metadata(&destination)?.len() as i64;

        let repo = Repository::new(self.connection);
        repo.create_asset(AssetCreate {
            project_id: input.project_id,
            kind: input.kind,
            display_name: input.display_name,
            relative_path,
            source_uri: input.source_path.to_string_lossy().to_string(),
            mime_type,
            file_size_bytes,
            sha256,
        })
    }
}

pub fn safe_join_asset_path(root: &Path, relative_path: &str) -> JoiResult<PathBuf> {
    let relative = Path::new(relative_path);
    if relative.is_absolute() {
        return Err(JoiError::FileSystem(format!(
            "asset path escapes root: {}",
            relative_path
        )));
    }

    let mut clean_relative = PathBuf::new();
    for component in relative.components() {
        match component {
            Component::Normal(value) => clean_relative.push(value),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(JoiError::FileSystem(format!(
                    "asset path escapes root: {}",
                    relative_path
                )));
            }
        }
    }

    if clean_relative.as_os_str().is_empty() {
        return Err(JoiError::FileSystem("asset path is empty".to_string()));
    }

    std::fs::create_dir_all(root)?;
    let root_full = root.canonicalize()?;
    let candidate = root_full.join(clean_relative);
    let parent = candidate.parent().unwrap_or(root_full.as_path());
    std::fs::create_dir_all(parent)?;
    let parent_full = parent.canonicalize()?;
    if !parent_full.starts_with(&root_full) {
        return Err(JoiError::FileSystem(format!(
            "asset path escapes root: {}",
            relative_path
        )));
    }
    Ok(candidate)
}
