import type { McpConnection } from "../../../shared/contracts/generated/McpConnection";
export function connectionDraft(): McpConnection {
  return {
    id: "",
    connectorId: "",
    name: "",
    pluginId: null,
    componentName: null,
    revision: "",
    accountId: null,
    enabled: false,
    tools: [],
    endpoint: { type: "streamable_http", url: "", headers: {}, allowLocalNetwork: false },
  };
}
