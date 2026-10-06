pub(super) mod commands;
mod metadata;
mod revisions;
use super::{
    content::{self, PackageFiles},
    contracts::SkillRevision,
};
pub(crate) use revisions::{files, list, load};

pub(super) fn review(files: &PackageFiles, name: &str) -> Result<SkillRevision, String> {
    content::validate_files(files)?;
    let metadata = metadata::parse(
        files.get("SKILL.md").ok_or("skill-markdown-required")?,
        name,
    )?;
    Ok(SkillRevision {
        id: uuid::Uuid::new_v4().to_string(),
        plugin_id: None,
        name: metadata.name,
        description: metadata.description,
        digest: content::files_digest(files),
        enabled: true,
        requirements: Vec::new(),
        included_tools: metadata.included_tools,
        unsupported_requirements: metadata.unsupported_requirements,
    })
}
