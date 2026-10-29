<script setup lang="ts">
import { Film, Image as ImageIcon, Music2, Upload } from "@lucide/vue";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDragGhost from "../../../../shared/ui/UiDragGhost.vue";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import UiSearchInput from "../../../../shared/ui/UiSearchInput.vue";
import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import { usePointerDrag } from "../../../../shared/ui/use-pointer-drag";
import { versionedMediaSource } from "../../media/media-element-source";
import type { VideoMaterialNode } from "../material";
import { formatVideoTimelineTime } from "../timecode";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor }>();
const { t } = useI18n();
const { materials, player, timeline } = props.editor;
const query = shallowRef("");
const ICONS = { video: Film, image: ImageIcon, audio: Music2 } as const;

const visible = computed(() => {
  const needle = query.value.trim().toLocaleLowerCase();
  return materials.nodes.value.filter((node) => node.data.title.toLocaleLowerCase().includes(needle));
});

function cover(node: VideoMaterialNode) {
  if (node.data.kind === "image") return versionedMediaSource(node.data.assetPath, node.data.sourceRevision);
  return node.data.kind === "video" && node.data.previewPath
    ? versionedMediaSource(node.data.previewPath, node.data.sourceRevision)
    : undefined;
}

function durationOf(node: VideoMaterialNode) {
  const seconds = "durationSeconds" in node.data ? node.data.durationSeconds : undefined;
  return typeof seconds === "number" ? formatVideoTimelineTime(seconds * 1000) : undefined;
}

// A material dragged onto the timeline starts where it is dropped; a click adds it at the playhead.
const materialDrag = usePointerDrag({
  targetAt: timeline.timeAtPoint,
  drop: (node: VideoMaterialNode, atMs: number) => materials.add(node, atMs),
});

function add(node: VideoMaterialNode) {
  void materials.add(node, player.positionMs());
}
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col">
    <div class="shrink-0 px-2 py-2">
      <UiSearchInput
        v-model="query"
        size="sm"
        :label="t('spaces.media.videoEditor.materials.search')"
        :placeholder="t('spaces.media.videoEditor.materials.search')"
      />
    </div>
    <UiScrollArea class="min-h-0 flex-1">
      <p v-if="!visible.length" class="px-3 py-8 text-center text-caption text-fg-secondary">
        {{ t("spaces.media.videoEditor.materials.empty") }}
      </p>
      <ul class="grid grid-cols-2 gap-2 p-2">
        <li v-for="node in visible" :key="node.id" class="group relative flex flex-col">
          <UiTooltip :text="node.data.title" side="top" class="max-w-64 break-words">
            <button
              type="button"
              class="flex h-full w-full cursor-pointer flex-col overflow-hidden rounded-lg border border-stroke-default bg-surface-raised text-left transition-colors hover:border-stroke-accent hover:shadow-xs focus-visible:outline-2 focus-visible:outline-stroke-focus"
              :aria-label="t('spaces.media.videoEditor.materials.add', { title: node.data.title })"
              :disabled="materials.busy.value"
              @mousedown="materialDrag.start($event, node)"
              @click="add(node)"
            >
              <span
                v-if="node.data.kind === 'audio'"
                class="from-surface-sunken relative flex aspect-[16/10] w-full items-center justify-center bg-gradient-to-br to-surface-muted"
              >
                <div
                  class="flex size-7 items-center justify-center rounded-full bg-surface-panel/90 text-fg-secondary shadow-xs"
                >
                  <Music2 class="size-3.5" />
                </div>
                <span
                  v-if="durationOf(node)"
                  class="absolute right-1 bottom-1 rounded bg-black/60 px-1 text-caption-sm text-fg-on-image tabular-nums"
                  >{{ durationOf(node) }}</span
                >
              </span>
              <span v-else class="bg-surface-sunken relative block aspect-[16/10] w-full overflow-hidden">
                <img
                  v-if="cover(node)"
                  :src="cover(node)"
                  alt=""
                  draggable="false"
                  class="size-full object-cover"
                />
                <component
                  :is="ICONS[node.data.kind as keyof typeof ICONS]"
                  v-else
                  class="absolute top-1/2 left-1/2 size-5 -translate-x-1/2 -translate-y-1/2 text-fg-tertiary"
                />
                <span
                  v-if="durationOf(node)"
                  class="absolute right-1 bottom-1 rounded bg-black/60 px-1 text-caption-sm text-fg-on-image tabular-nums"
                  >{{ durationOf(node) }}</span
                >
              </span>
              <span
                class="w-full truncate px-1.5 py-1 text-caption-sm text-fg-secondary transition-colors group-hover:text-fg-primary"
                >{{ node.data.title }}</span
              >
            </button>
          </UiTooltip>
        </li>
      </ul>
    </UiScrollArea>
    <div class="shrink-0 border-t border-stroke-divider p-2.5">
      <UiButton
        size="sm"
        class="w-full justify-center gap-1.5"
        :disabled="materials.busy.value"
        @click="materials.importFiles"
      >
        <Upload class="size-3.5" />{{ t("spaces.media.videoEditor.materials.import") }}
      </UiButton>
    </div>
    <UiDragGhost
      v-if="materialDrag.drag.value"
      :x="materialDrag.drag.value.x"
      :y="materialDrag.drag.value.y"
      :droppable="materialDrag.drag.value.target !== undefined"
    >
      <component
        :is="ICONS[materialDrag.drag.value.item.data.kind as keyof typeof ICONS]"
        class="size-3.5 shrink-0"
      />
      <span class="truncate">{{ materialDrag.drag.value.item.data.title }}</span>
    </UiDragGhost>
  </div>
</template>
