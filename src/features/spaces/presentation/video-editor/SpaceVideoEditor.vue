<script setup lang="ts">
import { toRef } from "vue";

import type { SpaceEditorTab } from "../../workspace/work-items";
import SpaceEditorProblem from "../editor/SpaceEditorProblem.vue";
import SpaceVideoEditorHeader from "./SpaceVideoEditorHeader.vue";
import SpaceVideoEditorState from "./SpaceVideoEditorState.vue";
import SpaceVideoInspector from "./SpaceVideoInspector.vue";
import SpaceVideoPreview from "./SpaceVideoPreview.vue";
import SpaceVideoTimeline from "./SpaceVideoTimeline.vue";
import SpaceVideoTransportBar from "./SpaceVideoTransportBar.vue";
import SpaceVideoUnsavedDialog from "./SpaceVideoUnsavedDialog.vue";
import { useVideoEditor } from "./use-video-editor";

const props = defineProps<{ tab: SpaceEditorTab; active: boolean }>();
const {
  session,
  data,
  proxyPath,
  prepareError,
  retryProxy,
  project,
  video,
  editorRoot,
  timelineZoom,
  selection,
  playback,
  inspectorVisible,
  onSelectText,
  onSelectClip,
  closeInspector,
  previewPath,
  editorError,
  history,
  actions,
  sourceDurationMs,
  onMetadataReady,
  onPreviewReady,
  onMetadataError,
  onAddText,
} = useVideoEditor(props.tab, toRef(props, "active"));
</script>

<template>
  <section
    v-if="active"
    :ref="(value) => (editorRoot = value as HTMLElement | null)"
    class="absolute inset-0 z-60 flex min-h-0 flex-col bg-surface-canvas"
  >
    <SpaceVideoEditorHeader
      :title="data.title"
      :can-save="
        !session.error.value &&
        !!project &&
        history.dirty.value &&
        !actions.saving.value &&
        !actions.rendering.value
      "
      :can-render="
        !session.error.value && !!project && !editorError && !actions.saving.value && !actions.rendering.value
      "
      :preparing="!proxyPath && !prepareError"
      :dirty="history.dirty.value"
      @close="actions.requestClose"
      @save="actions.save()"
      @render="actions.render"
    />
    <SpaceEditorProblem
      v-if="session.error.value"
      :error="session.error.value"
      @discard="actions.requestClose"
    />
    <div v-else-if="project && !editorError" class="flex min-h-0 flex-1 flex-col">
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
    :can-save="!session.error.value"
    @cancel="actions.cancelLeave"
    @discard="actions.confirmDiscard"
    @save="actions.confirmSave"
  />
</template>
