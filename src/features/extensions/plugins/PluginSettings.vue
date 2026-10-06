<script setup lang="ts">
import { useQuery, useMutation } from "@tanstack/vue-query";
import { shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { pluginQuery } from "../../../foundation/extensions/extension-query";
import {
  inspectPluginSource,
  inspectPluginUpdate,
  installPlugin,
  updatePlugin,
  setPluginEnabled,
  uninstallPlugin,
} from "../../../foundation/extensions/plugin-operations";
import type { ExtensionChange } from "../../../shared/contracts/generated/ExtensionChange";
import type { PluginInspection } from "../../../shared/contracts/generated/PluginInspection";
import type { PluginInstallation } from "../../../shared/contracts/generated/PluginInstallation";
import type { PluginPublication } from "../../../shared/contracts/generated/PluginPublication";
import type { PluginSource } from "../../../shared/contracts/generated/PluginSource";
import UiButton from "../../../shared/ui/UiButton.vue";
import ExtensionCleanupNotice from "../ExtensionCleanupNotice.vue";
import { useExtensionChanges } from "../use-extension-changes";
import PluginDiscovery from "./PluginDiscovery.vue";
import PluginInstallationPreview from "./PluginInstallationPreview.vue";
import PluginRevisionChanges from "./PluginRevisionChanges.vue";
import PluginSourceInput from "./PluginSourceInput.vue";
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
      const result = reviewed.previous
        ? await updatePlugin(reviewed.previous.id, reviewed.inspection.digest)
        : await installPlugin(reviewed.source, reviewed.inspection.digest, reviewed.publication);
      preview.value = undefined;
      return result;
    })
    .catch(() => {});
}
function mutate(operation: () => Promise<ExtensionChange<unknown> | void>) {
  void changes.mutateAsync(operation).catch(() => {});
}
</script>

<template>
  <div class="space-y-4">
    <PluginDiscovery
      :busy="changes.isPending.value || inspecting.isPending.value"
      @inspect="inspectRelease"
    />
    <PluginSourceInput :busy="changes.isPending.value || inspecting.isPending.value" @inspect="inspect" />
    <p
      v-if="query.error.value || changes.error.value || inspecting.error.value"
      role="alert"
      class="text-body text-fg-danger"
    >
      {{ getErrorMessage(query.error.value || changes.error.value || inspecting.error.value) }}
    </p>
    <ExtensionCleanupNotice :errors="changes.data.value?.cleanupErrors ?? []" />
    <template v-if="preview">
      <PluginRevisionChanges
        v-if="preview.previous"
        :before="preview.previous.revision"
        :after="preview.inspection"
      />
      <PluginInstallationPreview
        :inspection="preview.inspection"
        :busy="changes.isPending.value"
        :updating="Boolean(preview.previous)"
        @install="commit"
        @cancel="preview = undefined"
      />
    </template>
    <p v-if="query.isPending.value" class="text-body text-fg-secondary">{{ $t("extensions.loading") }}</p>
    <p v-else-if="!query.error.value && !query.data.value?.length" class="text-caption text-fg-secondary">
      {{ $t("extensions.emptyPlugins") }}
    </p>
    <section
      v-for="plugin in query.data.value"
      :key="plugin.id"
      class="space-y-2 border-b border-stroke-divider py-3"
      :data-testid="`plugin-${plugin.id}`"
    >
      <h2 class="text-body font-medium">
        {{ plugin.revision.manifest.name }} {{ plugin.revision.manifest.version }}
      </h2>
      <p class="text-caption text-fg-secondary">
        {{ plugin.builtin ? $t("extensions.builtin") : $t("extensions.publisherUnverified") }}
      </p>
      <p class="text-caption break-all text-fg-tertiary">
        {{ plugin.source }} · {{ plugin.publication?.publisherId }} · {{ plugin.revision.sourceRevision }}
      </p>
      <div class="flex gap-2">
        <UiButton
          size="sm"
          :disabled="changes.isPending.value"
          :data-testid="`toggle-plugin-${plugin.id}`"
          @click="mutate(() => setPluginEnabled(plugin.id, !plugin.enabled))"
          >{{ $t(plugin.enabled ? "extensions.disable" : "extensions.enable") }}</UiButton
        >
        <UiButton
          v-if="!plugin.builtin"
          size="sm"
          :disabled="changes.isPending.value || inspecting.isPending.value"
          :data-testid="`update-plugin-${plugin.id}`"
          @click="inspect(plugin.source, plugin)"
          >{{ $t("extensions.checkUpdate") }}</UiButton
        >
        <UiButton
          v-if="!plugin.builtin"
          size="sm"
          variant="dangerSubtle"
          :disabled="changes.isPending.value"
          :data-testid="`uninstall-plugin-${plugin.id}`"
          @click="mutate(() => uninstallPlugin(plugin.id))"
          >{{ $t("extensions.uninstall") }}</UiButton
        >
      </div>
    </section>
  </div>
</template>
