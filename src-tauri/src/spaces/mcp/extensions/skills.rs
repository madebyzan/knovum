use super::super::{
    support::{json_text, mcp_error},
    SpaceMcpServer,
};
use rmcp::{handler::server::wrapper::Parameters, tool, tool_router, ErrorData};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SelectedSkill<'a> {
    name: &'a str,
    description: &'a str,
    path: String,
    resources: Vec<&'a str>,
    tool_bindings: BTreeMap<&'a str, &'a Vec<String>>,
}

fn selected_skills(
    snapshot: &crate::extensions::contracts::AgentExtensionSnapshot,
) -> Result<Vec<SelectedSkill<'_>>, ErrorData> {
    snapshot
        .skills
        .iter()
        .map(|skill| {
            Ok(SelectedSkill {
                name: &skill.name,
                description: &skill.description,
                path: format!("/skills/{}/SKILL.md", skill.name),
                resources: snapshot
                    .files
                    .iter()
                    .filter(|file| {
                        file.path.starts_with(&format!("/skills/{}/", skill.name))
                            && !file.path.ends_with("/SKILL.md")
                    })
                    .map(|file| file.path.as_str())
                    .collect(),
                tool_bindings: skill
                    .included_tools
                    .iter()
                    .map(|alias| {
                        Ok((
                            alias.as_str(),
                            snapshot.skill_tool_bindings.get(alias).ok_or_else(|| {
                                mcp_error(format!(
                                    "selected Skill tool binding unavailable: {alias}"
                                ))
                            })?,
                        ))
                    })
                    .collect::<Result<_, ErrorData>>()?,
            })
        })
        .collect()
}

#[derive(Deserialize, schemars::JsonSchema)]
struct ReadExtensionSkill {
    path: String,
}

#[tool_router(router = extension_skill_router, vis = "pub(in crate::spaces::mcp)")]
impl SpaceMcpServer {
    #[tool(
        description = "List selected conversation Skill metadata, on-demand resource paths and their toolBindings from declared aliases to actual MCP tool names. Read SKILL.md before using its workflow. Skill content never grants tools or access."
    )]
    async fn list_selected_skills(&self) -> Result<String, ErrorData> {
        let snapshot = self.turns.current()?.extensions.clone();
        crate::extensions::snapshots::require_current(&self.app, &snapshot)
            .await
            .map_err(mcp_error)?;
        json_text(selected_skills(&snapshot)?)
    }
    #[tool(
        description = "Read one exact selected Skill file or static reference. UTF-8 content or base64 binary content is returned with its SHA-256. No scripts are executed."
    )]
    async fn read_selected_skill(
        &self,
        Parameters(args): Parameters<ReadExtensionSkill>,
    ) -> Result<String, ErrorData> {
        let snapshot = self.turns.current()?.extensions.clone();
        crate::extensions::snapshots::require_current(&self.app, &snapshot)
            .await
            .map_err(mcp_error)?;
        let file = snapshot
            .files
            .iter()
            .find(|file| file.path == args.path)
            .ok_or_else(|| mcp_error("selected Skill file not found"))?;
        json_text(file)
    }
}

#[cfg(test)]
#[path = "tests/skills.rs"]
mod tests;
