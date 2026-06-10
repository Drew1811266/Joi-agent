use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{JoiError, JoiResult};

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

macro_rules! string_enum {
    ($name:ident { $($variant:ident => $value:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $value),+
                }
            }
        }

        impl TryFrom<&str> for $name {
            type Error = JoiError;

            fn try_from(value: &str) -> JoiResult<Self> {
                match value {
                    $($value => Ok(Self::$variant),)+
                    other => Err(JoiError::Validation(format!("invalid {}: {}", stringify!($name), other))),
                }
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::try_from(value.as_str()).map_err(serde::de::Error::custom)
            }
        }
    };
}

string_enum!(AssetKind {
    ProductImage => "product_image",
    ReferenceImage => "reference_image",
    ReferenceVideo => "reference_video",
    Link => "link",
    Other => "other",
});

string_enum!(PromptPlatform {
    JimengVideo => "jimeng_video",
    GrokVideo => "grok_video",
    Banana2Image => "banana_2_image",
    JimengImage => "jimeng_image",
    GptImage2 => "gpt_image_2",
});

string_enum!(PromptModality {
    Video => "video",
    Image => "image",
});

string_enum!(MemoryScope {
    User => "user",
    Brand => "brand",
    Project => "project",
});

string_enum!(MemoryStatus {
    Proposed => "proposed",
    Accepted => "accepted",
    Rejected => "rejected",
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Brand {
    pub id: String,
    pub name: String,
    pub description: String,
    pub style_keywords: Value,
    pub visual_preferences: Value,
    pub negative_preferences: Value,
    pub common_scenes: Value,
    pub model_preferences: Value,
    pub platform_preferences: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub brand_id: String,
    pub title: String,
    pub advertising_goal: String,
    pub duration_seconds: i64,
    pub target_platforms: Value,
    pub workflow_stage: String,
    pub current_version_id: Option<String>,
    pub final_version_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub project_id: String,
    pub kind: String,
    pub display_name: String,
    pub relative_path: String,
    pub source_uri: String,
    pub mime_type: String,
    pub file_size_bytes: i64,
    pub sha256: String,
    pub metadata_json: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchReport {
    pub id: String,
    pub project_id: String,
    pub summary: String,
    pub findings_json: Value,
    pub sources_json: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductUnderstanding {
    pub id: String,
    pub project_id: String,
    pub product_name: String,
    pub category: String,
    pub audience: String,
    pub selling_points_json: Value,
    pub constraints_json: Value,
    pub notes: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativeDirection {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub concept: String,
    pub tone: String,
    pub visual_style: String,
    pub scene_direction: String,
    pub rationale: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storyboard {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub duration_seconds: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shot {
    pub id: String,
    pub storyboard_id: String,
    pub shot_number: i64,
    pub duration_seconds: i64,
    pub description: String,
    pub model_action: String,
    pub camera_movement: String,
    pub scene: String,
    pub lighting: String,
    pub subtitle_or_voiceover: String,
    pub rationale: String,
    pub is_locked: bool,
    pub metadata_json: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptPackage {
    pub id: String,
    pub project_id: String,
    pub shot_id: String,
    pub platform: String,
    pub modality: String,
    pub prompt_text: String,
    pub negative_prompt: String,
    pub parameters_json: Value,
    pub is_locked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectVersion {
    pub id: String,
    pub project_id: String,
    pub version_number: i64,
    pub label: String,
    pub change_reason: String,
    pub changed_entities_json: Value,
    pub snapshot_json: Value,
    pub created_by: String,
    pub is_final_candidate: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub scope: String,
    pub brand_id: Option<String>,
    pub project_id: Option<String>,
    pub content: String,
    pub source: String,
    pub source_entity_type: String,
    pub source_entity_id: String,
    pub confidence: f64,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
