import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";

export function connectorDraft(): ConnectorDefinition {
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
