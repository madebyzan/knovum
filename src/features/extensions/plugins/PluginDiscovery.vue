<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { computed, shallowRef } from "vue";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { marketplaceQuery } from "../../../foundation/extensions/extension-query";
import {
  addPluginMarketplace,
  refreshPluginMarketplace,
  removePluginMarketplace,
} from "../../../foundation/extensions/plugin-operations";
import type { MarketplaceSource } from "../../../shared/contracts/generated/MarketplaceSource";
import type { PluginPublication } from "../../../shared/contracts/generated/PluginPublication";
import type { PluginSource } from "../../../shared/contracts/generated/PluginSource";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import { useExtensionChanges } from "../use-extension-changes";
import MarketplaceSourceForm from "./MarketplaceSourceForm.vue";
defineProps<{ busy: boolean }>();
const emit = defineEmits<{ inspect: [source: PluginSource, publication: PluginPublication] }>();
const query = useQuery(marketplaceQuery);
const changes = useExtensionChanges();
const search = shallowRef("");
const sourceId = shallowRef("");
const releases = computed(() =>
  (query.data.value ?? [])
    .filter((source) => !sourceId.value || source.id === sourceId.value)
    .flatMap((source) =>
      source.releases.map((release) => ({ sourceId: source.id, sourceName: source.displayName, release })),
    )
    .filter(({ release }) =>
      `${release.name} ${release.publisherId} ${release.description}`
        .toLowerCase()
        .includes(search.value.toLowerCase()),
    ),
);
function mutate(operation: () => Promise<unknown>) {
  void changes
    .mutateAsync(async () => {
      await operation();
    })
    .catch(() => {});
}
function add(source: MarketplaceSource) {
  mutate(() => addPluginMarketplace(source));
}
</script>

<template>
  <section class="space-y-3 rounded-sm border border-stroke-divider p-3">
    <h2 class="text-body font-medium">{{ $t("extensions.discovery") }}</h2>
    <MarketplaceSourceForm :busy="changes.isPending.value" @add="add" />
    <div v-for="source in query.data.value" :key="source.id" class="flex items-center gap-2 text-caption">
      <span class="min-w-0 flex-1 truncate">{{ source.displayName }} · {{ source.id }}</span>
      <UiButton
        size="sm"
        variant="ghost"
        :disabled="changes.isPending.value"
        @click="mutate(() => refreshPluginMarketplace(source.id))"
        >{{ $t("extensions.refresh") }}</UiButton
      >
      <UiButton
        size="sm"
        variant="dangerSubtle"
        :disabled="changes.isPending.value"
        @click="mutate(() => removePluginMarketplace(source.id))"
        >{{ $t("extensions.remove") }}</UiButton
      >
    </div>
    <p v-if="query.error.value || changes.error.value" role="alert" class="text-caption text-fg-danger">
      {{ getErrorMessage(query.error.value || changes.error.value) }}
    </p>
    <div class="flex gap-2">
      <label class="flex-1 text-caption"
        >{{ $t("extensions.search") }}<UiInput v-model="search" data-testid="plugin-search"
      /></label>
      <label class="text-caption"
        >{{ $t("extensions.source")
        }}<select v-model="sourceId" class="block rounded-sm bg-surface-control p-2">
          <option value="">{{ $t("extensions.allSources") }}</option>
          <option v-for="source in query.data.value" :key="source.id" :value="source.id">
            {{ source.displayName }}
          </option>
        </select></label
      >
    </div>
    <div
      v-for="item in releases"
      :key="`${item.sourceId}:${item.release.publisherId}:${item.release.name}`"
      class="flex items-center gap-3 border-t border-stroke-divider py-2"
    >
      <div class="min-w-0 flex-1">
        <h3 class="text-body">{{ item.release.name }}</h3>
        <p class="text-caption text-fg-secondary">
          {{ item.sourceName }} · {{ item.release.publisherId }} · {{ $t("extensions.publisherUnverified") }}
        </p>
        <p class="text-caption text-fg-secondary">{{ item.release.description }}</p>
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
        >{{ $t("extensions.preview") }}</UiButton
      >
    </div>
  </section>
</template>
