use crate::error::{JoiError, JoiResult};
use crate::models::{PromptModality, PromptPlatform};

pub fn validate_required_text(label: &str, value: &str) -> JoiResult<()> {
    if value.trim().is_empty() {
        return Err(JoiError::Validation(format!("{} is required", label)));
    }
    Ok(())
}

pub fn validate_non_negative(label: &str, value: i64) -> JoiResult<()> {
    if value < 0 {
        return Err(JoiError::Validation(format!(
            "{} must be non-negative",
            label
        )));
    }
    Ok(())
}

pub fn validate_prompt_modality(
    platform: PromptPlatform,
    modality: PromptModality,
) -> JoiResult<()> {
    let expected = match platform {
        PromptPlatform::JimengVideo | PromptPlatform::GrokVideo => PromptModality::Video,
        PromptPlatform::Banana2Image | PromptPlatform::JimengImage | PromptPlatform::GptImage2 => {
            PromptModality::Image
        }
    };

    if modality != expected {
        return Err(JoiError::Validation(format!(
            "platform {} requires modality {}",
            platform.as_str(),
            expected.as_str()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AssetKind, MemoryScope, PromptModality, PromptPlatform};

    #[test]
    fn rejects_empty_required_text() {
        assert!(validate_required_text("Brand name", "").is_err());
        assert!(validate_required_text("Project title", "   ").is_err());
        assert!(validate_required_text("Brand name", " Atelier ").is_ok());
    }

    #[test]
    fn prompt_platform_modality_must_match() {
        assert!(
            validate_prompt_modality(PromptPlatform::JimengVideo, PromptModality::Video).is_ok()
        );
        assert!(validate_prompt_modality(PromptPlatform::GptImage2, PromptModality::Image).is_ok());
        assert!(
            validate_prompt_modality(PromptPlatform::GrokVideo, PromptModality::Image).is_err()
        );
        assert!(
            validate_prompt_modality(PromptPlatform::Banana2Image, PromptModality::Video).is_err()
        );
    }

    #[test]
    fn enum_values_round_trip_to_database_strings() {
        assert_eq!(AssetKind::ProductImage.as_str(), "product_image");
        assert_eq!(PromptPlatform::JimengVideo.as_str(), "jimeng_video");
        assert_eq!(MemoryScope::Project.as_str(), "project");
        assert_eq!(
            AssetKind::try_from("reference_video").unwrap(),
            AssetKind::ReferenceVideo
        );
        assert!(AssetKind::try_from("unknown").is_err());
    }
}
