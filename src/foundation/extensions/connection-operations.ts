import type { McpConnection } from "../../shared/contracts/generated/McpConnection";
import type { McpConnectionCatalog } from "../../shared/contracts/generated/McpConnectionCatalog";
import type { McpResourceContent } from "../../shared/contracts/generated/McpResourceContent";
import type { McpResourceSelection } from "../../shared/contracts/generated/McpResourceSelection";
import { invokeCommand } from "../ipc/invoke-command";

export const readMcpResource = (
  connection: McpConnection,
  catalogDigest: string,
  selection: McpResourceSelection,
) =>
  invokeCommand<McpResourceContent>("plugin:extensions|read_mcp_resource", {
    id: connection.id,
    revision: connection.revision,
    catalogDigest,
    selection,
  });
export const getMcpPrompt = (
  connection: McpConnection,
  catalogDigest: string,
  name: string,
  arguments_: Record<string, string>,
) =>
  invokeCommand<unknown>("plugin:extensions|get_mcp_prompt", {
    id: connection.id,
    revision: connection.revision,
    catalogDigest,
    name,
    arguments: arguments_,
  });
export const listMcpConnections = () =>
  invokeCommand<McpConnection[]>("plugin:extensions|list_mcp_connections");
export const createMcpConnection = (connectorId: string, name: string) =>
  invokeCommand<McpConnection>("plugin:extensions|create_mcp_connection", { connectorId, name });
export const updateMcpConnection = (connection: McpConnection) =>
  invokeCommand<McpConnection>("plugin:extensions|update_mcp_connection", { connection });
export const checkMcpConnection = (connectionId: string) =>
  invokeCommand<McpConnectionCatalog>("plugin:extensions|check_mcp_connection", { id: connectionId });
export const deleteMcpConnection = (connection: McpConnection) =>
  invokeCommand<void>("plugin:extensions|delete_mcp_connection", {
    id: connection.id,
    revision: connection.revision,
  });
