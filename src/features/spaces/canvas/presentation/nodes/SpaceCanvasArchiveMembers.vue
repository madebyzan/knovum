<script setup lang="ts">
import { File, Folder } from "@lucide/vue";
import { computed } from "vue";

import type { SpaceArchiveMember } from "../../../model/nodes/node-payload";
import type { CanvasNodePresentationContext } from "../../preview/policy";
import { projectPreviewItems, remainingPreviewItemCount } from "./canvas-preview-budget";

const props = defineProps<{
  members: SpaceArchiveMember[];
  presentationContext: CanvasNodePresentationContext;
}>();

const interactive = computed(() => props.presentationContext.mode === "interactive");
const displayedMembers = computed(() => projectPreviewItems(props.members, props.presentationContext));
const remaining = computed(() =>
  remainingPreviewItemCount(props.members.length, displayedMembers.value.length),
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
      v-for="member in displayedMembers"
      :key="member.path"
      class="flex h-7 items-center gap-2 border-b border-stroke-divider px-1 text-caption last:border-b-0"
    >
      <Folder v-if="member.kind === 'folder'" class="size-3 shrink-0 text-fg-tertiary" />
      <File v-else class="size-3 shrink-0 text-fg-tertiary" />
      <span class="min-w-0 flex-1 truncate font-mono text-fg-secondary" :title="member.path">
        {{ member.path }}
      </span>
    </div>
    <div v-if="remaining > 0" class="px-1 pt-1 text-caption-sm text-fg-tertiary">
      +{{ remaining }} more entries
    </div>
  </div>
</template>
