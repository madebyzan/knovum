<script setup lang="ts">
import { Compass, FolderInput, ArrowLeft } from "@lucide/vue";
import { computed, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { enablePlugin, disablePlugin } from "../../../foundation/extensions/plugin-operations";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiDialog from "../../../shared/ui/UiDialog.vue";
import UiSpinner from "../../../shared/ui/UiSpinner.vue";
import ExtensionCleanupNotice from "../components/ExtensionCleanupNotice.vue";
import ExtensionEmptyState from "../components/ExtensionEmptyState.vue";
import type { ExtensionDestination } from "../extension-destination";
import InstalledPlugin from "./InstalledPlugin.vue";
import PluginContents from "./PluginContents.vue";
import PluginDiscovery from "./PluginDiscovery.vue";
import PluginInstallationPreview from "./PluginInstallationPreview.vue";
import PluginRevisionChanges from "./PluginRevisionChanges.vue";
import PluginSourceInput from "./PluginSourceInput.vue";
import { usePluginManagement } from "./use-plugin-management";
const props = withDefaults(defineProps<{ search?: string; selectedId?: string | undefined }>(), {
  search: "",
});
defineEmits<{ navigate: [destination: ExtensionDestination] }>();
const discovering = shallowRef(false);
const sourceOpen = shallowRef(false);
const { query, changes, inspecting, preview, busy, inspect, inspectRelease, commit, mutate, remove } =
  usePluginManagement();
const plugins = computed(() =>
  (query.data.value ?? []).filter((plugin) =>
    `${plugin.revision.manifest.name} ${plugin.revision.manifest.description ?? ""}`
      .toLowerCase()
      .includes(props.search.trim().toLowerCase()),
  ),
);
const selected = computed(() => query.data.value?.find((plugin) => plugin.id === props.selectedId));
async function inspectSource(source: Parameters<typeof inspect>[0]) {
  if (await inspect(source)) sourceOpen.value = false;
}
async function commitPlugin() {
  if (!(await commit())) return;
  discovering.value = false;
}
</script>

<template>
  <div class="min-w-0 space-y-4">
    <header v-if="!selectedId" class="flex flex-wrap justify-end gap-2">
      <UiButton
        v-if="discovering"
        size="sm"
        variant="ghost"
        data-testid="back-to-installed-plugins"
        @click="discovering = false"
        ><ArrowLeft class="size-3.5" />{{ $t("extensions.plugins.title") }}</UiButton
      >
      <UiButton
        :disabled="busy"
        data-testid="open-plugin-source"
        @click="
          inspecting.reset();
          sourceOpen = true;
        "
        ><FolderInput class="size-4" />{{ $t("extensions.plugins.source.install") }}</UiButton
      >
      <UiButton
        v-if="!discovering"
        :disabled="busy"
        data-testid="discover-plugins"
        @click="discovering = true"
        ><Compass class="size-4" />{{ $t("extensions.plugins.discovery.title") }}</UiButton
      >
    </header>
    <p
      v-if="query.error.value || changes.error.value || inspecting.error.value"
      role="alert"
      class="text-body text-fg-danger"
    >
      {{ getErrorMessage(query.error.value || changes.error.value || inspecting.error.value) }}
    </p>
    <ExtensionCleanupNotice :errors="changes.data.value?.cleanupErrors ?? []" />
    <PluginContents
      v-if="selected"
      :plugin="selected"
      :busy="busy"
      @navigate="$emit('navigate', $event)"
      @toggle="mutate(() => (selected!.enabled ? disablePlugin(selected!.id) : enablePlugin(selected!.id)))"
      @update="inspect(selected.source, selected)"
      @uninstall="remove(selected)"
    />
    <ExtensionEmptyState
      v-else-if="selectedId && !query.isPending.value && !query.error.value"
      :title="$t('extensions.overview.notFound')"
      :description="$t('extensions.overview.notFoundHint')"
      ><UiButton @click="$emit('navigate', { tab: 'plugins' })">{{
        $t("extensions.overview.back")
      }}</UiButton></ExtensionEmptyState
    >
    <PluginDiscovery v-else-if="discovering" :busy="busy" :search="search" @inspect="inspectRelease" />
    <div
      v-else-if="query.isPending.value"
      role="status"
      class="flex items-center justify-center gap-2 py-12 text-body text-fg-secondary"
    >
      <UiSpinner class="size-4" />{{ $t("common.loading") }}
    </div>
    <ExtensionEmptyState
      v-else-if="!query.error.value && !plugins.length"
      :title="$t(search ? 'extensions.plugins.search.empty' : 'extensions.plugins.empty.title')"
      :description="$t(search ? 'extensions.plugins.search.hint' : 'extensions.plugins.empty.description')"
    />
    <div v-else-if="!selectedId" class="divide-y divide-stroke-divider">
      <InstalledPlugin
        v-for="plugin in plugins"
        :key="plugin.id"
        :plugin="plugin"
        :busy="busy"
        @inspect="$emit('navigate', { tab: 'plugins', id: plugin.id })"
        @toggle="mutate(() => (plugin.enabled ? disablePlugin(plugin.id) : enablePlugin(plugin.id)))"
        @update="inspect(plugin.source, plugin)"
        @uninstall="remove(plugin)"
      />
    </div>
    <UiDialog
      :open="sourceOpen"
      :title="$t('extensions.plugins.source.install')"
      :description="$t('extensions.plugins.source.description')"
      :closable="!busy"
      @update:open="!busy && (sourceOpen = $event)"
    >
      <PluginSourceInput :busy="busy" @inspect="inspectSource" />
      <p v-if="inspecting.error.value" role="alert" class="mt-3 text-body text-fg-danger">
        {{ getErrorMessage(inspecting.error.value) }}
      </p>
    </UiDialog>
    <UiDialog
      :open="Boolean(preview)"
      :title="
        $t(preview?.previous ? 'extensions.plugins.update.title' : 'extensions.plugins.installation.title')
      "
      size="wide"
      :closable="!changes.isPending.value"
      @update:open="!changes.isPending.value && (preview = undefined)"
    >
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
          @install="commitPlugin"
          @cancel="preview = undefined"
        />
        <p v-if="changes.error.value" role="alert" class="mt-3 text-body text-fg-danger">
          {{ getErrorMessage(changes.error.value) }}
        </p>
      </template>
    </UiDialog>
  </div>
</template>
