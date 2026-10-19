import type { ConnectorAccount } from "../../shared/contracts/generated/ConnectorAccount";
import type { ConnectorDefinition } from "../../shared/contracts/generated/ConnectorDefinition";
import type { ExtensionChange } from "../../shared/contracts/generated/ExtensionChange";
import type { PluginConnectorConfiguration } from "../../shared/contracts/generated/PluginConnectorConfiguration";
import { invokeCommand } from "../ipc/invoke-command";

export function listConnectorDefinitions() {
  return invokeCommand<ConnectorDefinition[]>("plugin:extensions|list_connector_definitions");
}
export function updateConnectorDefinition(definition: ConnectorDefinition) {
  return invokeCommand<ExtensionChange<ConnectorDefinition>>(
    "plugin:extensions|update_connector_definition",
    {
      definition,
    },
  );
}
export function deleteConnectorDefinition(definition: ConnectorDefinition) {
  return invokeCommand<ExtensionChange<null>>("plugin:extensions|delete_connector_definition", {
    connectorId: definition.id,
    revision: definition.revision,
  });
}
export function listConnectorAccounts() {
  return invokeCommand<ConnectorAccount[]>("plugin:extensions|list_connector_accounts");
}
/** Connects a service, or renews the authorization of one that is already connected. */
export function connectConnector(connectorId: string, secretFields: Record<string, string>) {
  return invokeCommand<ConnectorAccount>("plugin:extensions|connect_connector", {
    connectorId,
    secretFields,
  });
}
export function cancelConnectorAuthorization(connectorId: string) {
  return invokeCommand<void>("plugin:extensions|cancel_connector_authorization", { connectorId });
}
export function disconnectConnector(connectorId: string) {
  return invokeCommand<void>("plugin:extensions|disconnect_connector", { connectorId });
}

export function configurePluginConnector(
  definition: ConnectorDefinition,
  configuration: PluginConnectorConfiguration,
) {
  return invokeCommand<ExtensionChange<ConnectorDefinition>>("plugin:extensions|configure_plugin_connector", {
    connectorId: definition.id,
    revision: definition.revision,
    configuration,
  });
}
