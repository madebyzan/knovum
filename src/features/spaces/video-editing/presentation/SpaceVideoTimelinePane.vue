<script setup lang="ts">
import SpaceVideoTimeline from "./SpaceVideoTimeline.vue";
import SpaceVideoTransport from "./SpaceVideoTransport.vue";
import type { VideoEditor } from "./use-video-editor";

defineProps<{ editor: VideoEditor }>();
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col bg-surface-canvas">
    <SpaceVideoTransport :editor="editor" />
    <SpaceVideoTimeline :editor="editor" />
  </div>
</template>
