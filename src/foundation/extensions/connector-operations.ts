import type { ConnectorAccount } from "../../shared/contracts/generated/ConnectorAccount";
import type { ConnectorDefinition } from "../../shared/contracts/generated/ConnectorDefinition";
import type { ExtensionChange } from "../../shared/contracts/generated/ExtensionChange";
import type { PluginConnectorConfiguration } from "../../shared/contracts/generated/PluginConnectorConfiguration";
import { invokeCommand } from "../ipc/invoke-command";

export const listConnectorDefinitions = () =>
  invokeCommand<ConnectorDefinition[]>("plugin:extensions|list_connector_definitions");
export const createConnectorDefinition = (definition: ConnectorDefinition) =>
  invokeCommand<ExtensionChange<ConnectorDefinition>>("plugin:extensions|create_connector_definition", {
    definition,
  });
export const updateConnectorDefinition = (definition: ConnectorDefinition) =>
  invokeCommand<ExtensionChange<ConnectorDefinition>>("plugin:extensions|update_connector_definition", {
    definition,
  });
export const deleteConnectorDefinition = (definition: ConnectorDefinition) =>
  invokeCommand<ExtensionChange<null>>("plugin:extensions|delete_connector_definition", {
    id: definition.id,
    revision: definition.revision,
  });
export const createConnectorAccount = (connectorId: string, displayName: string) =>
  invokeCommand<ConnectorAccount>("plugin:extensions|create_connector_account", { connectorId, displayName });

export const listConnectorAccounts = () =>
  invokeCommand<ConnectorAccount[]>("plugin:extensions|list_connector_accounts");
export const authorizeConnectorAccount = (id: string, secretFields: Record<string, string>) =>
  invokeCommand<ConnectorAccount>("plugin:extensions|authorize_connector_account", { id, secretFields });
export const cancelConnectorAuthorization = (id: string) =>
  invokeCommand<void>("plugin:extensions|cancel_connector_authorization", { id });
export const disconnectConnectorAccount = (id: string) =>
  invokeCommand<void>("plugin:extensions|disconnect_connector_account", { id });

export const configurePluginConnector = (
  definition: ConnectorDefinition,
  configuration: PluginConnectorConfiguration,
) =>
  invokeCommand<ExtensionChange<ConnectorDefinition>>("plugin:extensions|configure_plugin_connector", {
    id: definition.id,
    revision: definition.revision,
    configuration,
  });
