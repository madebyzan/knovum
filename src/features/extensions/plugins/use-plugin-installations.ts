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
import { usePluginLabels } from "./use-plugin-labels";

export interface PluginInstallationProposal {
  source: PluginSource;
  inspection: PluginInspection;
  previous?: PluginInstallation;
  publication: PluginPublication | null;
}

type PluginInspectionRequest = Omit<PluginInstallationProposal, "inspection">;

export function usePluginInstallations() {
  const { t } = useI18n();
  const plugins = useQuery(pluginQuery);
  const { describePlugin } = usePluginLabels();
  const changes = useExtensionChanges();
  const review = shallowRef<PluginInstallationProposal>();
  const inspection = useMutation({
    mutationFn: async ({ source, previous, publication }: PluginInspectionRequest) => {
      const inspected = previous
        ? await inspectPluginUpdate(previous.id)
        : { source, inspection: await inspectPluginSource(source) };
      review.value = { ...inspected, ...(previous ? { previous } : {}), publication };
    },
  });
  async function inspect(
    source: PluginSource,
    previous?: PluginInstallation,
    publication: PluginPublication | null = null,
  ) {
    return inspection
      .mutateAsync({ source, ...(previous ? { previous } : {}), publication })
      .then(() => true)
      .catch(() => false);
  }
  function inspectRelease(source: PluginSource, publication: PluginPublication) {
    return inspect(source, undefined, publication);
  }
  async function commit() {
    const reviewed = review.value;
    if (!reviewed) return false;
    return changes
      .mutateAsync(() =>
        reviewed.previous
          ? updatePlugin(reviewed.previous.id, reviewed.inspection.digest)
          : installPlugin(reviewed.source, reviewed.inspection.digest, reviewed.publication),
      )
      .then(() => {
        review.value = undefined;
        return true;
      })
      .catch(() => false);
  }
  function mutate(operation: () => Promise<ExtensionChange<unknown> | void>) {
    void changes.mutateAsync(operation).catch(() => {});
  }
  async function uninstall(plugin: PluginInstallation) {
    const label = describePlugin(plugin.source, plugin.revision.manifest.name);
    const isConfirmed = await confirmAction({
      title: t("extensions.plugins.uninstall.title", { name: label.name }),
      description: t("extensions.plugins.uninstall.description"),
      confirmLabel: t("extensions.plugins.uninstall.action"),
      danger: true,
    });
    if (!isConfirmed) return false;
    return changes
      .mutateAsync(() => uninstallPlugin(plugin.id))
      .then(
        () => true,
        () => false,
      );
  }
  const isBusy = computed(() => changes.isPending.value || inspection.isPending.value);

  return {
    plugins,
    changes,
    inspection,
    review,
    isBusy,
    inspect,
    inspectRelease,
    commit,
    mutate,
    uninstall,
  };
}
