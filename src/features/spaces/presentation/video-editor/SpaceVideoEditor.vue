<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import type { SpaceVideoPayload } from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";
import { addTextOverlay, splitClip } from "../../video-editing/project";
import { renderVideoEdit } from "../../video-editing/render";
import { versionedMediaSource } from "../media/element";
import SpaceVideoEditorHeader from "./SpaceVideoEditorHeader.vue";
import SpaceVideoInspector from "./SpaceVideoInspector.vue";
import SpaceVideoPreview from "./SpaceVideoPreview.vue";
import SpaceVideoTimeline from "./SpaceVideoTimeline.vue";
import SpaceVideoTransportBar from "./SpaceVideoTransportBar.vue";
import { useVideoEditProject } from "./use-video-edit-project";
import { useVideoEditProxy } from "./use-video-edit-proxy";
import { useVideoEditSelection } from "./use-video-edit-selection";
import { useVideoProjectPlayback } from "./use-video-project-playback";

const store = useSpaceStore();
const { t } = useI18n();
const node = computed(() => store.nodes.find(({ id }) => id === store.videoEditorNodeId));
const data = computed(() => (node.value?.data.kind === "video" ? node.value.data : undefined));
const { path: proxyPath, error: prepareError } = useVideoEditProxy(
  computed(() => node.value?.id),
  computed(() => data.value?.sourceRevision),
  computed(() => store.activeSpaceId),
);
const { project, metadataError, initializeFromMetadata } = useVideoEditProject(
  computed(() => node.value?.id),
  data,
  proxyPath,
);
const video = shallowRef<HTMLVideoElement | null>(null);
const previewPath = shallowRef<string>();
const editorError = computed(() => prepareError.value ?? metadataError.value);
const { selectedClipId, selectedTextId, selectClip, selectText, removeSelection, reset } =
  useVideoEditSelection(project);
const playback = useVideoProjectPlayback(video, project);

watch(
  [() => data.value?.assetPath, () => data.value?.sourceRevision],
  ([path]) => {
    previewPath.value = path;
    metadataError.value = undefined;
  },
  { immediate: true },
);
watch([proxyPath, playback.playing], ([prepared, playing]) => {
  if (prepared && !playing) previewPath.value = prepared;
});
watch(project, () => playback.seek(playback.timelineMs.value));

watch(
  [() => node.value?.id, () => data.value?.sourceRevision],
  () => {
    reset();
    playback.seek(0);
  },
  { immediate: true },
);

function onMetadataReady(event: Event) {
  initializeFromMetadata(event);
  if (project.value) {
    reset();
    playback.seek(0);
  }
}

function onMetadataError() {
  if (!proxyPath.value) return;
  if (previewPath.value !== proxyPath.value) {
    previewPath.value = proxyPath.value;
    return;
  }
  metadataError.value = "Video preview cannot be opened";
}

async function save() {
  if (!node.value || !data.value || !project.value) return;
  const after: SpaceVideoPayload = { ...data.value, videoEditProject: project.value };
  await store.updateManyData([{ id: node.value.id, before: data.value, after }]);
  toast.success(t("spaces.media.projectSaved"));
}

async function render() {
  if (!node.value || node.value.data.kind !== "video" || !project.value) return;
  await renderVideoEdit(store, { ...node.value, data: node.value.data }, project.value);
}
</script>

<template>
  <section v-if="data" class="absolute inset-0 z-60 flex min-h-0 flex-col bg-canvas">
    <SpaceVideoEditorHeader
      :title="data.title"
      :can-save="!!project"
      :can-render="!!project && !editorError"
      :preparing="!proxyPath && !prepareError"
      @close="store.closeVideoEditor"
      @save="save()"
      @render="render"
    />
    <div v-if="project && !editorError" class="flex min-h-0 flex-1">
      <div class="flex min-w-0 flex-1 flex-col">
        <SpaceVideoPreview
          v-model:element="video"
          :source-path="previewPath ?? data.assetPath"
          :project="project"
          :timeline-ms="playback.timelineMs.value"
          @playing="playback.playing.value = $event"
          @ready="playback.onSourceReady"
          @failed="onMetadataError"
        />
        <SpaceVideoTransportBar
          :playing="playback.playing.value"
          :current-ms="playback.timelineMs.value"
          :duration-ms="playback.durationMs.value"
          :can-delete="!!selectedClipId || !!selectedTextId"
          @split="project = splitClip(project, playback.timelineMs.value)"
          @add-text="project = addTextOverlay(project, playback.timelineMs.value)"
          @remove="removeSelection"
          @toggle="playback.toggle"
          @seek="playback.seek"
        />
        <SpaceVideoTimeline
          :project="project"
          :playhead-ms="playback.timelineMs.value"
          :selected-clip-id="selectedClipId"
          :selected-text-id="selectedTextId"
          @seek="playback.seek"
          @select-clip="selectClip"
          @select-text="selectText"
        />
      </div>
      <SpaceVideoInspector
        v-model="project"
        :selected-clip-id="selectedClipId"
        :selected-text-id="selectedTextId"
      />
    </div>
    <div v-else-if="editorError" class="grid min-h-0 flex-1 place-items-center text-body text-danger">
      {{ editorError }}
    </div>
    <div v-else class="grid min-h-0 flex-1 place-items-center text-body text-foreground-secondary">
      <video
        v-if="!project"
        class="pointer-events-none absolute size-px opacity-0"
        :src="versionedMediaSource(proxyPath ?? data.assetPath, data.sourceRevision)"
        preload="metadata"
        aria-hidden="true"
        tabindex="-1"
        @loadedmetadata="onMetadataReady"
        @error="onMetadataError"
      />
      {{ t("spaces.media.loadingEditor") }}
    </div>
  </section>
</template>
