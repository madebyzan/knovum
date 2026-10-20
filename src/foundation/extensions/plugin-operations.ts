import type { ExtensionChange } from "../../shared/contracts/generated/ExtensionChange";
import type { MarketplaceSource } from "../../shared/contracts/generated/MarketplaceSource";
import type { PluginInspection } from "../../shared/contracts/generated/PluginInspection";
import type { PluginInstallation } from "../../shared/contracts/generated/PluginInstallation";
import type { PluginMarketplace } from "../../shared/contracts/generated/PluginMarketplace";
import type { PluginPublication } from "../../shared/contracts/generated/PluginPublication";
import type { PluginSource } from "../../shared/contracts/generated/PluginSource";
import type { PluginUpdatePreview } from "../../shared/contracts/generated/PluginUpdatePreview";
import { invokeCommand } from "../ipc/invoke-command";
import { runPluginOperation } from "./plugin-operation";

export function listPluginInstallations() {
  return invokeCommand<PluginInstallation[]>("plugin:extensions|list_plugin_installations");
}
export function inspectPluginSource(source: PluginSource) {
  return runPluginOperation<PluginInspection>("inspect_plugin_source", "extensions.plugins.tasks.inspect", {
    source,
  });
}
export function installPlugin(
  source: PluginSource,
  expectedDigest: string,
  publication: PluginPublication | null,
) {
  return runPluginOperation<ExtensionChange<PluginInstallation>>(
    "install_plugin",
    "extensions.plugins.tasks.install",
    {
      source,
      expectedDigest,
      publication,
    },
  );
}
export function updatePlugin(pluginId: string, expectedDigest: string) {
  return runPluginOperation<ExtensionChange<PluginInstallation>>(
    "update_plugin",
    "extensions.plugins.tasks.update",
    {
      pluginId,
      expectedDigest,
    },
  );
}
export function enablePlugin(pluginId: string) {
  return invokeCommand<void>("plugin:extensions|enable_plugin", { pluginId });
}
export function disablePlugin(pluginId: string) {
  return invokeCommand<void>("plugin:extensions|disable_plugin", { pluginId });
}
export function uninstallPlugin(pluginId: string) {
  return invokeCommand<ExtensionChange<null>>("plugin:extensions|uninstall_plugin", { pluginId });
}
export function searchPlugins(query: string) {
  return invokeCommand<PluginMarketplace[]>("plugin:extensions|search_plugins", { query });
}
export function createPluginMarketplace(source: MarketplaceSource) {
  return runPluginOperation<PluginMarketplace>(
    "create_plugin_marketplace",
    "extensions.plugins.tasks.catalog",
    {
      source,
    },
  );
}
export function refreshPluginMarketplace(marketplaceId: string) {
  return runPluginOperation<PluginMarketplace>(
    "refresh_plugin_marketplace",
    "extensions.plugins.tasks.catalog",
    {
      marketplaceId,
    },
  );
}
export function deletePluginMarketplace(marketplaceId: string) {
  return invokeCommand<void>("plugin:extensions|delete_plugin_marketplace", { marketplaceId });
}

export function inspectPluginUpdate(pluginId: string) {
  return runPluginOperation<PluginUpdatePreview>(
    "inspect_plugin_update",
    "extensions.plugins.tasks.inspect",
    {
      pluginId,
    },
  );
}
