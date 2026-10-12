<script setup lang="ts">
import { shallowRef } from "vue";

import UiButton from "../../../../shared/ui/UiButton.vue";
import SpaceVideoOutputSettings from "./SpaceVideoOutputSettings.vue";
import SpaceVideoPreview from "./SpaceVideoPreview.vue";
import type { VideoEditor } from "./use-video-editor";
const props = defineProps<{ editor: VideoEditor }>();
const {
  session,
  project,
  visibleProject,
  previewError,
  materials,
  playback,
  selection,
  inspector,
  history,
  failPreview,
} = props.editor;
const attempt = shallowRef(0);
function retry() {
  previewError.value = undefined;
  attempt.value++;
}
</script>
<template>
  <div v-if="project && visibleProject" class="flex h-full min-h-0 flex-col">
    <header class="flex h-10 shrink-0 items-center justify-between border-b border-stroke-divider px-3">
      <span class="text-caption text-fg-secondary"
        >{{ project.output.width }} × {{ project.output.height }} · {{ project.output.frameRate }} fps</span
      >
      <SpaceVideoOutputSettings :project="project" @edit="history.commit" />
    </header>
    <div
      v-if="previewError"
      role="alert"
      class="flex shrink-0 items-center justify-between gap-2 border-b border-stroke-divider px-3 py-2 text-caption text-fg-danger"
    >
      <span>{{ previewError }}</span
      ><UiButton size="sm" @click="retry">{{ $t("spaces.media.videoEditor.preview.retry") }}</UiButton>
    </div>
    <SpaceVideoPreview
      :key="attempt"
      :project="visibleProject"
      :materials="materials.available.value"
      :space-id="session.tab.spaceId"
      :timeline-ms="playback.timelineMs.value"
      :playing="playback.playing.value"
      :selected-text-id="selection.selectedTextId.value"
      @ready="playback.ready.value = $event"
      @failed="failPreview"
      @select-text="inspector.selectText"
    />
    <p
      v-if="playback.playing.value && !playback.ready.value"
      role="status"
      class="shrink-0 py-1 text-center text-caption-sm text-fg-secondary"
    >
      {{ $t("spaces.media.videoEditor.preview.optimizing") }}
    </p>
  </div>
</template>
