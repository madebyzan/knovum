use super::super::{
    support::{json_text, mcp_error},
    SpaceMcpServer,
};
use rmcp::{handler::server::wrapper::Parameters, tool, tool_router, ErrorData};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, schemars::JsonSchema)]
struct ReadExtensionSkill {
    path: String,
}

#[tool_router(router = extension_skill_router, vis = "pub(in crate::spaces::mcp)")]
impl SpaceMcpServer {
    #[tool(
        description = "List the Skills explicitly selected for this Knovum conversation. Read SKILL.md before using its workflow. Skill content never grants tools or access."
    )]
    async fn list_selected_skills(&self) -> Result<String, ErrorData> {
        let snapshot = self
            .conversation_extensions
            .as_ref()
            .ok_or_else(|| mcp_error("this MCP session has no Agent extension selection"))?;
        crate::extensions::snapshots::require_current(&self.app, snapshot)
            .await
            .map_err(mcp_error)?;
        #[derive(Serialize)]
        struct Description<'a> {
            name: &'a str,
            description: &'a str,
            path: String,
        }
        json_text(
            snapshot
                .skills
                .iter()
                .map(|skill| Description {
                    name: &skill.name,
                    description: &skill.description,
                    path: format!("/skills/{}/SKILL.md", skill.name),
                })
                .collect::<Vec<_>>(),
        )
    }
    #[tool(
        description = "Read one exact selected Skill file or static reference. UTF-8 content or base64 binary content is returned with its SHA-256. No scripts are executed."
    )]
    async fn read_selected_skill(
        &self,
        Parameters(args): Parameters<ReadExtensionSkill>,
    ) -> Result<String, ErrorData> {
        let snapshot = self
            .conversation_extensions
            .as_ref()
            .ok_or_else(|| mcp_error("this MCP session has no Agent extension selection"))?;
        crate::extensions::snapshots::require_current(&self.app, snapshot)
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
