<script setup lang="ts">
import { SplitterGroup, SplitterPanel, SplitterResizeHandle } from "reka-ui";

import SpaceVideoEditorStage from "./SpaceVideoEditorStage.vue";
import SpaceVideoTimelinePane from "./SpaceVideoTimelinePane.vue";
import type { VideoEditor } from "./use-video-editor";
defineProps<{ editor: VideoEditor }>();
</script>
<template>
  <SplitterGroup direction="vertical" class="min-h-0 flex-1">
    <SplitterPanel :min-size="35" :default-size="65" class="flex min-h-0 flex-col"
      ><SpaceVideoEditorStage :editor="editor"
    /></SplitterPanel>
    <SplitterResizeHandle
      class="h-1 shrink-0 bg-stroke-divider hover:bg-surface-accent focus-visible:bg-surface-accent"
      :aria-label="$t('spaces.media.videoEditor.layout.timeline')"
    />
    <SplitterPanel :min-size="20" :default-size="35" :max-size="65" class="flex min-h-0 flex-col"
      ><SpaceVideoTimelinePane :editor="editor"
    /></SplitterPanel>
  </SplitterGroup>
</template>
