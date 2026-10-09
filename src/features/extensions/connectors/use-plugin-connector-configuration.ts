import { useQuery } from "@tanstack/vue-query";
import { open } from "@tauri-apps/plugin-dialog";
import { computed, onScopeDispose, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { configurePluginConnector } from "../../../foundation/extensions/connector-operations";
import { pluginQuery } from "../../../foundation/extensions/extension-query";
import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import type { PluginConnectorConfiguration } from "../../../shared/contracts/generated/PluginConnectorConfiguration";
import { useSettingsLeaveContext } from "../../../shared/contracts/settings-leave";
import { confirmAction } from "../../../shared/ui/confirm-action";
import { useExtensionChanges } from "../use-extension-changes";

interface DeclaredTransport {
  type: string;
  command?: string;
  args?: string[];
  url?: string;
}
export function usePluginConnectorConfiguration(connector: () => ConnectorDefinition) {
  const { t } = useI18n();
  const plugins = useQuery(pluginQuery);
  const changes = useExtensionChanges();
  const localNetwork = shallowRef(false);
  const baselineNetwork = shallowRef(false);
  const hasConflict = shallowRef(false);
  const dirty = computed(() => localNetwork.value !== baselineNetwork.value);
  const configuration = computed(
    () =>
      plugins.data.value
        ?.find((plugin) => plugin.id === connector().pluginId)
        ?.revision.connectors.find((component) => component.componentName === connector().componentName)
        ?.configuration as DeclaredTransport | undefined,
  );
  function reset() {
    const endpoint = connector().endpoint;
    localNetwork.value = endpoint?.type === "streamable_http" && endpoint.allowLocalNetwork;
    baselineNetwork.value = localNetwork.value;
    hasConflict.value = false;
  }
  watch(
    () => connector().revision,
    () => {
      if (dirty.value) hasConflict.value = true;
      else reset();
    },
    { immediate: true },
  );
  const unregister = useSettingsLeaveContext().register({
    async prepare() {
      if (changes.isPending.value) return false;
      if (!dirty.value) return true;
      return confirmAction({
        title: t("extensions.connectors.discard.title"),
        description: t("extensions.connectors.discard.description"),
        confirmLabel: t("common.discardChanges"),
        danger: true,
      });
    },
    finalize: reset,
  });
  onScopeDispose(unregister);
  function save(selection: PluginConnectorConfiguration) {
    if (changes.isPending.value || hasConflict.value) return;
    const definition = connector();
    void changes
      .mutateAsync(async () => {
        const change = await configurePluginConnector(definition, selection);
        const endpoint = change.value.endpoint;
        localNetwork.value = endpoint?.type === "streamable_http" && endpoint.allowLocalNetwork;
        baselineNetwork.value = localNetwork.value;
        return change;
      })
      .catch(() => {});
  }
  async function chooseExecutable() {
    const definition = connector();
    await changes
      .mutateAsync(async () => {
        const path = await open({ directory: false, multiple: false });
        if (typeof path === "string")
          return configurePluginConnector(definition, { type: "stdio", executable: path });
      })
      .catch(() => {});
  }
  return { plugins, changes, configuration, localNetwork, hasConflict, save, chooseExecutable };
}
