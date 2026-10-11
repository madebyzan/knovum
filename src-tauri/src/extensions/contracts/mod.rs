pub mod connections;
pub mod operations;
pub mod plugins;
pub mod resources;
pub mod skills;
pub mod snapshots;
pub use connections::{
    ConnectorAccount, ConnectorAuthorization, ConnectorDefinition, McpCatalogTool, McpConnection,
    McpConnectionCatalog, McpEndpoint, McpToolPermission, McpToolPolicy,
    PluginConnectorConfiguration,
};
pub use operations::{ExtensionChange, PluginOperationStage};
pub use plugins::{
    MarketplaceSource, PluginComponent, PluginConnectorDeclaration, PluginInspection,
    PluginInstallation, PluginManifest, PluginMarketplace, PluginPublication, PluginRelease,
    PluginSource, PluginUpdatePreview,
};
pub use resources::{McpResourceContent, McpResourceSelection};
pub use skills::{SkillContent, SkillRevision};
pub use snapshots::{
    AgentConnectionSelection, AgentExtensionAvailability, AgentExtensionSelection,
    AgentExtensionSnapshot, AgentExtensionTool, AgentSkillFile, AgentToolBinding,
    McpResourceTemplateSelection, McpResourceTemplateValue, SkillFileEncoding,
};
