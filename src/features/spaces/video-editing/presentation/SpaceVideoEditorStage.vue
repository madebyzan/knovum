<script setup lang="ts">
import { SplitterGroup, SplitterPanel, SplitterResizeHandle } from "reka-ui";

import SpaceVideoInspector from "./SpaceVideoInspector.vue";
import SpaceVideoMaterials from "./SpaceVideoMaterials.vue";
import SpaceVideoPreviewPane from "./SpaceVideoPreviewPane.vue";
import type { VideoEditor } from "./use-video-editor";
const props = defineProps<{ editor: VideoEditor }>();
const { project, materials, playback, inspector, selection, history, onAddText } = props.editor;
</script>
<template>
  <SplitterGroup v-if="project" direction="horizontal" class="min-h-0 flex-1">
    <SplitterPanel :min-size="15" :default-size="22" :max-size="35"
      ><SpaceVideoMaterials
        :nodes="materials.nodes.value"
        :busy="materials.busy.value"
        :playing="playback.playing.value"
        @add="materials.add($event, playback.timelineMs.value)"
        @import="materials.importFiles"
        @preview="playback.pause"
    /></SplitterPanel>
    <SplitterResizeHandle
      class="w-1 shrink-0 bg-stroke-divider hover:bg-surface-accent focus-visible:bg-surface-accent"
      :aria-label="$t('spaces.media.videoEditor.layout.materials')"
    />
    <SplitterPanel :min-size="30" class="flex min-h-0 flex-col"
      ><SpaceVideoPreviewPane :editor="editor"
    /></SplitterPanel>
    <SplitterResizeHandle
      v-if="inspector.visible.value"
      class="w-1 shrink-0 bg-stroke-divider hover:bg-surface-accent focus-visible:bg-surface-accent"
      :aria-label="$t('spaces.media.videoEditor.layout.inspector')"
    />
    <SplitterPanel v-if="inspector.visible.value" :min-size="20" :default-size="25" :max-size="40"
      ><SpaceVideoInspector
        :model-value="project"
        :selected-clip-id="selection.selectedClipId.value"
        :selected-text-id="selection.selectedTextId.value"
        :selected-audio-id="selection.selectedAudioId.value"
        @edit="history.commit"
        @close="inspector.close"
        @select-text="inspector.selectText"
        @add-text="onAddText"
    /></SplitterPanel>
  </SplitterGroup>
</template>
