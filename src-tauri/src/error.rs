use serde::Serialize;
use thiserror::Error;

pub type JoiResult<T> = Result<T, JoiError>;

#[derive(Debug, Error)]
pub enum JoiError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("database error: {0}")]
    Database(String),
    #[error("file system error: {0}")]
    FileSystem(String),
    #[error("import/export error: {0}")]
    Package(String),
    #[error("not found: {0}")]
    NotFound(String),
}

impl Serialize for JoiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<rusqlite::Error> for JoiError {
    fn from(value: rusqlite::Error) -> Self {
        JoiError::Database(value.to_string())
    }
}

impl From<std::io::Error> for JoiError {
    fn from(value: std::io::Error) -> Self {
        JoiError::FileSystem(value.to_string())
    }
}

impl From<serde_json::Error> for JoiError {
    fn from(value: serde_json::Error) -> Self {
        JoiError::Package(value.to_string())
    }
}
