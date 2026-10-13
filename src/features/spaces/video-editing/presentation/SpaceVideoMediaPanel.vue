<script setup lang="ts">
import { Film, Image as ImageIcon, Music2, Plus, Upload } from "@lucide/vue";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import UiSearchInput from "../../../../shared/ui/UiSearchInput.vue";
import { versionedMediaSource } from "../../media/media-element-source";
import type { VideoMaterialNode } from "../material";
import { formatVideoTimelineTime } from "../timecode";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor }>();
const { t } = useI18n();
const { materials, engine } = props.editor;
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

function drag(event: DragEvent, node: VideoMaterialNode) {
  event.dataTransfer?.setData("application/x-knovum-video-material", node.id);
  if (event.dataTransfer) event.dataTransfer.effectAllowed = "copy";
}

function add(node: VideoMaterialNode) {
  void materials.add(node, engine.timeMs.value);
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
        <li v-for="node in visible" :key="node.id" class="group relative">
          <button
            type="button"
            class="w-full overflow-hidden rounded-lg border border-stroke-default bg-surface-muted text-left transition-colors hover:border-stroke-accent focus-visible:outline-2 focus-visible:outline-stroke-focus"
            draggable="true"
            :aria-label="t('spaces.media.videoEditor.materials.add', { title: node.data.title })"
            @dragstart="drag($event, node)"
            @dblclick="add(node)"
            @click="add(node)"
          >
            <span class="relative block aspect-video bg-surface-skeleton">
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
            <span class="block truncate px-2 py-1.5 text-caption">{{ node.data.title }}</span>
          </button>
          <UiIconButton
            class="absolute top-1 right-1 bg-surface-overlay opacity-0 shadow-control group-focus-within:opacity-100 group-hover:opacity-100"
            size="sm"
            :label="t('spaces.media.videoEditor.materials.add', { title: node.data.title })"
            :disabled="materials.busy.value"
            @click="add(node)"
          >
            <Plus class="size-3.5" />
          </UiIconButton>
        </li>
      </ul>
    </UiScrollArea>
    <div class="shrink-0 border-t border-stroke-divider p-2">
      <UiButton
        size="sm"
        class="w-full justify-center"
        :disabled="materials.busy.value"
        @click="materials.importFiles"
      >
        <Upload class="size-3.5" />{{ t("spaces.media.videoEditor.materials.import") }}
      </UiButton>
    </div>
  </div>
</template>
