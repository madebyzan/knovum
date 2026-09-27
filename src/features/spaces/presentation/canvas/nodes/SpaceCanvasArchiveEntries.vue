<script setup lang="ts">
import { File, Folder } from "@lucide/vue";
import { computed } from "vue";

import type { CanvasNodePresentationContext } from "../../../canvas/preview/detail-profile";
import type { SpaceArchiveMember } from "../../../model/nodes/node-payload";
import { projectPreviewItems, remainingPreviewItemCount } from "./canvas-preview-budget";

const props = defineProps<{
  entries: SpaceArchiveMember[];
  presentationContext: CanvasNodePresentationContext;
}>();

const interactive = computed(() => props.presentationContext.mode === "interactive");
const displayEntries = computed(() => projectPreviewItems(props.entries, props.presentationContext));
const remaining = computed(() =>
  remainingPreviewItemCount(props.entries.length, displayEntries.value.length),
);
</script>

<template>
  <div
    class="min-h-0 flex-1 px-2 py-1.5 select-none"
    :class="interactive ? 'overflow-y-auto' : 'overflow-hidden'"
    data-canvas-scrollable
    @wheel.stop
  >
    <div
      v-for="entry in displayEntries"
      :key="entry.path"
      class="flex h-7 items-center gap-2 border-b border-divider px-1 text-caption last:border-b-0"
    >
      <Folder
        v-if="entry.kind === 'folder'"
        class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary"
      />
      <File v-else class="size-(--knovum-icon-size-xs) shrink-0 text-foreground-tertiary" />
      <span class="min-w-0 flex-1 truncate font-mono text-foreground-secondary" :title="entry.path">
        {{ entry.path }}
      </span>
    </div>
    <div v-if="remaining > 0" class="px-1 pt-1 text-micro text-foreground-tertiary">
      +{{ remaining }} more entries
    </div>
  </div>
</template>
