<script setup lang="ts">
import { File, Folder } from "@lucide/vue";
import { computed } from "vue";

import type { SpaceResourceEntry } from "../../../model/nodes/node-payload";

const props = defineProps<{
  entries: SpaceResourceEntry[];
  detailLevel: "compact" | "summary" | "full";
  interactive?: boolean;
}>();

const isCompact = computed(() => props.detailLevel === "compact");
const displayEntries = computed(() => (isCompact.value ? props.entries.slice(0, 3) : props.entries));
const compactRemaining = computed(() => Math.max(0, props.entries.length - displayEntries.value.length));
</script>

<template>
  <div
    class="min-h-0 flex-1 px-2 py-1.5 select-none"
    :class="isCompact ? 'overflow-hidden' : 'overflow-y-auto'"
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
    <div v-if="isCompact && compactRemaining > 0" class="px-1 pt-1 text-micro text-foreground-tertiary">
      +{{ compactRemaining }} more items
    </div>
  </div>
</template>
