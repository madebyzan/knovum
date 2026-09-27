<script setup lang="ts">
import { File, Folder } from "@lucide/vue";
import { computed } from "vue";

import type { CanvasNodePresentationContext } from "../../../canvas/preview/policy";
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
      class="flex h-7 items-center gap-2 border-b border-stroke-divider px-1 text-caption-md last:border-b-0"
    >
      <Folder v-if="entry.kind === 'folder'" class="size-3 shrink-0 text-fg-tertiary" />
      <File v-else class="size-3 shrink-0 text-fg-tertiary" />
      <span class="min-w-0 flex-1 truncate font-mono text-fg-secondary" :title="entry.path">
        {{ entry.path }}
      </span>
    </div>
    <div v-if="remaining > 0" class="px-1 pt-1 text-caption-sm text-fg-tertiary">
      +{{ remaining }} more entries
    </div>
  </div>
</template>
