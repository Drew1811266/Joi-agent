use std::path::PathBuf;

use tempfile::TempDir;

pub struct TestApp {
    pub temp_dir: TempDir,
    pub db_path: PathBuf,
}

impl TestApp {
    pub fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let db_path = temp_dir.path().join("joi.db");
        Self { temp_dir, db_path }
    }
}
