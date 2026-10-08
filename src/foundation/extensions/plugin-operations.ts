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

export const listPluginInstallations = () =>
  invokeCommand<PluginInstallation[]>("plugin:extensions|list_plugin_installations");
export const inspectPluginSource = (source: PluginSource) =>
  runPluginOperation<PluginInspection>("inspect_plugin_source", "extensions.plugins.tasks.inspect", {
    source,
  });
export const installPlugin = (
  source: PluginSource,
  expectedDigest: string,
  publication: PluginPublication | null,
) =>
  runPluginOperation<ExtensionChange<PluginInstallation>>(
    "install_plugin",
    "extensions.plugins.tasks.install",
    {
      source,
      expectedDigest,
      publication,
    },
  );
export const updatePlugin = (pluginId: string, expectedDigest: string) =>
  runPluginOperation<ExtensionChange<PluginInstallation>>(
    "update_plugin",
    "extensions.plugins.tasks.update",
    {
      id: pluginId,
      expectedDigest,
    },
  );
export const setPluginEnabled = (pluginId: string, enabled: boolean) =>
  invokeCommand<void>("plugin:extensions|set_plugin_enabled", { id: pluginId, enabled });
export const uninstallPlugin = (pluginId: string) =>
  invokeCommand<ExtensionChange<null>>("plugin:extensions|uninstall_plugin", { id: pluginId });
export const searchPlugins = (query: string) =>
  invokeCommand<PluginMarketplace[]>("plugin:extensions|search_plugins", { query });
export const addPluginMarketplace = (source: MarketplaceSource) =>
  runPluginOperation<PluginMarketplace>("add_plugin_marketplace", "extensions.plugins.tasks.catalog", {
    source,
  });
export const refreshPluginMarketplace = (marketplaceId: string) =>
  runPluginOperation<PluginMarketplace>("refresh_plugin_marketplace", "extensions.plugins.tasks.catalog", {
    id: marketplaceId,
  });
export const removePluginMarketplace = (marketplaceId: string) =>
  invokeCommand<void>("plugin:extensions|remove_plugin_marketplace", { id: marketplaceId });

export const inspectPluginUpdate = (pluginId: string) =>
  runPluginOperation<PluginUpdatePreview>("inspect_plugin_update", "extensions.plugins.tasks.inspect", {
    id: pluginId,
  });
