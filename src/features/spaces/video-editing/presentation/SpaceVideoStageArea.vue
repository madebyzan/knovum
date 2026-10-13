<script setup lang="ts">
import { SplitterGroup, SplitterPanel, SplitterResizeHandle } from "reka-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import SpaceVideoInspector from "./SpaceVideoInspector.vue";
import SpaceVideoLibrary from "./SpaceVideoLibrary.vue";
import SpaceVideoStage from "./SpaceVideoStage.vue";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor }>();
const { t } = useI18n();
const { selection } = props.editor;
const inspecting = computed(() => !!selection.selection.value);
</script>

<template>
  <SplitterGroup direction="horizontal" class="min-h-0 flex-1">
    <SplitterPanel :min-size="14" :default-size="19" :max-size="30" class="flex min-h-0 flex-col">
      <SpaceVideoLibrary :editor="editor" />
    </SplitterPanel>
    <SplitterResizeHandle
      class="group relative z-20 w-px shrink-0 bg-stroke-default transition-colors outline-none hover:bg-surface-accent focus-visible:bg-surface-accent data-[state=drag]:bg-surface-accent"
      :aria-label="t('spaces.media.videoEditor.layout.materials')"
      @pointerdown.stop
    >
      <span class="absolute inset-y-0 -left-1.5 w-3" />
    </SplitterResizeHandle>
    <SplitterPanel :min-size="30" class="flex min-h-0 flex-col">
      <SpaceVideoStage :editor="editor" />
    </SplitterPanel>
    <template v-if="inspecting">
      <SplitterResizeHandle
        class="group relative z-20 w-px shrink-0 bg-stroke-default transition-colors outline-none hover:bg-surface-accent focus-visible:bg-surface-accent data-[state=drag]:bg-surface-accent"
        :aria-label="t('spaces.media.videoEditor.layout.inspector')"
        @pointerdown.stop
      >
        <span class="absolute inset-y-0 -left-1.5 w-3" />
      </SplitterResizeHandle>
      <SplitterPanel :min-size="16" :default-size="21" :max-size="34" class="flex min-h-0 flex-col">
        <SpaceVideoInspector :editor="editor" />
      </SplitterPanel>
    </template>
  </SplitterGroup>
</template>
