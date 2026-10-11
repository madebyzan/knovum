<script setup lang="ts">
import { File, FileImage, FileText, Link2, X } from "@lucide/vue";

import type { ConversationAttachmentPresentation } from "../../conversation/conversation-attachment-presentation";

defineProps<{
  attachment: ConversationAttachmentPresentation;
  unresolved?: boolean;
}>();
const emit = defineEmits<{ remove: [key: string]; preview: [key: string] }>();
</script>

<template>
  <div
    class="group relative flex h-20 shrink-0 flex-col overflow-hidden rounded-lg border border-stroke-divider bg-surface-raised transition-[border-color,box-shadow] duration-150 focus-within:border-stroke-focus hover:border-stroke-default-hover hover:shadow-control"
    :class="attachment.isImage ? 'w-20' : 'w-28'"
    role="group"
    :aria-label="`${$t(attachment.kind === 'node' ? 'spaces.conversation.attachments.canvasReference.shortLabel' : 'spaces.conversation.attachments.localFile.shortLabel')}: ${attachment.title}`"
  >
    <button
      type="button"
      class="flex min-h-0 flex-1 flex-col text-left focus-visible:outline-2 focus-visible:outline-stroke-focus"
      :title="attachment.title"
      :aria-label="
        unresolved
          ? attachment.title
          : $t('spaces.conversation.attachments.preview', { name: attachment.title })
      "
      :disabled="unresolved"
      @click="emit('preview', attachment.key)"
    >
      <span class="flex min-h-0 w-full flex-1 items-center justify-center overflow-hidden bg-surface-muted">
        <img
          v-if="attachment.previewSource"
          :src="attachment.previewSource"
          alt=""
          class="size-full object-cover"
        />
        <FileImage v-else-if="attachment.isImage" class="size-5 text-fg-tertiary" :stroke-width="1.5" />
        <File v-else class="size-5 text-fg-tertiary" :stroke-width="1.5" />
      </span>
      <span v-if="!attachment.isImage" class="flex h-7 w-full min-w-0 shrink-0 items-center gap-1.5 px-2">
        <Link2 v-if="attachment.kind === 'node'" class="size-3.5 shrink-0 text-fg-accent" />
        <FileText v-else class="size-3.5 shrink-0 text-fg-secondary" />
        <span class="min-w-0 truncate text-caption font-medium text-fg-primary">{{ attachment.title }}</span>
      </span>
    </button>
    <button
      type="button"
      class="attachment-remove absolute top-1 right-1 flex size-6 items-center justify-center rounded-md bg-surface-overlay text-fg-secondary opacity-0 shadow-control transition-[color,opacity] duration-150 group-focus-within:opacity-100 group-hover:opacity-100 hover:bg-surface-overlay hover:text-fg-primary hover:opacity-100 focus-visible:opacity-100 focus-visible:outline-2 focus-visible:outline-stroke-focus"
      :aria-label="$t('spaces.conversation.attachments.remove', { name: attachment.title })"
      @click="emit('remove', attachment.key)"
    >
      <X class="size-3.5" />
    </button>
  </div>
</template>

<style scoped>
@media (hover: none) {
  .attachment-remove {
    opacity: 1;
  }
}
</style>
