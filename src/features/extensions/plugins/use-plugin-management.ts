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

export function usePluginManagement(onInspected: () => void, onInstalled: () => void) {
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
      onInspected();
    },
  });
  function inspect(
    source: PluginSource,
    previous?: PluginInstallation,
    publication: PluginPublication | null = null,
  ) {
    inspecting.mutate({ source, ...(previous ? { previous } : {}), publication });
  }
  function inspectRelease(source: PluginSource, publication: PluginPublication) {
    inspect(source, undefined, publication);
  }
  async function commit() {
    const reviewed = preview.value;
    if (!reviewed) return;
    await changes
      .mutateAsync(async () => {
        const change = reviewed.previous
          ? await updatePlugin(reviewed.previous.id, reviewed.inspection.digest)
          : await installPlugin(reviewed.source, reviewed.inspection.digest, reviewed.publication);
        preview.value = undefined;
        onInstalled();
        return change;
      })
      .catch(() => {});
  }
  function mutate(operation: () => Promise<ExtensionChange<unknown> | void>) {
    void changes.mutateAsync(operation).catch(() => {});
  }
  async function remove(plugin: PluginInstallation) {
    if (
      await confirmAction({
        title: t("extensions.uninstallNamed", { name: plugin.revision.manifest.name }),
        description: t("extensions.uninstallDescription"),
        confirmLabel: t("extensions.uninstall"),
        danger: true,
      })
    )
      mutate(() => uninstallPlugin(plugin.id));
  }
  const busy = computed(() => changes.isPending.value || inspecting.isPending.value);

  return { query, changes, inspecting, preview, busy, inspect, inspectRelease, commit, mutate, remove };
}
