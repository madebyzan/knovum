import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import type { PluginInstallation } from "../../../shared/contracts/generated/PluginInstallation";

export function isAppConnector(connector: ConnectorDefinition, plugin: PluginInstallation | undefined) {
  return Boolean(
    plugin?.revision.components.some(
      (component) =>
        component.kind === "app" && component.name === connector.componentName && component.supported,
    ),
  );
}
