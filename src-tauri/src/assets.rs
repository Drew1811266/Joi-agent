use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Component, Path, PathBuf};

use rusqlite::Connection;
use sha2::{Digest, Sha256};

use crate::error::{JoiError, JoiResult};
use crate::models::{new_id, Asset, AssetKind};
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
        let kind = AssetKind::try_from(input.kind.as_str())?;
        validate_single_path_segment("project id", &input.project_id)?;
        let repo = Repository::new(self.connection);
        repo.get_project(&input.project_id)?;

        if !input.source_path.is_file() {
            return Err(JoiError::FileSystem(format!(
                "source asset does not exist: {}",
                input.source_path.display()
            )));
        }

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
        let (sha256, file_size_bytes) = match copy_file_with_hash(&input.source_path, &destination)
        {
            Ok(result) => result,
            Err(error) => {
                let _ = std::fs::remove_file(&destination);
                return Err(error);
            }
        };
        let mime_type = mime_guess::from_path(&input.source_path)
            .first_or_octet_stream()
            .essence_str()
            .to_string();

        let result = repo.create_asset(AssetCreate {
            project_id: input.project_id,
            kind: kind.as_str().to_string(),
            display_name: input.display_name,
            relative_path,
            source_uri: input.source_path.to_string_lossy().to_string(),
            mime_type,
            file_size_bytes,
            sha256,
        });
        if result.is_err() {
            let _ = std::fs::remove_file(&destination);
        }
        result
    }
}

pub fn safe_join_asset_path(root: &Path, relative_path: &str) -> JoiResult<PathBuf> {
    if relative_path.contains('\\') {
        return Err(JoiError::FileSystem(format!(
            "asset path escapes root: {}",
            relative_path
        )));
    }

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

fn validate_single_path_segment(label: &str, value: &str) -> JoiResult<()> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.contains('/')
        || value.contains('\\')
    {
        return Err(JoiError::Validation(format!(
            "{} must be a safe path segment",
            label
        )));
    }

    let mut components = Path::new(value).components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(_)), None) => Ok(()),
        _ => Err(JoiError::Validation(format!(
            "{} must be a safe path segment",
            label
        ))),
    }
}

fn copy_file_with_hash(source: &Path, destination: &Path) -> JoiResult<(String, i64)> {
    let mut reader = BufReader::new(File::open(source)?);
    let mut writer = BufWriter::new(File::create(destination)?);
    let mut hasher = Sha256::new();
    let mut file_size_bytes = 0_i64;
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        writer.write_all(&buffer[..bytes_read])?;
        hasher.update(&buffer[..bytes_read]);
        file_size_bytes += bytes_read as i64;
    }
    writer.flush()?;

    Ok((hex::encode(hasher.finalize()), file_size_bytes))
}
