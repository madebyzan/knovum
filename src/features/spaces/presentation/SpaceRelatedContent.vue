<script setup lang="ts">
import { LocateFixed, Search, TriangleAlert } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, shallowRef, toRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiSpinner from "../../../shared/ui/UiSpinner.vue";
import { spaceCanvasOverviewSource } from "../assets/node-preview-source";
import { selectAndRevealSpaceNode } from "../canvas/selection-actions";
import { scheduleKnowledgeIndex } from "../knowledge/indexing/index-scheduler";
import { useRelatedContent } from "../knowledge/queries/use-related-content";
import { useSpaceStore } from "../session/space-store";
import { nodeIcon } from "./node-icon";

const props = defineProps<{ nodeId: string }>();
const { t } = useI18n();
const store = useSpaceStore();
const { activeSpaceId, nodes } = storeToRefs(store);
const nodeId = toRef(props, "nodeId");
const { query, indexStatus } = useRelatedContent(activeSpaceId, nodeId);
const expanded = shallowRef(false);
const currentNodes = computed(() => new Map(nodes.value.map((node) => [node.id, node])));
const items = computed(() =>
  (query.data.value?.items ?? []).flatMap((item) => {
    const node = currentNodes.value.get(item.nodeId);
    if (!node) return [];
    const data = node.data;
    return [
      {
        nodeId: item.nodeId,
        nodeTitle: data.title,
        nodeKind: data.kind,
        thumbnail: spaceCanvasOverviewSource(data),
      },
    ];
  }),
);
const visibleItems = computed(() => (expanded.value ? items.value : items.value.slice(0, 3)));
const pending = computed(
  () =>
    query.isPending.value ||
    (query.data.value?.state === "unindexed" &&
      (indexStatus.value === "scheduled" || indexStatus.value === "indexing")),
);
watch(nodeId, () => (expanded.value = false));

function reveal(id: string) {
  selectAndRevealSpaceNode(id);
}

function startIndex() {
  if (activeSpaceId.value) scheduleKnowledgeIndex(activeSpaceId.value, [props.nodeId]);
}

function retry() {
  if (indexStatus.value === "unavailable" && query.data.value?.state === "unindexed") {
    startIndex();
  } else {
    void query.refetch();
  }
}
</script>

<template>
  <section class="mx-4 border-t border-stroke-divider py-1.5">
    <header class="flex min-h-interaction items-center gap-2 px-1">
      <h3 class="flex-1 text-label font-medium text-fg-primary">
        {{ t("spaces.inspector.related.title") }}
      </h3>
      <button
        v-if="items.length > 3"
        type="button"
        class="rounded-md px-1 text-caption text-fg-secondary hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
        :aria-expanded="expanded"
        @click="expanded = !expanded"
      >
        {{ t(expanded ? "spaces.inspector.related.showLess" : "spaces.inspector.related.show") }}
      </button>
    </header>
    <div
      v-if="query.isError.value || (indexStatus === 'unavailable' && query.data.value?.state === 'unindexed')"
      class="flex items-start gap-2 px-1 py-2 text-caption text-fg-danger"
    >
      <TriangleAlert class="mt-0.5 size-3 shrink-0" />
      <div>
        <p>{{ t("spaces.inspector.related.unavailable") }}</p>
        <button type="button" class="mt-1 underline" @click="retry">{{ t("common.retry") }}</button>
      </div>
    </div>
    <div v-else-if="pending" class="flex items-center gap-2 px-1 py-2 text-caption text-fg-tertiary" role="status">
      <UiSpinner size="sm" />{{ t("spaces.inspector.related.indexing") }}
    </div>
    <div
      v-else-if="query.data.value?.state === 'unindexed'"
      class="px-1 py-2 text-caption leading-relaxed text-fg-tertiary"
    >
      <p>{{ t("spaces.inspector.related.notIndexed") }}</p>
      <button type="button" class="mt-1 text-fg-secondary underline" @click="startIndex">
        {{ t("spaces.inspector.related.prepare") }}
      </button>
    </div>
    <p
      v-else-if="query.data.value?.state === 'no_content'"
      class="px-1 py-2 text-caption leading-relaxed text-fg-tertiary"
    >
      {{ t("spaces.inspector.related.noContent") }}
    </p>
    <div v-else-if="!items.length" class="flex items-center gap-2 px-1 py-2 text-caption text-fg-tertiary">
      <Search class="size-3.5" :stroke-width="1.5" />{{ t("spaces.inspector.related.empty") }}
    </div>
    <div v-else>
      <p v-if="expanded" class="px-1 pb-1 text-caption leading-relaxed text-fg-tertiary">
        {{ t("spaces.inspector.related.explanation") }}
      </p>
      <button
        v-for="item in visibleItems"
        :key="item.nodeId"
        type="button"
        class="flex w-full items-center gap-2.5 rounded-md px-1 py-1.5 text-left transition-colors hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus"
        :aria-label="`${t('spaces.canvas.revealInCanvas')}: ${item.nodeTitle}`"
        @click="reveal(item.nodeId)"
      >
        <span class="grid size-9 shrink-0 place-items-center overflow-hidden rounded-md bg-surface-control text-fg-tertiary">
          <img v-if="item.thumbnail" :src="item.thumbnail" alt="" class="size-full object-contain" loading="lazy" decoding="async" />
          <component :is="nodeIcon(item.nodeKind)" v-else class="size-4" :stroke-width="1.5" />
        </span>
        <span class="min-w-0 flex-1">
          <span class="block truncate text-label font-medium text-fg-primary">{{ item.nodeTitle }}</span>
        </span>
        <LocateFixed class="size-3.5 shrink-0 text-fg-tertiary" :stroke-width="1.6" />
      </button>
    </div>
  </section>
</template>
