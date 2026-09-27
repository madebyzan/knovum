<script setup lang="ts">
import { Play } from "@lucide/vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { openSpaceNode } from "../../canvas/selection-actions";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { isSpaceManagedAssetPayload } from "../../model/nodes/capabilities";
import { truncateUtf16 } from "../../model/text";
import { useSpaceStore } from "../../session/space-store";
import SpaceVisualAnalysis from "./SpaceVisualAnalysis.vue";

const props = defineProps<{ node: SpaceCanvasNode }>();
const { t } = useI18n();
const store = useSpaceStore();
const preview = computed(() => {
  const data = props.node.data;
  return isSpaceManagedAssetPayload(data) && data.previewPath ? convertFileSrc(data.previewPath) : undefined;
});
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

async function play() {
  try {
    await openSpaceNode(store, props.node);
  } catch (error) {
    toast.error(String(error));
  }
}
</script>

<template>
  <section v-if="preview || visual || excerpt" class="px-4 pb-4">
    <div
      v-if="preview"
      class="relative overflow-hidden rounded-section-inner border border-stroke-divider bg-surface-control"
    >
      <img :src="preview" alt="" class="h-40 w-full object-contain" />
      <button
        v-if="node.data.kind === 'video'"
        type="button"
        class="absolute inset-0 grid place-items-center"
        :aria-label="t('spaces.inspector.playVideo')"
        @click="play"
      >
        <span class="grid size-9 place-items-center rounded-full bg-surface-inverse/80 text-fg-inverse">
          <Play class="size-3.5 fill-current" />
        </span>
      </button>
      <span
        v-if="duration"
        class="absolute right-2 bottom-2 rounded-md bg-surface-inverse/80 px-1.5 py-0.5 text-caption-sm text-fg-inverse"
        >{{ duration }}</span
      >
    </div>
    <div v-if="visual" class="mt-4">
      <h3 class="mb-2 text-caption-md font-semibold text-fg-primary">
        {{ $t("spaces.inspector.content") }}
      </h3>
      <SpaceVisualAnalysis :node="node" />
    </div>
    <div v-if="excerpt" class="mt-4">
      <h3 class="mb-2 text-caption-md font-semibold text-fg-primary">
        {{ $t("spaces.inspector.excerpts.content") }}
      </h3>
      <p class="line-clamp-4 text-body-sm leading-relaxed whitespace-pre-wrap text-fg-secondary select-text">
        {{ excerpt }}
      </p>
    </div>
  </section>
</template>
