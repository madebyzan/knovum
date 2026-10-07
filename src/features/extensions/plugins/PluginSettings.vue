<script setup lang="ts">
import { Compass, FolderInput, ArrowLeft } from "@lucide/vue";
import { computed, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { setPluginEnabled } from "../../../foundation/extensions/plugin-operations";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiDialog from "../../../shared/ui/UiDialog.vue";
import UiSearchInput from "../../../shared/ui/UiSearchInput.vue";
import UiSettingsSection from "../../../shared/ui/UiSettingsSection.vue";
import UiSpinner from "../../../shared/ui/UiSpinner.vue";
import ExtensionCleanupNotice from "../ExtensionCleanupNotice.vue";
import InstalledPlugin from "./InstalledPlugin.vue";
import PluginDiscovery from "./PluginDiscovery.vue";
import PluginInstallationPreview from "./PluginInstallationPreview.vue";
import PluginRevisionChanges from "./PluginRevisionChanges.vue";
import PluginSourceInput from "./PluginSourceInput.vue";
import { usePluginManagement } from "./use-plugin-management";
const discovering = shallowRef(false);
const sourceOpen = shallowRef(false);
const search = shallowRef("");
const { query, changes, inspecting, preview, busy, inspect, inspectRelease, commit, mutate, remove } =
  usePluginManagement(
    () => {
      sourceOpen.value = false;
    },
    () => {
      discovering.value = false;
      search.value = "";
    },
  );
const plugins = computed(() =>
  (query.data.value ?? []).filter((plugin) =>
    `${plugin.revision.manifest.name} ${plugin.revision.manifest.description ?? ""}`
      .toLowerCase()
      .includes(search.value.trim().toLowerCase()),
  ),
);
</script>

<template>
  <div class="min-w-0 space-y-4">
    <header class="flex flex-wrap items-end justify-between gap-3 px-0.5">
      <div class="min-w-0 flex-1 basis-48">
        <UiButton
          v-if="discovering"
          size="sm"
          variant="ghost"
          class="mb-2 -ml-2.5"
          data-testid="back-to-installed-plugins"
          @click="discovering = false"
          ><ArrowLeft class="size-3.5" />{{ $t("extensions.installedPlugins") }}</UiButton
        >
        <h2 class="text-title-sm font-semibold tracking-title text-fg-primary">
          {{ $t(discovering ? "extensions.discovery" : "extensions.installedPlugins") }}
        </h2>
        <p class="mt-1 text-caption text-fg-secondary">
          {{ $t(discovering ? "extensions.discoveryDescription" : "extensions.pluginsPurpose") }}
        </p>
      </div>
      <div class="flex flex-wrap gap-2">
        <UiButton
          :disabled="busy"
          data-testid="open-plugin-source"
          @click="
            inspecting.reset();
            sourceOpen = true;
          "
          ><FolderInput class="size-4" />{{ $t("extensions.installFromSource") }}</UiButton
        >
        <UiButton
          v-if="!discovering"
          :disabled="busy"
          data-testid="discover-plugins"
          @click="discovering = true"
          ><Compass class="size-4" />{{ $t("extensions.discovery") }}</UiButton
        >
      </div>
    </header>
    <p
      v-if="query.error.value || changes.error.value || inspecting.error.value"
      role="alert"
      class="text-body text-fg-danger"
    >
      {{ getErrorMessage(query.error.value || changes.error.value || inspecting.error.value) }}
    </p>
    <ExtensionCleanupNotice :errors="changes.data.value?.cleanupErrors ?? []" />
    <PluginDiscovery v-if="discovering" :busy="busy" @inspect="inspectRelease" />
    <template v-else>
      <UiSearchInput
        v-if="query.data.value?.length"
        v-model="search"
        :label="$t('extensions.searchPlugins')"
        :placeholder="$t('extensions.searchPlugins')"
        data-testid="installed-plugin-search"
      />
      <div
        v-if="query.isPending.value"
        role="status"
        class="flex items-center justify-center gap-2 py-12 text-body text-fg-secondary"
      >
        <UiSpinner class="size-4" />{{ $t("extensions.loading") }}
      </div>
      <UiSettingsSection v-else-if="!query.error.value && !plugins.length">
        <div class="px-5 py-6">
          <h3 class="text-body-lg font-medium text-fg-primary">
            {{ $t(search ? "extensions.noSearchResults" : "extensions.emptyPlugins") }}
          </h3>
          <p class="mt-1 text-caption text-fg-secondary">
            {{ $t(search ? "extensions.tryAnotherSearch" : "extensions.emptyPluginsHint") }}
          </p>
        </div>
      </UiSettingsSection>
      <UiSettingsSection v-else-if="plugins.length">
        <div class="divide-y divide-stroke-divider">
          <InstalledPlugin
            v-for="plugin in plugins"
            :key="plugin.id"
            :plugin="plugin"
            :busy="busy"
            @toggle="mutate(() => setPluginEnabled(plugin.id, !plugin.enabled))"
            @update="inspect(plugin.source, plugin)"
            @uninstall="remove(plugin)"
          />
        </div>
      </UiSettingsSection>
    </template>
    <UiDialog
      :open="sourceOpen"
      :title="$t('extensions.installFromSource')"
      :description="$t('extensions.sourceDescription')"
      :closable="!busy"
      @update:open="!busy && (sourceOpen = $event)"
    >
      <PluginSourceInput :busy="busy" @inspect="inspect" />
      <p v-if="inspecting.error.value" role="alert" class="mt-3 text-body text-fg-danger">
        {{ getErrorMessage(inspecting.error.value) }}
      </p>
    </UiDialog>
    <UiDialog
      :open="Boolean(preview)"
      :title="$t(preview?.previous ? 'extensions.reviewUpdate' : 'extensions.reviewInstallation')"
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
          @install="commit"
          @cancel="preview = undefined"
        />
        <p v-if="changes.error.value" role="alert" class="mt-3 text-body text-fg-danger">
          {{ getErrorMessage(changes.error.value) }}
        </p>
      </template>
    </UiDialog>
  </div>
</template>
