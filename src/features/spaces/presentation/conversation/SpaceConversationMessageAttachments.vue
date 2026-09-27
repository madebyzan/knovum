<script setup lang="ts">
import { FileText, Link2 } from "@lucide/vue";

import type { AssistantMessageAttachment } from "../../contracts/generated/AssistantMessageAttachment";
import { useSpaceStore } from "../../session/space-store";

defineProps<{ attachments: readonly AssistantMessageAttachment[] }>();
const store = useSpaceStore();
</script>

<template>
  <div class="flex max-w-full flex-wrap justify-end gap-1.5">
    <template v-for="(attachment, index) in attachments" :key="index">
      <button
        v-if="attachment.source === 'node'"
        type="button"
        class="inline-flex max-w-full items-center gap-1.5 rounded-md border border-stroke-divider px-2 py-1 text-caption-md text-fg-secondary hover:bg-surface-control-hover hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
        @click="store.requestReveal([attachment.nodeId], 'focus')"
      >
        <Link2 class="size-3 shrink-0 text-fg-accent" />
        <span class="truncate">{{ attachment.title }}</span>
      </button>
      <span
        v-else
        class="inline-flex max-w-full items-center gap-1.5 rounded-md border border-stroke-divider px-2 py-1 text-caption-md text-fg-secondary"
      >
        <FileText class="size-3 shrink-0" />
        <span class="truncate">{{ attachment.title }}</span>
      </span>
    </template>
  </div>
</template>
