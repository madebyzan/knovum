<script setup lang="ts">
import SpaceVideoTimeline from "./SpaceVideoTimeline.vue";
import SpaceVideoTransportBar from "./SpaceVideoTransportBar.vue";
import type { VideoEditor } from "./use-video-editor";
const props = defineProps<{ editor: VideoEditor }>();
const {
  project,
  data,
  playback,
  actions,
  history,
  timelineZoom,
  materials,
  selection,
  inspector,
  trimDraft,
  onAddText,
} = props.editor;
function addMaterial(id: string, atMs: number) {
  const node = materials.nodes.value.find((candidate) => candidate.id === id);
  if (node) void materials.add(node, atMs);
}
</script>
<template>
  <template v-if="project">
    <SpaceVideoTransportBar
      :playing="playback.playing.value"
      :current-ms="playback.timelineMs.value"
      :duration-ms="playback.durationMs.value"
      :can-delete="actions.canDelete.value"
      :can-split="actions.canSplit.value"
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
      :space-id="editor.session.tab.spaceId"
      :project="project"
      :materials="materials.available.value"
      :playhead-ms="playback.timelineMs.value"
      :selected-clip-id="selection.selectedClipId.value"
      :selected-text-id="selection.selectedTextId.value"
      :selected-audio-id="selection.selectedAudioId.value"
      :zoom="timelineZoom"
      :playing="playback.playing.value"
      :waveform="
        data.mediaAnalysis?.sourceRevision === data.sourceRevision ? data.mediaAnalysis?.waveform : undefined
      "
      @seek="playback.seek"
      @pause="playback.pause"
      @select-clip="inspector.selectClip"
      @select-text="inspector.selectText"
      @select-audio="inspector.selectAudio"
      @edit="history.commit"
      @draft="trimDraft = $event"
      @add-material="addMaterial"
    />
  </template>
</template>
