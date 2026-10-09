import type { ConnectorDefinition } from "../../../../shared/contracts/generated/ConnectorDefinition";
import type { McpConnection } from "../../../../shared/contracts/generated/McpConnection";

export const connectorFixture: ConnectorDefinition = {
  id: "github",
  revision: "service-one",
  name: "GitHub",
  description: "Read issues and repositories",
  pluginId: null,
  componentName: null,
  authorization: { type: "none" },
  endpoint: {
    type: "streamable_http",
    url: "https://example.com/mcp",
    headers: {},
    allowLocalNetwork: false,
  },
};

export function mcpConnectionFixture(id: string, name = id): McpConnection {
  return {
    id,
    connectorId: connectorFixture.id,
    name,
    revision: "instance-one",
    accountId: null,
    enabled: false,
    tools: [],
  };
}
