import { useQuery, useMutation } from "@tanstack/vue-query";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { pluginQuery } from "../../../foundation/extensions/extension-query";
import {
  inspectPluginSource,
  inspectPluginUpdate,
  installPlugin,
  updatePlugin,
  uninstallPlugin,
} from "../../../foundation/extensions/plugin-operations";
import type { ExtensionChange } from "../../../shared/contracts/generated/ExtensionChange";
import type { PluginInspection } from "../../../shared/contracts/generated/PluginInspection";
import type { PluginInstallation } from "../../../shared/contracts/generated/PluginInstallation";
import type { PluginPublication } from "../../../shared/contracts/generated/PluginPublication";
import type { PluginSource } from "../../../shared/contracts/generated/PluginSource";
import { confirmAction } from "../../../shared/ui/confirm-action";
import { useExtensionChanges } from "../use-extension-changes";

export function usePluginManagement() {
  const { t } = useI18n();
  const query = useQuery(pluginQuery);
  const changes = useExtensionChanges();
  const preview = shallowRef<{
    source: PluginSource;
    inspection: PluginInspection;
    previous?: PluginInstallation;
    publication: PluginPublication | null;
  }>();
  const inspecting = useMutation({
    mutationFn: async ({
      source,
      previous,
      publication,
    }: {
      source: PluginSource;
      previous?: PluginInstallation;
      publication: PluginPublication | null;
    }) => {
      const inspected = previous
        ? await inspectPluginUpdate(previous.id)
        : { source, inspection: await inspectPluginSource(source) };
      preview.value = { ...inspected, ...(previous ? { previous } : {}), publication };
    },
  });
  async function inspect(
    source: PluginSource,
    previous?: PluginInstallation,
    publication: PluginPublication | null = null,
  ) {
    return inspecting
      .mutateAsync({ source, ...(previous ? { previous } : {}), publication })
      .then(() => true)
      .catch(() => false);
  }
  function inspectRelease(source: PluginSource, publication: PluginPublication) {
    return inspect(source, undefined, publication);
  }
  async function commit() {
    const reviewed = preview.value;
    if (!reviewed) return false;
    return changes
      .mutateAsync(() =>
        reviewed.previous
          ? updatePlugin(reviewed.previous.id, reviewed.inspection.digest)
          : installPlugin(reviewed.source, reviewed.inspection.digest, reviewed.publication),
      )
      .then(() => {
        preview.value = undefined;
        return true;
      })
      .catch(() => false);
  }
  function mutate(operation: () => Promise<ExtensionChange<unknown> | void>) {
    void changes.mutateAsync(operation).catch(() => {});
  }
  async function uninstall(plugin: PluginInstallation) {
    if (
      await confirmAction({
        title: t("extensions.plugins.uninstall.title", { name: plugin.revision.manifest.name }),
        description: t("extensions.plugins.uninstall.description"),
        confirmLabel: t("extensions.plugins.uninstall.action"),
        danger: true,
      })
    )
      mutate(() => uninstallPlugin(plugin.id));
  }
  const busy = computed(() => changes.isPending.value || inspecting.isPending.value);

  return { query, changes, inspecting, preview, busy, inspect, inspectRelease, commit, mutate, uninstall };
}
