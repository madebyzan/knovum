<script setup lang="ts">
import { computed, nextTick, shallowRef, watch } from "vue";

import { useSpaceStore } from "../../session/space-store";
import { useVideoEditProxy } from "../../video-editing/use-video-edit-proxy";
import SpaceVideoEditorHeader from "./SpaceVideoEditorHeader.vue";
import SpaceVideoEditorState from "./SpaceVideoEditorState.vue";
import SpaceVideoInspector from "./SpaceVideoInspector.vue";
import SpaceVideoPreview from "./SpaceVideoPreview.vue";
import SpaceVideoTimeline from "./SpaceVideoTimeline.vue";
import SpaceVideoTransportBar from "./SpaceVideoTransportBar.vue";
import SpaceVideoUnsavedDialog from "./SpaceVideoUnsavedDialog.vue";
import { useVideoEditActions } from "./use-video-edit-actions";
import { useVideoEditHistory } from "./use-video-edit-history";
import { useVideoEditPreviewSource } from "./use-video-edit-preview-source";
import { useVideoEditProject } from "./use-video-edit-project";
import { useVideoEditSelection } from "./use-video-edit-selection";
import { useVideoEditorInspector } from "./use-video-editor-inspector";
import { useVideoEditorMedia } from "./use-video-editor-media";
import { useVideoProjectPlayback } from "./use-video-project-playback";

const store = useSpaceStore();
const node = computed(() => store.nodes.find(({ id }) => id === store.videoEditorNodeId));
const data = computed(() => (node.value?.data.kind === "video" ? node.value.data : undefined));
const {
  path: proxyPath,
  error: prepareError,
  retry: retryProxy,
} = useVideoEditProxy(
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
const editorRoot = shallowRef<HTMLElement | null>(null);
const timelineZoom = shallowRef(1);
const selection = useVideoEditSelection(project);
const playback = useVideoProjectPlayback(video, project);
const {
  visible: inspectorVisible,
  selectText: onSelectText,
  selectClip: onSelectClip,
  close: closeInspector,
} = useVideoEditorInspector({ project, video, editorRoot, selection, playback });
const editorError = computed(() => prepareError.value ?? metadataError.value ?? playback.audioError.value);
const { previewPath, onMetadataError } = useVideoEditPreviewSource({
  data,
  proxyPath,
  playing: playback.playing,
  metadataError,
});
const history = useVideoEditHistory(project);
const actions = useVideoEditActions({ node, data, project, video, history, selection, playback });
const { sourceDurationMs, onMetadataReady, onPreviewReady } = useVideoEditorMedia({
  nodeId: computed(() => node.value?.id),
  data,
  project,
  video,
  timelineZoom,
  history,
  selection,
  playback,
  initializeFromMetadata,
});

watch(project, () => playback.seek(playback.timelineMs.value));

async function onAddText() {
  actions.addText();
  await nextTick();
  const input = editorRoot.value?.querySelector<HTMLTextAreaElement>("#video-edit-text");
  input?.focus();
  input?.select();
}
</script>

<template>
  <section
    v-if="data"
    :ref="(value) => (editorRoot = value as HTMLElement | null)"
    class="absolute inset-0 z-60 flex min-h-0 flex-col bg-surface-canvas"
  >
    <SpaceVideoEditorHeader
      :title="data.title"
      :can-save="!!project && history.dirty.value && !actions.saving.value && !actions.rendering.value"
      :can-render="!!project && !editorError && !actions.saving.value && !actions.rendering.value"
      :preparing="!proxyPath && !prepareError"
      :dirty="history.dirty.value"
      @close="actions.requestClose"
      @save="actions.save()"
      @render="actions.render"
    />
    <div v-if="project && !editorError" class="flex min-h-0 flex-1 flex-col">
      <div class="relative flex min-h-0 flex-1 bg-surface-control/25" :class="inspectorVisible && 'pr-84'">
        <SpaceVideoPreview
          v-model:element="video"
          :source-path="previewPath ?? data.assetPath"
          :project="project"
          :timeline-ms="playback.timelineMs.value"
          :selected-text-id="selection.selectedTextId.value"
          @playing="playback.playing.value = $event"
          @ready="onPreviewReady"
          @failed="onMetadataError"
          @select-text="onSelectText"
        />
        <SpaceVideoInspector
          v-if="inspectorVisible"
          :model-value="project"
          :source-duration-ms="sourceDurationMs"
          :selected-clip-id="selection.selectedClipId.value"
          :selected-text-id="selection.selectedTextId.value"
          @edit="history.commit"
          @close="closeInspector"
          @select-text="onSelectText"
          @add-text="onAddText"
        />
      </div>
      <SpaceVideoTransportBar
        :playing="playback.playing.value"
        :current-ms="playback.timelineMs.value"
        :duration-ms="playback.durationMs.value"
        :can-delete="actions.canDelete.value"
        :can-undo="history.canUndo.value"
        :can-redo="history.canRedo.value"
        :zoom="timelineZoom"
        @split="actions.split"
        @add-text="onAddText"
        @remove="actions.remove"
        @undo="history.undo"
        @redo="history.redo"
        @toggle="actions.togglePlayback"
        @step="actions.step"
        @zoom="timelineZoom = $event"
      />
      <SpaceVideoTimeline
        :project="project"
        :source-duration-ms="sourceDurationMs"
        :playhead-ms="playback.timelineMs.value"
        :selected-clip-id="selection.selectedClipId.value"
        :selected-text-id="selection.selectedTextId.value"
        :zoom="timelineZoom"
        :playing="playback.playing.value"
        :thumbnail-source="proxyPath"
        :waveform="
          data.mediaAnalysis?.sourceRevision === data.sourceRevision
            ? data.mediaAnalysis?.waveform
            : undefined
        "
        @seek="playback.seek"
        @select-clip="onSelectClip"
        @select-text="onSelectText"
        @edit="history.commit($event)"
      />
    </div>
    <SpaceVideoEditorState
      v-else
      :data="data"
      :proxy-path="proxyPath"
      :error="editorError"
      :can-retry="!!prepareError"
      @metadata-ready="onMetadataReady"
      @metadata-error="onMetadataError"
      @retry="retryProxy"
    />
  </section>
  <SpaceVideoUnsavedDialog
    v-model:open="actions.closeDialogOpen.value"
    :saving="actions.saving.value"
    @cancel="actions.cancelLeave"
    @discard="actions.confirmDiscard"
    @save="actions.confirmSave"
  />
</template>
