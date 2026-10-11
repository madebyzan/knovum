<script setup lang="ts">
import { Package, Settings2 } from "@lucide/vue";
import { useQuery } from "@tanstack/vue-query";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { marketplaceQuery, pluginQuery } from "../../../../foundation/extensions/extension-query";
import type { PluginPublication } from "../../../../shared/contracts/generated/PluginPublication";
import type { PluginSource } from "../../../../shared/contracts/generated/PluginSource";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiSpinner from "../../../../shared/ui/UiSpinner.vue";
import ExtensionEmptyState from "../../components/ExtensionEmptyState.vue";
import ExtensionIdentity from "../../components/ExtensionIdentity.vue";
import { usePluginLabels } from "../use-plugin-labels";
import MarketplaceSources from "./MarketplaceSources.vue";
const props = defineProps<{ busy: boolean; search: string }>();
const emit = defineEmits<{
  inspect: [source: PluginSource, publication: PluginPublication];
  navigate: [pluginId: string];
}>();
const { t } = useI18n();
const isSourcesOpen = shallowRef(false);
const marketplaces = useQuery(marketplaceQuery);
const installations = useQuery(pluginQuery);
const { describePlugin } = usePluginLabels();
const sourceId = shallowRef("all");
const sourceOptions = computed(() => [
  { value: "all", label: t("extensions.plugins.catalogs.all") },
  ...(marketplaces.data.value ?? []).map((source) => ({ value: source.id, label: source.displayName })),
]);
const releases = computed(() =>
  (marketplaces.data.value ?? [])
    .filter((source) => sourceId.value === "all" || source.id === sourceId.value)
    .flatMap((source) =>
      source.releases.map((release) => ({
        sourceId: source.id,
        sourceName: source.displayName,
        release,
        label: describePlugin(release.source, release.name, release.description),
        installation: installations.data.value?.find((plugin) =>
          release.source.type === "bundled"
            ? plugin.source.type === "bundled" && plugin.source.name === release.source.name
            : plugin.publication?.marketplaceId === source.id &&
              plugin.publication.publisherId === release.publisherId &&
              plugin.publication.pluginName === release.name,
        ),
      })),
    )
    .filter(({ release, label }) =>
      `${release.name} ${release.publisherId} ${release.description} ${label.name} ${label.description}`
        .toLowerCase()
        .includes(props.search.trim().toLowerCase()),
    ),
);
function openRelease(item: (typeof releases.value)[number]) {
  if (item.installation) {
    emit("navigate", item.installation.id);
    return;
  }
  emit("inspect", item.release.source, {
    marketplaceId: item.sourceId,
    publisherId: item.release.publisherId,
    pluginName: item.release.name,
  });
}
</script>

<template>
  <section class="min-w-0 space-y-4">
    <div class="flex flex-wrap items-center gap-2">
      <UiSelect
        v-model="sourceId"
        class="w-40"
        :label="$t('extensions.plugins.source.label')"
        :options="sourceOptions"
      />
      <UiButton :disabled="busy" data-testid="manage-marketplaces" @click="isSourcesOpen = true"
        ><Settings2 class="size-4" />{{ $t("extensions.plugins.catalogs.title") }}</UiButton
      >
    </div>
    <p
      v-if="marketplaces.error.value || installations.error.value"
      role="alert"
      class="text-body text-fg-danger"
    >
      {{ getErrorMessage(marketplaces.error.value || installations.error.value) }}
    </p>
    <div
      v-if="marketplaces.isPending.value || installations.isPending.value"
      role="status"
      class="flex items-center justify-center gap-2 py-12 text-body text-fg-secondary"
    >
      <UiSpinner class="size-4" />{{ $t("common.loading") }}
    </div>
    <ExtensionEmptyState
      v-else-if="!marketplaces.error.value && !releases.length"
      :title="
        $t(search || sourceId !== 'all' ? 'common.search.empty' : 'extensions.plugins.catalogs.empty.title')
      "
      :description="
        $t(
          marketplaces.data.value?.length
            ? 'common.search.hint'
            : 'extensions.plugins.catalogs.empty.description',
        )
      "
    >
      <UiButton
        v-if="!marketplaces.data.value?.length"
        class="mt-4"
        size="sm"
        data-testid="add-first-marketplace"
        @click="isSourcesOpen = true"
        >{{ $t("extensions.plugins.catalogs.add") }}</UiButton
      >
    </ExtensionEmptyState>
    <div v-else-if="releases.length">
      <div class="divide-y divide-stroke-divider">
        <ExtensionIdentity
          v-for="item in releases"
          :key="`${item.sourceId}:${item.release.publisherId}:${item.release.name}`"
          class="px-4"
          :icon="Package"
          :name="item.label.name"
          :description="item.label.description"
          :disabled="busy || installations.isPending.value || Boolean(installations.error.value)"
          @inspect="openRelease(item)"
        >
          <template #status>
            <span class="mt-1 block text-caption text-fg-tertiary">
              {{ item.sourceName }} · {{ item.release.publisherId }}
            </span>
            <span
              v-if="item.release.source.type !== 'bundled'"
              class="mt-0.5 block text-caption text-fg-tertiary"
            >
              {{ $t("extensions.plugins.verification.unverified") }}
            </span>
          </template>
          <UiButton
            v-if="item.installation"
            size="sm"
            :data-testid="`installed-plugin-${item.release.name}`"
            @click="emit('navigate', item.installation.id)"
            >{{ $t("extensions.plugins.discovery.installed") }}</UiButton
          >
          <UiButton
            v-else
            size="sm"
            :disabled="busy || installations.isPending.value || Boolean(installations.error.value)"
            :data-testid="`preview-plugin-${item.release.name}`"
            @click="openRelease(item)"
            >{{ $t("extensions.plugins.installation.preview") }}</UiButton
          >
        </ExtensionIdentity>
      </div>
    </div>
    <MarketplaceSources v-model:open="isSourcesOpen" @removed="sourceId === $event && (sourceId = 'all')" />
  </section>
</template>
