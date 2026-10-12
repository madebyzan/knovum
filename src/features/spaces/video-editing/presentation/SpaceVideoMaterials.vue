<script setup lang="ts">
import { Film, Music2, Plus, Upload } from "@lucide/vue";
import { computed, shallowRef, watch } from "vue";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import { versionedMediaSource } from "../../media/media-element-source";
import SpaceAudioPlayer from "../../media/presentation/SpaceAudioPlayer.vue";
import SpaceVideoPlayer from "../../media/presentation/SpaceVideoPlayer.vue";
import type { VideoMaterialNode } from "../material";
const props = defineProps<{ nodes: readonly VideoMaterialNode[]; busy: boolean; playing: boolean }>();
const emit = defineEmits<{ add: [node: VideoMaterialNode]; import: []; preview: [] }>();
const search = shallowRef("");
const selectedId = shallowRef<string>();
const nodes = computed(() =>
  props.nodes.filter((node) =>
    node.data.title.toLocaleLowerCase().includes(search.value.toLocaleLowerCase()),
  ),
);
const selected = computed(() => props.nodes.find((node) => node.id === selectedId.value));
watch(
  () => props.playing,
  (playing) => {
    if (playing) selectedId.value = undefined;
  },
);
function select(id: string) {
  selectedId.value = id;
  emit("preview");
}
function drag(event: DragEvent, id: string) {
  event.dataTransfer?.setData("application/x-knovum-video-material", id);
  if (event.dataTransfer) event.dataTransfer.effectAllowed = "copy";
}
</script>
<template>
  <aside
    class="flex h-full min-h-0 flex-col border-r border-stroke-divider bg-surface-panel"
    :aria-label="$t('spaces.media.videoEditor.materials.title')"
  >
    <header class="flex h-10 shrink-0 items-center justify-between px-3">
      <h2 class="text-label font-semibold">{{ $t("spaces.media.videoEditor.materials.title") }}</h2>
      <UiIconButton
        :label="$t('spaces.media.videoEditor.materials.import')"
        size="sm"
        :disabled="busy"
        tooltip
        @click="emit('import')"
        ><Upload class="size-3.5"
      /></UiIconButton>
    </header>
    <div class="px-3 pb-2">
      <UiInput
        v-model="search"
        size="sm"
        :placeholder="$t('spaces.media.videoEditor.materials.search')"
        :aria-label="$t('spaces.media.videoEditor.materials.search')"
      />
    </div>
    <UiScrollArea class="min-h-0 flex-1">
      <p v-if="!nodes.length" class="px-3 py-6 text-center text-caption text-fg-secondary">
        {{ $t("spaces.media.videoEditor.materials.empty") }}
      </p>
      <div
        v-for="node in nodes"
        :key="node.id"
        class="group flex items-center gap-2 border-b border-stroke-divider px-2 py-2"
        :class="selectedId === node.id && 'bg-surface-accent-subtle'"
        draggable="true"
        @dragstart="drag($event, node.id)"
      >
        <button
          type="button"
          class="flex min-w-0 flex-1 items-center gap-2 rounded-md text-left focus-visible:outline-2 focus-visible:outline-stroke-focus"
          @click="select(node.id)"
          @dblclick="emit('add', node)"
        >
          <span class="grid h-9 w-12 shrink-0 place-items-center overflow-hidden rounded bg-surface-muted">
            <img
              v-if="node.data.kind === 'image' || node.data.previewPath"
              :src="
                versionedMediaSource(
                  node.data.kind === 'image' ? node.data.assetPath : node.data.previewPath,
                  node.data.sourceRevision,
                )
              "
              alt=""
              draggable="false"
              class="size-full object-cover"
            />
            <component
              :is="node.data.kind === 'video' ? Film : Music2"
              v-else
              class="size-4 text-fg-tertiary"
            />
          </span>
          <span class="min-w-0 truncate text-caption">{{ node.data.title }}</span>
        </button>
        <UiIconButton
          size="sm"
          :label="$t('spaces.media.videoEditor.materials.add', { title: node.data.title })"
          :disabled="busy"
          tooltip
          @click="emit('add', node)"
          ><Plus class="size-3.5"
        /></UiIconButton>
      </div>
    </UiScrollArea>
    <div
      v-if="selected"
      class="flex h-40 shrink-0 flex-col border-t border-stroke-divider bg-surface-muted p-2"
    >
      <SpaceVideoPlayer v-if="selected.data.kind === 'video'" :key="selected.id" :data="selected.data" />
      <SpaceAudioPlayer v-else-if="selected.data.kind === 'audio'" :key="selected.id" :data="selected.data" />
      <img
        v-else
        :src="versionedMediaSource(selected.data.assetPath, selected.data.sourceRevision)"
        :alt="selected.data.title"
        class="min-h-0 flex-1 object-contain"
      />
    </div>
    <UiButton size="sm" class="m-2 shrink-0" :disabled="busy" @click="emit('import')"
      ><Upload class="size-3.5" />{{ $t("spaces.media.videoEditor.materials.import") }}</UiButton
    >
  </aside>
</template>
