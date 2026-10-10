<script setup lang="ts">
import { Compass, FolderInput, ArrowLeft } from "@lucide/vue";
import { computed, shallowRef } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { enablePlugin, disablePlugin } from "../../../../foundation/extensions/plugin-operations";
import type { PluginInstallation } from "../../../../shared/contracts/generated/PluginInstallation";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import UiSettingsHeaderActions from "../../../../shared/ui/UiSettingsHeaderActions.vue";
import UiSettingsSection from "../../../../shared/ui/UiSettingsSection.vue";
import UiSpinner from "../../../../shared/ui/UiSpinner.vue";
import ExtensionCleanupNotice from "../../components/ExtensionCleanupNotice.vue";
import ExtensionEmptyState from "../../components/ExtensionEmptyState.vue";
import type { ExtensionDestination } from "../../extension-destination";
import { usePluginInstallations } from "../use-plugin-installations";
import { usePluginLabels } from "../use-plugin-labels";
import InstalledPlugin from "./InstalledPlugin.vue";
import PluginContents from "./PluginContents.vue";
import PluginDiscovery from "./PluginDiscovery.vue";
import PluginReviewDialog from "./PluginReviewDialog.vue";
import PluginSourceInput from "./PluginSourceInput.vue";
const props = withDefaults(
  defineProps<{
    search?: string;
    pluginId?: string | undefined;
    action?: "create" | "browse" | undefined;
  }>(),
  {
    search: "",
  },
);
const emit = defineEmits<{ navigate: [destination: ExtensionDestination] }>();
const isDiscovering = computed(() => props.action === "browse");
const isSourceOpen = shallowRef(false);
const { plugins, changes, inspection, review, isBusy, inspect, inspectRelease, commit, mutate, uninstall } =
  usePluginInstallations();
const { describePlugin } = usePluginLabels();
const visiblePlugins = computed(() =>
  (plugins.data.value ?? []).filter((plugin) => {
    const manifest = plugin.revision.manifest;
    const { name, description } = describePlugin(plugin.source, manifest.name, manifest.description);
    return `${manifest.name} ${name} ${description ?? ""}`
      .toLowerCase()
      .includes(props.search.trim().toLowerCase());
  }),
);
const selectedPlugin = computed(() => plugins.data.value?.find((plugin) => plugin.id === props.pluginId));
async function inspectSource(source: Parameters<typeof inspect>[0]) {
  if (await inspect(source)) isSourceOpen.value = false;
}
async function commitPlugin() {
  if (!(await commit())) return;
  emit("navigate", { tab: "plugins" });
}
async function uninstallPlugin(plugin: PluginInstallation) {
  if ((await uninstall(plugin)) && props.pluginId === plugin.id) emit("navigate", { tab: "plugins" });
}
</script>

<template>
  <div class="min-w-0 space-y-4">
    <UiSettingsHeaderActions v-if="!pluginId">
      <UiButton
        v-if="isDiscovering"
        size="sm"
        variant="ghost"
        data-testid="back-to-installed-plugins"
        @click="emit('navigate', { tab: 'plugins' })"
        ><ArrowLeft class="size-3.5" />{{ $t("extensions.plugins.title") }}</UiButton
      >
      <UiButton
        :disabled="isBusy"
        size="sm"
        data-testid="open-plugin-source"
        @click="
          inspection.reset();
          isSourceOpen = true;
        "
        ><FolderInput class="size-4" />{{ $t("extensions.plugins.source.install") }}</UiButton
      >
      <UiButton
        v-if="!isDiscovering"
        :disabled="isBusy"
        size="sm"
        data-testid="discover-plugins"
        @click="emit('navigate', { tab: 'plugins', action: 'browse' })"
        ><Compass class="size-4" />{{ $t("extensions.plugins.discovery.title") }}</UiButton
      >
    </UiSettingsHeaderActions>
    <p
      v-if="plugins.error.value || changes.error.value || inspection.error.value"
      role="alert"
      class="text-body text-fg-danger"
    >
      {{ getErrorMessage(plugins.error.value || changes.error.value || inspection.error.value) }}
    </p>
    <ExtensionCleanupNotice :errors="changes.data.value?.cleanupErrors ?? []" />
    <PluginContents
      v-if="selectedPlugin"
      :plugin="selectedPlugin"
      :busy="isBusy"
      @navigate="$emit('navigate', $event)"
      @toggle="
        mutate(() =>
          selectedPlugin!.enabled ? disablePlugin(selectedPlugin!.id) : enablePlugin(selectedPlugin!.id),
        )
      "
      @update="inspect(selectedPlugin.source, selectedPlugin)"
      @uninstall="uninstallPlugin(selectedPlugin)"
    />
    <ExtensionEmptyState
      v-else-if="pluginId && !plugins.isPending.value && !plugins.error.value"
      :title="$t('extensions.overview.notFound')"
      :description="$t('extensions.overview.notFoundHint')"
      ><UiButton @click="$emit('navigate', { tab: 'plugins' })">{{
        $t("extensions.overview.back")
      }}</UiButton></ExtensionEmptyState
    >
    <PluginDiscovery
      v-else-if="isDiscovering"
      :busy="isBusy"
      :search="search"
      @inspect="inspectRelease"
      @navigate="emit('navigate', { tab: 'plugins', id: $event })"
    />
    <div
      v-else-if="plugins.isPending.value"
      role="status"
      class="flex items-center justify-center gap-2 py-12 text-body text-fg-secondary"
    >
      <UiSpinner class="size-4" />{{ $t("common.loading") }}
    </div>
    <ExtensionEmptyState
      v-else-if="!plugins.error.value && !visiblePlugins.length"
      :title="$t(search ? 'extensions.plugins.search.empty' : 'extensions.plugins.empty.title')"
      :description="$t(search ? 'extensions.plugins.search.hint' : 'extensions.plugins.empty.description')"
      ><UiButton
        v-if="!search"
        class="mt-4"
        @click="emit('navigate', { tab: 'plugins', action: 'browse' })"
        >{{ $t("extensions.plugins.discovery.title") }}</UiButton
      ></ExtensionEmptyState
    >
    <UiSettingsSection v-else-if="!pluginId">
      <div class="divide-y divide-stroke-divider px-4">
        <InstalledPlugin
          v-for="plugin in visiblePlugins"
          :key="plugin.id"
          :plugin="plugin"
          :busy="isBusy"
          @inspect="$emit('navigate', { tab: 'plugins', id: plugin.id })"
          @toggle="mutate(() => (plugin.enabled ? disablePlugin(plugin.id) : enablePlugin(plugin.id)))"
          @update="inspect(plugin.source, plugin)"
          @uninstall="uninstallPlugin(plugin)"
        />
      </div>
    </UiSettingsSection>
    <UiDialog
      :open="isSourceOpen"
      :title="$t('extensions.plugins.source.install')"
      :description="$t('extensions.plugins.source.description')"
      :closable="!isBusy"
      @update:open="!isBusy && (isSourceOpen = $event)"
    >
      <PluginSourceInput :busy="isBusy" @inspect="inspectSource" />
      <p v-if="inspection.error.value" role="alert" class="mt-3 text-body text-fg-danger">
        {{ getErrorMessage(inspection.error.value) }}
      </p>
    </UiDialog>
    <PluginReviewDialog
      v-model="review"
      :busy="changes.isPending.value"
      :error="changes.error.value"
      @commit="commitPlugin"
    />
  </div>
</template>
