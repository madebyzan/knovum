<script setup lang="ts">
import { computed } from "vue";

import { VIDEO_CAPTION_HEIGHT } from "../../../canvas/node-size";
import { mediaProviderDisplayName } from "../../../media/provider-display-name";
import type { SpaceVideoPayload } from "../../../model/nodes/node-payload";
import SpaceCanvasNodeStatus from "./SpaceCanvasNodeStatus.vue";

const props = defineProps<{ id: string; data: SpaceVideoPayload }>();
const provider = computed(() =>
  props.data.origin ? mediaProviderDisplayName(props.data.origin.provider) : undefined,
);
</script>

<template>
  <footer
    class="flex shrink-0 flex-col justify-center gap-1 border-t border-divider px-3 py-1.5"
    :style="{ height: `${VIDEO_CAPTION_HEIGHT}px` }"
  >
    <div class="flex min-w-0 items-start gap-2">
      <strong
        data-video-title
        class="line-clamp-2 min-w-0 flex-1 text-label leading-snug font-semibold text-foreground"
        :title="data.title"
      >
        {{ data.title }}
      </strong>
      <SpaceCanvasNodeStatus :id="id" :data="data" />
    </div>
    <div
      v-if="provider || data.creator"
      data-video-source
      class="flex min-w-0 items-center gap-1.5 text-caption text-foreground-tertiary"
    >
      <span v-if="provider" class="shrink-0 font-medium text-foreground-secondary">{{ provider }}</span>
      <span v-if="provider && data.creator" aria-hidden="true">·</span>
      <span v-if="data.creator" class="min-w-0 flex-1 truncate">{{ data.creator }}</span>
    </div>
  </footer>
</template>
