use serde::{Deserialize, Serialize};

use crate::error::JoiResult;
use crate::models::{
    Asset, Brand, CreativeDirection, MemoryEntry, ProductUnderstanding, Project, ProjectVersion,
};
use crate::repositories::Repository;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProjectContext {
    pub brand: Brand,
    pub project: Project,
    pub assets: Vec<Asset>,
    pub latest_product_understanding: Option<ProductUnderstanding>,
    pub latest_creative_direction: Option<CreativeDirection>,
    pub project_memory: Vec<MemoryEntry>,
    pub versions: Vec<ProjectVersion>,
}

pub fn build_project_context(
    repo: &Repository<'_>,
    project_id: &str,
) -> JoiResult<AgentProjectContext> {
    let project = repo.get_project(project_id)?;
    let brand = repo.get_brand(&project.brand_id)?;
    let assets = repo.list_assets(project_id)?;
    let latest_product_understanding = repo
        .list_product_understandings(project_id)?
        .into_iter()
        .last();
    let latest_creative_direction = repo
        .list_creative_directions(project_id)?
        .into_iter()
        .last();
    let project_memory = repo.list_memory_entries_for_project(project_id)?;
    let versions = repo.list_project_versions(project_id)?;

    Ok(AgentProjectContext {
        brand,
        project,
        assets,
        latest_product_understanding,
        latest_creative_direction,
        project_memory,
        versions,
    })
}
