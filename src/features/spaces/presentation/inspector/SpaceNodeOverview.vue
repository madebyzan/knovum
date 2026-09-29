<script setup lang="ts">
import { Play } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { spaceCanvasOverviewSource } from "../../assets/node-preview-source";
import { openSpaceNode } from "../../canvas/selection-actions";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { truncateUtf16 } from "../../model/text";
import { useSpaceStore } from "../../session/space-store";
import SpaceVisualAnalysis from "./SpaceVisualAnalysis.vue";

const props = defineProps<{ node: SpaceCanvasNode }>();
const { t } = useI18n();
const store = useSpaceStore();
const preview = computed(() => spaceCanvasOverviewSource(props.node.data));
const visual = computed(() => props.node.data.kind === "image" || props.node.data.kind === "video");
const excerpt = computed(() => {
  const data = props.node.data;
  const text = data.kind === "file" ? data.previewText?.trim() : undefined;
  return text ? truncateUtf16(text, 400) : undefined;
});
const duration = computed(() => {
  const data = props.node.data;
  if (data.kind !== "video" || data.durationSeconds === undefined) return undefined;
  const minutes = Math.floor(data.durationSeconds / 60);
  const seconds = data.durationSeconds % 60;
  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
});

async function openPreview() {
  try {
    await openSpaceNode(store, props.node);
  } catch (error) {
    toast.error(String(error));
  }
}
</script>

<template>
  <section v-if="preview || visual || excerpt" class="px-4 pb-3">
    <div
      v-if="preview"
      class="relative overflow-hidden rounded-section-inner border border-stroke-divider bg-surface-control"
    >
      <button
        v-if="visual"
        type="button"
        class="group/preview relative block w-full cursor-pointer focus-visible:outline-2 focus-visible:outline-stroke-focus"
        :aria-label="t(node.data.kind === 'video' ? 'spaces.inspector.playVideo' : 'spaces.inspector.openContent')"
        @click="openPreview"
      >
        <img :src="preview" alt="" class="h-44 w-full object-contain transition-opacity duration-100 group-hover/preview:opacity-90" />
        <span v-if="node.data.kind === 'video'" class="absolute inset-0 grid place-items-center">
          <span class="grid size-9 place-items-center rounded-full bg-surface-inverse/80 text-fg-inverse">
            <Play class="size-3.5 fill-current" />
          </span>
        </span>
      </button>
      <img v-else :src="preview" alt="" class="h-44 w-full object-contain" />
      <span
        v-if="duration"
        class="absolute right-2 bottom-2 rounded-md bg-surface-inverse/80 px-1.5 py-0.5 text-caption-sm text-fg-inverse"
        >{{ duration }}</span
      >
    </div>
    <div v-if="visual" class="mt-4">
      <h3 class="mb-1.5 text-label font-medium text-fg-primary">
        {{ $t("spaces.inspector.content") }}
      </h3>
      <SpaceVisualAnalysis :node="node" />
    </div>
    <div v-if="excerpt" class="mt-4">
      <h3 class="mb-1.5 text-label font-medium text-fg-primary">
        {{ $t("spaces.inspector.excerpts.content") }}
      </h3>
      <p class="line-clamp-4 text-body leading-relaxed whitespace-pre-wrap text-fg-secondary select-text">
        {{ excerpt }}
      </p>
    </div>
  </section>
</template>
