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
  runPluginOperation<PluginInspection>("inspect_plugin_source", "extensions.taskInspect", { source });
export const installPlugin = (
  source: PluginSource,
  expectedDigest: string,
  publication: PluginPublication | null,
) =>
  runPluginOperation<ExtensionChange<PluginInstallation>>("install_plugin", "extensions.taskInstall", {
    source,
    expectedDigest,
    publication,
  });
export const updatePlugin = (id: string, expectedDigest: string) =>
  runPluginOperation<ExtensionChange<PluginInstallation>>("update_plugin", "extensions.taskUpdate", {
    id,
    expectedDigest,
  });
export const setPluginEnabled = (id: string, enabled: boolean) =>
  invokeCommand<void>("plugin:extensions|set_plugin_enabled", { id, enabled });
export const uninstallPlugin = (id: string) =>
  invokeCommand<ExtensionChange<null>>("plugin:extensions|uninstall_plugin", { id });
export const searchPlugins = (query: string) =>
  invokeCommand<PluginMarketplace[]>("plugin:extensions|search_plugins", { query });
export const addPluginMarketplace = (source: MarketplaceSource) =>
  runPluginOperation<PluginMarketplace>("add_plugin_marketplace", "extensions.taskMarketplace", { source });
export const refreshPluginMarketplace = (id: string) =>
  runPluginOperation<PluginMarketplace>("refresh_plugin_marketplace", "extensions.taskMarketplace", { id });
export const removePluginMarketplace = (id: string) =>
  invokeCommand<void>("plugin:extensions|remove_plugin_marketplace", { id });

export const inspectPluginUpdate = (id: string) =>
  runPluginOperation<PluginUpdatePreview>("inspect_plugin_update", "extensions.taskInspect", { id });
