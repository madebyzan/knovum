import type { ConnectorAccount } from "../../shared/contracts/generated/ConnectorAccount";
import type { ConnectorAuthorization } from "../../shared/contracts/generated/ConnectorAuthorization";
import type { ConnectorDefinition } from "../../shared/contracts/generated/ConnectorDefinition";
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
export const createMcpConnection = (connection: McpConnection, authorization: ConnectorAuthorization) =>
  invokeCommand<McpConnection>("plugin:extensions|create_mcp_connection", { connection, authorization });
export const updateMcpConnection = (
  connection: McpConnection,
  authorization: ConnectorAuthorization | null = null,
) => invokeCommand<McpConnection>("plugin:extensions|update_mcp_connection", { connection, authorization });
export const checkMcpConnection = (id: string) =>
  invokeCommand<McpConnectionCatalog>("plugin:extensions|check_mcp_connection", { id });
export const deleteMcpConnection = (id: string) =>
  invokeCommand<void>("plugin:extensions|delete_mcp_connection", { id });
export const listConnectorAccounts = () =>
  invokeCommand<ConnectorAccount[]>("plugin:extensions|list_connector_accounts");
export const listConnectorDefinitions = () =>
  invokeCommand<ConnectorDefinition[]>("plugin:extensions|list_connector_definitions");
export const createConnectorAccount = (connectionId: string, displayName: string) =>
  invokeCommand<ConnectorAccount>("plugin:extensions|create_connector_account", {
    connectionId,
    displayName,
  });
export const authorizeConnectorAccount = (id: string, secretFields: Record<string, string>) =>
  invokeCommand<ConnectorAccount>("plugin:extensions|authorize_connector_account", { id, secretFields });
export const cancelConnectorAuthorization = (id: string) =>
  invokeCommand<void>("plugin:extensions|cancel_connector_authorization", { id });
export const disconnectConnectorAccount = (id: string) =>
  invokeCommand<void>("plugin:extensions|disconnect_connector_account", { id });
