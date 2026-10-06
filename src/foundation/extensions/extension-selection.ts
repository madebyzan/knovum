import type { AgentExtensionSelection } from "../../shared/contracts/generated/AgentExtensionSelection";

export function copyExtensionSelection(selection: AgentExtensionSelection): AgentExtensionSelection {
  return {
    skillIds: [...selection.skillIds],
    connectionIds: [...selection.connectionIds],
    connections: selection.connections.map((connection) => ({
      connectionId: connection.connectionId,
      toolNames: [...connection.toolNames],
      resourceUris: [...connection.resourceUris],
      resourceTemplates: connection.resourceTemplates.map((template) => ({
        uriTemplate: template.uriTemplate,
        parameters: cloneDeep(template.parameters),
      })),
      promptNames: [...connection.promptNames],
    })),
  };
}
import { cloneDeep } from "es-toolkit";
