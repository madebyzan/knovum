import { randomUUID } from "node:crypto";

import { expect } from "@wdio/globals";

import type { ConnectorDefinition } from "../../src/shared/contracts/generated/ConnectorDefinition";
import type { ExtensionChange } from "../../src/shared/contracts/generated/ExtensionChange";
import type { McpConnection } from "../../src/shared/contracts/generated/McpConnection";
import { extensionCommand } from "./extension-ipc";

const listConnections = () => extensionCommand<McpConnection[]>("list_mcp_connections");

// Exercises persisted IPC ownership without contacting a provider or starting an MCP process.
describe("external service connector lifecycle", () => {
  it("reuses one definition, revises every instance and deletes instances independently", async () => {
    let definition: ConnectorDefinition | undefined;
    try {
      definition = (
        await extensionCommand<ExtensionChange<ConnectorDefinition>>("create_connector_definition", {
          definition: {
            id: "",
            revision: "",
            name: `Service ${randomUUID()}`,
            description: "Inspect connector ownership",
            pluginId: null,
            componentName: null,
            authorization: { type: "none" },
            endpoint: {
              type: "streamable_http",
              url: "https://example.com/mcp",
              headers: {},
              allowLocalNetwork: false,
            },
          },
        })
      ).value;
      const work = await extensionCommand<McpConnection>("create_mcp_connection", {
        connectorId: definition.id,
        name: "Work",
      });
      const personal = await extensionCommand<McpConnection>("create_mcp_connection", {
        connectorId: definition.id,
        name: "Personal",
      });
      await extensionCommand("update_mcp_connection", { connection: { ...work, enabled: true } });
      expect(
        (await listConnections()).filter((connection) => connection.connectorId === definition!.id),
      ).toHaveLength(2);
      const oldDefinition = definition;
      definition = (
        await extensionCommand<ExtensionChange<ConnectorDefinition>>("update_connector_definition", {
          definition: { ...definition, description: "Updated service purpose" },
        })
      ).value;
      expect(definition.id).toBe(oldDefinition.id);
      const updatedWork = (await listConnections()).find((connection) => connection.id === work.id)!;
      expect(updatedWork.enabled).toBe(true);
      expect(updatedWork.revision).not.toBe(work.revision);
      definition = (
        await extensionCommand<ExtensionChange<ConnectorDefinition>>("update_connector_definition", {
          definition: {
            ...definition,
            endpoint: {
              type: "streamable_http",
              url: "https://new.example.com/mcp",
              headers: {},
              allowLocalNetwork: false,
            },
          },
        })
      ).value;
      const revised = (await listConnections()).filter(
        (connection) => connection.connectorId === definition!.id,
      );
      expect(
        revised.every(
          (connection) =>
            !connection.enabled && connection.accountId === null && connection.tools.length === 0,
        ),
      ).toBe(true);
      const currentWork = revised.find((connection) => connection.id === work.id)!;
      await extensionCommand("delete_mcp_connection", { id: currentWork.id, revision: currentWork.revision });
      expect((await listConnections()).some((connection) => connection.id === personal.id)).toBe(true);
      expect(
        (await extensionCommand<ConnectorDefinition[]>("list_connector_definitions")).some(
          (connector) => connector.id === definition!.id,
        ),
      ).toBe(true);
      await extensionCommand("delete_connector_definition", {
        id: definition.id,
        revision: definition.revision,
      });
      expect((await listConnections()).some((connection) => connection.id === personal.id)).toBe(false);
      definition = undefined;
    } finally {
      if (definition)
        await extensionCommand("delete_connector_definition", {
          id: definition.id,
          revision: definition.revision,
        });
    }
  });
});
