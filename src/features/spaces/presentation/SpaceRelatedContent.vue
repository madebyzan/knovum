<script setup lang="ts">
import { ArrowUpRight, ChevronLeft, Search, TriangleAlert } from "@lucide/vue";
import { storeToRefs } from "pinia";
import { computed, shallowRef, toRef, watch } from "vue";
import { useI18n } from "vue-i18n";

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
    return node ? [{ ...item, nodeTitle: node.data.title, nodeKind: node.data.kind }] : [];
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
  <section class="mx-4 border-t border-stroke-divider py-3">
    <header class="mb-2 flex min-h-(--knovum-control-height-sm) items-center gap-2">
      <button
        v-if="expanded"
        type="button"
        class="grid size-6 place-items-center rounded-md text-fg-secondary hover:bg-surface-control-hover"
        :aria-label="t('common.back')"
        @click="expanded = false"
      >
        <ChevronLeft class="size-3" />
      </button>
      <h3 class="flex-1 text-caption-md font-semibold text-fg-primary">
        {{ t("spaces.inspector.related.title") }}
      </h3>
      <button
        v-if="!expanded && items.length > 3"
        type="button"
        class="text-caption-md text-fg-secondary hover:text-fg-primary"
        @click="expanded = true"
      >
        {{ t("spaces.inspector.related.show") }}
      </button>
    </header>
    <div
      v-if="query.isError.value || (indexStatus === 'unavailable' && query.data.value?.state === 'unindexed')"
      class="flex items-start gap-2 py-3 text-caption-md text-fg-danger"
    >
      <TriangleAlert class="mt-0.5 size-3 shrink-0" />
      <div>
        <p>{{ t("spaces.inspector.related.unavailable") }}</p>
        <button type="button" class="mt-1 underline" @click="retry">{{ t("common.retry") }}</button>
      </div>
    </div>
    <div v-else-if="pending" class="py-3">
      <p class="text-caption-md leading-relaxed text-fg-tertiary">
        {{ t("spaces.inspector.related.indexing") }}
      </p>
      <div class="mt-3 h-2 w-4/5 rounded-md bg-surface-control" />
      <div class="mt-2 h-2 w-full rounded-md bg-surface-control" />
      <div class="mt-2 h-2 w-3/5 rounded-md bg-surface-control" />
    </div>
    <div
      v-else-if="query.data.value?.state === 'unindexed'"
      class="py-3 text-caption-md leading-relaxed text-fg-tertiary"
    >
      <p>{{ t("spaces.inspector.related.notIndexed") }}</p>
      <button type="button" class="mt-1 text-fg-secondary underline" @click="startIndex">
        {{ t("spaces.inspector.related.prepare") }}
      </button>
    </div>
    <p
      v-else-if="query.data.value?.state === 'no_content'"
      class="py-3 text-caption-md leading-relaxed text-fg-tertiary"
    >
      {{ t("spaces.inspector.related.noContent") }}
    </p>
    <div v-else-if="!items.length" class="py-5 text-center text-caption-md text-fg-tertiary">
      <Search class="mx-auto mb-2 size-3.5" :stroke-width="1.5" />
      {{ t("spaces.inspector.related.empty") }}
    </div>
    <div v-else>
      <p v-if="expanded" class="mb-2 text-caption-md leading-relaxed text-fg-tertiary">
        {{ t("spaces.inspector.related.explanation") }}
      </p>
      <button
        v-for="item in visibleItems"
        :key="item.nodeId"
        type="button"
        class="group flex w-full items-start gap-2.5 border-t border-stroke-divider py-2.5 text-left first:border-t-0 hover:bg-surface-control-hover"
        @click="reveal(item.nodeId)"
      >
        <span class="grid size-8 shrink-0 place-items-center rounded-md bg-surface-control text-fg-tertiary">
          <component :is="nodeIcon(item.nodeKind)" class="size-3.5" :stroke-width="1.5" />
        </span>
        <span class="min-w-0 flex-1">
          <span class="block truncate text-caption-md font-medium text-fg-primary">{{ item.nodeTitle }}</span>
          <span class="mt-0.5 line-clamp-2 block text-caption-md leading-snug text-fg-tertiary">{{
            item.snippet
          }}</span>
        </span>
        <ArrowUpRight class="mt-1 size-3 shrink-0 text-fg-tertiary" />
      </button>
    </div>
  </section>
</template>
