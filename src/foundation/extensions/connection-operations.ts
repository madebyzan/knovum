import type { ConnectorDefinition } from "../../shared/contracts/generated/ConnectorDefinition";
import type { ExtensionChange } from "../../shared/contracts/generated/ExtensionChange";
import type { McpConnection } from "../../shared/contracts/generated/McpConnection";
import type { McpConnectionCatalog } from "../../shared/contracts/generated/McpConnectionCatalog";
import type { McpResourceContent } from "../../shared/contracts/generated/McpResourceContent";
import type { McpResourceSelection } from "../../shared/contracts/generated/McpResourceSelection";
import { invokeCommand } from "../ipc/invoke-command";

export function readMcpResource(
  connection: McpConnection,
  catalogDigest: string,
  selection: McpResourceSelection,
) {
  return invokeCommand<McpResourceContent>("plugin:extensions|read_mcp_resource", {
    connectionId: connection.id,
    revision: connection.revision,
    catalogDigest,
    selection,
  });
}
export function getMcpPrompt(
  connection: McpConnection,
  catalogDigest: string,
  name: string,
  promptArguments: Record<string, string>,
) {
  return invokeCommand<unknown>("plugin:extensions|get_mcp_prompt", {
    connectionId: connection.id,
    revision: connection.revision,
    catalogDigest,
    name,
    arguments: promptArguments,
  });
}
export function listMcpConnections() {
  return invokeCommand<McpConnection[]>("plugin:extensions|list_mcp_connections");
}
export function createMcpConnection(connectorId: string, name: string) {
  return invokeCommand<McpConnection>("plugin:extensions|create_mcp_connection", { connectorId, name });
}
export function updateMcpConnection(connection: McpConnection) {
  return invokeCommand<McpConnection>("plugin:extensions|update_mcp_connection", { connection });
}
export function checkMcpConnection(connectionId: string) {
  return invokeCommand<McpConnectionCatalog>("plugin:extensions|check_mcp_connection", { connectionId });
}
export function deleteMcpConnection(connection: McpConnection) {
  return invokeCommand<void>("plugin:extensions|delete_mcp_connection", {
    connectionId: connection.id,
    revision: connection.revision,
  });
}

export function createMcpServer(definition: ConnectorDefinition) {
  return invokeCommand<ExtensionChange<McpConnection>>("plugin:extensions|create_mcp_server", { definition });
}
