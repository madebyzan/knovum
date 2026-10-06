import { queryOptions } from "@tanstack/vue-query";

import type { AgentExtensionAvailability } from "../../shared/contracts/generated/AgentExtensionAvailability";
import { invokeCommand } from "../ipc/invoke-command";
import { listMcpConnections, listConnectorAccounts, listConnectorDefinitions } from "./connection-operations";
import { listPluginInstallations, searchPlugins } from "./plugin-operations";
import { listSkills } from "./skill-operations";

export const extensionQueryKey = ["agent-extensions"] as const;
export const pluginQuery = queryOptions({
  queryKey: [...extensionQueryKey, "plugins"],
  queryFn: listPluginInstallations,
});
export const skillQuery = queryOptions({ queryKey: [...extensionQueryKey, "skills"], queryFn: listSkills });
export const connectionQuery = queryOptions({
  queryKey: [...extensionQueryKey, "connections"],
  queryFn: listMcpConnections,
});
export const accountQuery = queryOptions({
  queryKey: [...extensionQueryKey, "accounts"],
  queryFn: listConnectorAccounts,
});
export const connectorQuery = queryOptions({
  queryKey: [...extensionQueryKey, "connectors"],
  queryFn: listConnectorDefinitions,
});
export const marketplaceQuery = queryOptions({
  queryKey: [...extensionQueryKey, "marketplaces"],
  queryFn: () => searchPlugins(""),
});
export const extensionAvailabilityQuery = queryOptions({
  queryKey: [...extensionQueryKey, "availability"],
  queryFn: () =>
    invokeCommand<AgentExtensionAvailability>("plugin:extensions|get_agent_extension_availability"),
});
