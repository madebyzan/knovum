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
        class="inline-flex max-w-full min-w-0 items-center gap-2 rounded-control border border-divider bg-control py-1 pr-1 pl-1.5 text-caption focus-visible:outline-2 focus-visible:outline-focus-ring"
        tabindex="0"
      >
        <img
          v-if="attachment.previewSource"
          :src="attachment.previewSource"
          alt=""
          class="size-7 shrink-0 rounded-control object-cover"
        />
        <span
          v-else
          class="flex size-7 shrink-0 items-center justify-center rounded-control"
          :class="attachment.kind === 'node' ? 'bg-accent-subtle text-accent' : 'bg-raised text-foreground-secondary'"
        >
          <Link2 v-if="attachment.kind === 'node'" class="size-(--knovum-icon-size-sm)" />
          <FileText v-else class="size-(--knovum-icon-size-sm)" />
        </span>
        <span class="min-w-0 max-w-36">
          <span class="block text-micro text-foreground-tertiary">{{
            $t(attachment.kind === "node" ? "spaces.assistant.canvasNodeShort" : "spaces.assistant.localFileShort")
          }}</span>
          <span class="block truncate font-medium text-foreground">{{ attachment.title }}</span>
        </span>
        <button
          type="button"
          class="ml-0.5 flex size-6 shrink-0 items-center justify-center rounded-control text-foreground-tertiary hover:bg-control-hover hover:text-foreground focus-visible:outline-2 focus-visible:outline-focus-ring"
          :aria-label="$t('spaces.assistant.removeAttachedResource', { name: attachment.title })"
          @click="emit('remove', attachment.key)"
        >
          <X class="size-(--knovum-icon-size-xs)" />
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
        class="z-50 w-56 rounded-panel border border-border bg-panel p-1.5 shadow-dialog"
      >
        <img :src="attachment.previewSource" :alt="attachment.title" class="max-h-64 w-full rounded-control object-contain" />
        <p class="truncate px-1 pt-1 text-caption text-foreground-secondary">{{ attachment.title }}</p>
      </HoverCardContent>
    </HoverCardPortal>
  </HoverCardRoot>
</template>
