import { queryOptions } from "@tanstack/vue-query";

import type { AgentExtensionAvailability } from "../../shared/contracts/generated/AgentExtensionAvailability";
import { invokeCommand } from "../ipc/invoke-command";
import { listMcpConnections } from "./connection-operations";
import { listConnectorDefinitions, listConnectorAccounts } from "./connector-operations";
import { listPluginInstallations, searchPlugins } from "./plugin-operations";
import { readSkill, listSkills } from "./skill-operations";

export const EXTENSION_QUERY_KEY = ["agent-extensions"] as const;
export const pluginQuery = queryOptions({
  queryKey: [...EXTENSION_QUERY_KEY, "plugins"],
  queryFn: listPluginInstallations,
});
export const skillQuery = queryOptions({ queryKey: [...EXTENSION_QUERY_KEY, "skills"], queryFn: listSkills });
export const connectionQuery = queryOptions({
  queryKey: [...EXTENSION_QUERY_KEY, "connections"],
  queryFn: listMcpConnections,
});
export const accountQuery = queryOptions({
  queryKey: [...EXTENSION_QUERY_KEY, "accounts"],
  queryFn: listConnectorAccounts,
});
export const connectorQuery = queryOptions({
  queryKey: [...EXTENSION_QUERY_KEY, "connectors"],
  queryFn: listConnectorDefinitions,
});
export const marketplaceQuery = queryOptions({
  queryKey: [...EXTENSION_QUERY_KEY, "marketplaces"],
  queryFn: () => searchPlugins(""),
});
export const extensionAvailabilityQuery = queryOptions({
  queryKey: [...EXTENSION_QUERY_KEY, "availability"],
  queryFn: () =>
    invokeCommand<AgentExtensionAvailability>("plugin:extensions|get_agent_extension_availability"),
});

export function skillContentQuery(skillId: string, digest: string) {
  return queryOptions({
    queryKey: [...EXTENSION_QUERY_KEY, "skill-content", skillId, digest],
    queryFn: () => readSkill(skillId),
  });
}
