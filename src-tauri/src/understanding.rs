use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::{JoiError, JoiResult};
use crate::models::{CreativeDirection, ProductUnderstanding};
use crate::repositories::{CreativeDirectionCreate, ProductUnderstandingCreate, Repository};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BriefUnderstandingInput {
    pub project_id: String,
    pub brief_text: String,
    pub product_name: String,
    pub category: String,
    pub audience: String,
    pub target_platforms: Vec<String>,
    pub selling_points_text: String,
    pub visual_direction: String,
    pub constraints_text: String,
    pub reference_asset_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefUnderstandingResult {
    pub product_understanding: ProductUnderstanding,
    pub creative_direction: Option<CreativeDirection>,
    pub brief_summary: String,
    pub brand_summary: String,
    pub visual_direction: String,
    pub selling_points: Vec<String>,
    pub constraints: Vec<String>,
    pub missing_questions: Vec<String>,
}

pub fn split_list_text(value: &str) -> Vec<String> {
    value
        .split(|character| matches!(character, '\n' | ',' | '，' | ';' | '；'))
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}

pub fn missing_questions(input: &BriefUnderstandingInput) -> Vec<String> {
    let mut questions = Vec::new();
    if input.brief_text.trim().is_empty() {
        questions.push("What is the core campaign brief for this project?".to_string());
    }
    if input.product_name.trim().is_empty() {
        questions.push("Which product or collection should the content focus on?".to_string());
    }
    if input.category.trim().is_empty() {
        questions.push(
            "What garment category should Joi optimize the visual language for?".to_string(),
        );
    }
    if input.audience.trim().is_empty() {
        questions.push("Who is the primary audience for this ad?".to_string());
    }
    if input.target_platforms.is_empty() {
        questions.push("Which output platforms should this project target?".to_string());
    }
    if split_list_text(&input.selling_points_text).is_empty() {
        questions.push("Which product selling points must be visible in the content?".to_string());
    }
    if input.visual_direction.trim().is_empty() {
        questions.push(
            "What visual direction should guide scenes, lighting, and camera language?".to_string(),
        );
    }
    if input.reference_asset_ids.is_empty() {
        questions.push("Which reference materials should Joi use as visual anchors?".to_string());
    }
    questions
}

pub fn generate_brief_understanding(
    repo: &Repository<'_>,
    input: BriefUnderstandingInput,
) -> JoiResult<BriefUnderstandingResult> {
    if input.brief_text.trim().is_empty()
        && input.product_name.trim().is_empty()
        && input.selling_points_text.trim().is_empty()
        && input.visual_direction.trim().is_empty()
    {
        return Err(JoiError::Validation(
            "brief, product name, selling points, or visual direction is required".to_string(),
        ));
    }

    let project = repo.get_project(&input.project_id)?;
    let brand = repo.get_brand(&project.brand_id)?;
    let project_assets = repo.list_assets(&project.id)?;
    for asset_id in &input.reference_asset_ids {
        if !project_assets.iter().any(|asset| &asset.id == asset_id) {
            return Err(JoiError::Validation(format!(
                "reference asset {asset_id} does not belong to project"
            )));
        }
    }

    let selling_points = split_list_text(&input.selling_points_text);
    let constraints = split_list_text(&input.constraints_text);
    let brief_summary = if input.brief_text.trim().is_empty() {
        format!("{}: {}", project.title, project.advertising_goal)
    } else {
        input.brief_text.trim().to_string()
    };
    let brand_summary = if brand.description.trim().is_empty() {
        brand.name.clone()
    } else {
        format!("{}: {}", brand.name, brand.description)
    };
    let visual_direction = if input.visual_direction.trim().is_empty() {
        "Brand-led visual direction pending user input.".to_string()
    } else {
        input.visual_direction.trim().to_string()
    };
    let missing_questions = missing_questions(&input);
    let notes = json!({
        "format_version": "joi.product_understanding_notes.v1",
        "brief_summary": brief_summary.clone(),
        "brand_summary": brand_summary.clone(),
        "visual_direction": visual_direction.clone(),
        "target_platforms": input.target_platforms.clone(),
        "reference_asset_ids": input.reference_asset_ids.clone(),
        "missing_questions": missing_questions.clone(),
    })
    .to_string();

    let product_understanding = repo.create_product_understanding(ProductUnderstandingCreate {
        project_id: project.id.clone(),
        product_name: input.product_name,
        category: input.category,
        audience: input.audience,
        selling_points: selling_points.clone(),
        constraints: constraints.clone(),
        notes,
    })?;
    let creative_direction = if input.visual_direction.trim().is_empty() {
        None
    } else {
        Some(repo.create_creative_direction(CreativeDirectionCreate {
            project_id: project.id,
            title: "Initial visual direction".to_string(),
            concept: visual_direction.clone(),
            tone: "user-defined".to_string(),
            visual_style: visual_direction.clone(),
            scene_direction: String::new(),
            rationale: "Generated from 0.12 brief and material understanding input.".to_string(),
        })?)
    };

    Ok(BriefUnderstandingResult {
        product_understanding,
        creative_direction,
        brief_summary,
        brand_summary,
        visual_direction,
        selling_points,
        constraints,
        missing_questions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_selling_points_and_constraints_from_mixed_separators() {
        assert_eq!(
            split_list_text("water-resistant cotton, oversized collar\nsoft structure；close texture"),
            vec![
                "water-resistant cotton".to_string(),
                "oversized collar".to_string(),
                "soft structure".to_string(),
                "close texture".to_string(),
            ]
        );
    }

    #[test]
    fn asks_missing_questions_for_blank_inputs() {
        let questions = missing_questions(&BriefUnderstandingInput {
            project_id: "project-1".to_string(),
            brief_text: "".to_string(),
            product_name: "".to_string(),
            category: "".to_string(),
            audience: "".to_string(),
            target_platforms: Vec::new(),
            selling_points_text: "".to_string(),
            visual_direction: "".to_string(),
            constraints_text: "".to_string(),
            reference_asset_ids: Vec::new(),
        });

        assert!(questions.contains(&"What is the core campaign brief for this project?".to_string()));
        assert!(questions.contains(&"Which product or collection should the content focus on?".to_string()));
        assert!(questions.contains(&"Which reference materials should Joi use as visual anchors?".to_string()));
    }
}
