<script setup lang="ts">
import { FileText, Link2, X } from "@lucide/vue";
import { HoverCardContent, HoverCardPortal, HoverCardRoot, HoverCardTrigger } from "reka-ui";

defineProps<{
  attachment: { key: string; kind: "node" | "file"; title: string; previewSource: string };
}>();
const emit = defineEmits<{ remove: [key: string] }>();
</script>

<template>
  <HoverCardRoot :open-delay="350" :close-delay="100">
    <HoverCardTrigger as-child>
      <span
        class="inline-flex max-w-full min-w-0 items-center gap-2 rounded-md border border-stroke-divider bg-surface-control py-1 pr-1 pl-1.5 text-caption focus-visible:outline-2 focus-visible:outline-stroke-focus"
        tabindex="0"
      >
        <img
          v-if="attachment.previewSource"
          :src="attachment.previewSource"
          alt=""
          class="size-7 shrink-0 rounded-md object-cover"
        />
        <span
          v-else
          class="flex size-7 shrink-0 items-center justify-center rounded-md"
          :class="
            attachment.kind === 'node'
              ? 'bg-surface-accent-subtle text-fg-accent'
              : 'bg-surface-raised text-fg-secondary'
          "
        >
          <Link2 v-if="attachment.kind === 'node'" class="size-3.5" />
          <FileText v-else class="size-3.5" />
        </span>
        <span class="max-w-36 min-w-0">
          <span class="block text-caption-sm text-fg-tertiary">{{
            $t(
              attachment.kind === "node"
                ? "spaces.assistant.attachments.canvasNodeShort"
                : "spaces.assistant.attachments.localFileShort",
            )
          }}</span>
          <span class="block truncate font-medium text-fg-primary">{{ attachment.title }}</span>
        </span>
        <button
          type="button"
          class="ml-0.5 flex size-6 shrink-0 items-center justify-center rounded-md text-fg-tertiary hover:bg-surface-control-hover hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
          :aria-label="$t('spaces.assistant.attachments.remove', { name: attachment.title })"
          @click="emit('remove', attachment.key)"
        >
          <X class="size-3" />
        </button>
      </span>
    </HoverCardTrigger>
    <HoverCardPortal v-if="attachment.previewSource">
      <HoverCardContent
        side="top"
        align="start"
        :side-offset="8"
        :collision-padding="12"
        avoid-collisions
        class="z-50 w-56 rounded-panel border border-stroke-default bg-surface-panel p-1.5 shadow-dialog"
      >
        <img
          :src="attachment.previewSource"
          :alt="attachment.title"
          class="max-h-64 w-full rounded-md object-contain"
        />
        <p class="truncate px-1 pt-1 text-caption text-fg-secondary">{{ attachment.title }}</p>
      </HoverCardContent>
    </HoverCardPortal>
  </HoverCardRoot>
</template>
