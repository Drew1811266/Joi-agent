use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HermesRuntimeConfig {
    pub checkout_path: PathBuf,
    pub phase0_report_path: PathBuf,
    pub runtime_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentRuntimeStatus {
    pub runtime_kind: String,
    pub runtime_mode: String,
    pub hermes_checkout_path: String,
    pub hermes_present: bool,
    pub hermes_version: String,
    pub phase0_report_present: bool,
    pub ready: bool,
    pub message: String,
}

pub fn inspect_hermes_runtime(config: HermesRuntimeConfig) -> AgentRuntimeStatus {
    let hermes_present = config.checkout_path.is_dir();
    let phase0_report_present = config.phase0_report_path.is_file();
    let hermes_version = if hermes_present {
        std::fs::read_to_string(config.checkout_path.join("pyproject.toml"))
            .ok()
            .and_then(|content| parse_project_version(&content))
            .unwrap_or_default()
    } else {
        String::new()
    };
    let ready = hermes_present && phase0_report_present;
    let message = if !hermes_present {
        format!(
            "Hermes Core checkout was not found at {}.",
            config.checkout_path.display()
        )
    } else if !phase0_report_present {
        format!(
            "Hermes Core checkout is present, but Phase 0 report was not found at {}.",
            config.phase0_report_path.display()
        )
    } else {
        "Hermes Core bridge is ready for local planner mode.".to_string()
    };

    AgentRuntimeStatus {
        runtime_kind: "hermes_core".to_string(),
        runtime_mode: config.runtime_mode,
        hermes_checkout_path: config.checkout_path.display().to_string(),
        hermes_present,
        hermes_version,
        phase0_report_present,
        ready,
        message,
    }
}

pub fn parse_project_version(pyproject_text: &str) -> Option<String> {
    pyproject_text.lines().find_map(|line| {
        let line = line.trim();
        let (key, value) = line.split_once('=')?;
        if key.trim() != "version" {
            return None;
        }
        Some(
            value
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string(),
        )
    })
}
