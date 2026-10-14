<script setup lang="ts">
import { SplitterGroup, SplitterPanel, SplitterResizeHandle } from "reka-ui";
import { useI18n } from "vue-i18n";

import SpaceVideoStageArea from "./SpaceVideoStageArea.vue";
import SpaceVideoTimelinePane from "./SpaceVideoTimelinePane.vue";
import type { VideoEditor } from "./use-video-editor";

defineProps<{ editor: VideoEditor }>();
const { t } = useI18n();
</script>

<template>
  <SplitterGroup direction="vertical" class="min-h-0 flex-1">
    <SplitterPanel :min-size="34" :default-size="66" class="flex min-h-0 flex-col">
      <SpaceVideoStageArea :editor="editor" />
    </SplitterPanel>
    <SplitterResizeHandle
      class="h-0.5 shrink-0 bg-stroke-default transition-colors outline-none hover:bg-surface-accent focus-visible:bg-surface-accent data-[state=drag]:bg-surface-accent"
      :aria-label="t('spaces.media.videoEditor.layout.timeline')"
    />
    <SplitterPanel :min-size="20" :default-size="34" :max-size="62" class="flex min-h-0 flex-col">
      <SpaceVideoTimelinePane :editor="editor" />
    </SplitterPanel>
  </SplitterGroup>
</template>
