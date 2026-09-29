<script setup lang="ts">
import { File, FileImage, Link2 } from "@lucide/vue";

import type { AssistantMessageAttachment } from "../../contracts/generated/AssistantMessageAttachment";
import { useSpaceStore } from "../../session/space-store";

const props = defineProps<{ attachment: AssistantMessageAttachment }>();
const store = useSpaceStore();

function openSource() {
  if (props.attachment.source === "node") store.requestReveal([props.attachment.nodeId], "focus");
}
</script>

<template>
  <component
    :is="attachment.source === 'node' ? 'button' : 'div'"
    :type="attachment.source === 'node' ? 'button' : undefined"
    :role="attachment.source === 'file' ? 'img' : undefined"
    class="group relative flex h-20 shrink-0 flex-col overflow-hidden rounded-lg border border-stroke-divider bg-surface-raised text-left transition-colors duration-100 enabled:hover:border-stroke-default-hover enabled:hover:bg-surface-control-hover focus-visible:outline-2 focus-visible:outline-stroke-focus"
    :class="attachment.kind === 'image' ? 'w-20' : 'w-28'"
    :aria-label="attachment.title"
    :title="attachment.title"
    @click="openSource"
  >
    <img
      v-if="attachment.thumbnail"
      :src="attachment.thumbnail"
      alt=""
      class="min-h-0 w-full flex-1 bg-surface-muted object-contain"
    />
    <div v-else class="flex min-h-0 flex-1 items-center justify-center bg-surface-muted">
      <FileImage v-if="attachment.kind === 'image'" class="size-5 text-fg-tertiary" />
      <File v-else class="size-5 text-fg-tertiary" />
    </div>
    <div v-if="attachment.kind !== 'image'" class="flex h-7 min-w-0 shrink-0 items-center gap-1.5 px-2 text-caption text-fg-primary">
      <Link2 v-if="attachment.source === 'node'" class="size-3.5 shrink-0 text-fg-accent" />
      <span class="truncate">{{ attachment.title }}</span>
    </div>
    <span v-if="attachment.source === 'node' && attachment.kind === 'image'" class="absolute bottom-1 left-1 rounded bg-surface-overlay/90 p-0.5 text-fg-accent">
      <Link2 class="size-3" />
    </span>
  </component>
</template>
