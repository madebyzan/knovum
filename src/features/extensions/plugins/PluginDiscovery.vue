<script setup lang="ts">
import { Settings2 } from "@lucide/vue";
import { useQuery } from "@tanstack/vue-query";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { marketplaceQuery } from "../../../foundation/extensions/extension-query";
import type { PluginPublication } from "../../../shared/contracts/generated/PluginPublication";
import type { PluginSource } from "../../../shared/contracts/generated/PluginSource";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import UiSpinner from "../../../shared/ui/UiSpinner.vue";
import ExtensionEmptyState from "../components/ExtensionEmptyState.vue";
import MarketplaceSources from "./MarketplaceSources.vue";
const props = defineProps<{ busy: boolean; search: string }>();
const emit = defineEmits<{ inspect: [source: PluginSource, publication: PluginPublication] }>();
const { t } = useI18n();
const sourcesOpen = shallowRef(false);
const query = useQuery(marketplaceQuery);
const sourceId = shallowRef("all");
const sourceOptions = computed(() => [
  { value: "all", label: t("extensions.plugins.catalogs.all") },
  ...(query.data.value ?? []).map((source) => ({ value: source.id, label: source.displayName })),
]);
const releases = computed(() =>
  (query.data.value ?? [])
    .filter((source) => sourceId.value === "all" || source.id === sourceId.value)
    .flatMap((source) =>
      source.releases.map((release) => ({ sourceId: source.id, sourceName: source.displayName, release })),
    )
    .filter(({ release }) =>
      `${release.name} ${release.publisherId} ${release.description}`
        .toLowerCase()
        .includes(props.search.trim().toLowerCase()),
    ),
);
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
      <UiButton :disabled="busy" data-testid="manage-marketplaces" @click="sourcesOpen = true"
        ><Settings2 class="size-4" />{{ $t("extensions.plugins.catalogs.title") }}</UiButton
      >
    </div>
    <p v-if="query.error.value" role="alert" class="text-body text-fg-danger">
      {{ getErrorMessage(query.error.value) }}
    </p>
    <div
      v-if="query.isPending.value"
      role="status"
      class="flex items-center justify-center gap-2 py-12 text-body text-fg-secondary"
    >
      <UiSpinner class="size-4" />{{ $t("common.loading") }}
    </div>
    <ExtensionEmptyState
      v-else-if="!query.error.value && !releases.length"
      :title="
        $t(
          search || sourceId !== 'all'
            ? 'extensions.plugins.search.empty'
            : 'extensions.plugins.catalogs.empty.title',
        )
      "
      :description="
        $t(
          query.data.value?.length
            ? 'extensions.plugins.search.hint'
            : 'extensions.plugins.catalogs.empty.description',
        )
      "
    >
      <UiButton
        v-if="!query.data.value?.length"
        class="mt-4"
        size="sm"
        data-testid="add-first-marketplace"
        @click="sourcesOpen = true"
        >{{ $t("extensions.plugins.catalogs.add") }}</UiButton
      >
    </ExtensionEmptyState>
    <div v-else-if="releases.length">
      <div class="divide-y divide-stroke-divider">
        <div
          v-for="item in releases"
          :key="`${item.sourceId}:${item.release.publisherId}:${item.release.name}`"
          class="flex min-h-(--knovum-setting-min-height) items-start gap-3 px-4 py-3"
        >
          <div class="min-w-0 flex-1">
            <h3 class="text-body-lg font-medium break-words text-fg-primary">{{ item.release.name }}</h3>
            <p class="mt-0.5 text-caption leading-normal text-fg-secondary">{{ item.release.description }}</p>
            <p class="mt-1.5 text-caption break-words text-fg-tertiary">
              {{ item.sourceName }} · {{ item.release.publisherId }}
            </p>
            <p class="mt-0.5 text-caption text-fg-tertiary">
              {{ $t("extensions.plugins.verification.unverified") }}
            </p>
          </div>
          <UiButton
            size="sm"
            :disabled="busy"
            :data-testid="`preview-plugin-${item.release.name}`"
            @click="
              emit('inspect', item.release.source, {
                marketplaceId: item.sourceId,
                publisherId: item.release.publisherId,
                pluginName: item.release.name,
              })
            "
            >{{ $t("extensions.plugins.installation.preview") }}</UiButton
          >
        </div>
      </div>
    </div>
    <MarketplaceSources v-model:open="sourcesOpen" @removed="sourceId === $event && (sourceId = 'all')" />
  </section>
</template>
