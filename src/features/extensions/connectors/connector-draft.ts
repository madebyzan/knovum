import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import type { McpEndpoint } from "../../../shared/contracts/generated/McpEndpoint";

export function connectorDraft(): ConnectorDefinition & { endpoint: McpEndpoint } {
  return {
    id: "",
    revision: "",
    name: "",
    description: "",
    pluginId: null,
    componentName: null,
    endpoint: { type: "streamable_http", url: "", headers: {}, allowLocalNetwork: false },
    authorization: { type: "none" },
  };
}
