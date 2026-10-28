<script setup lang="ts">
import { X } from "@lucide/vue";

import type { ConversationAttachmentPresentation } from "../conversation-attachment-presentation";
import SpaceConversationAttachmentFace from "./SpaceConversationAttachmentFace.vue";

defineProps<{
  attachment: ConversationAttachmentPresentation;
  unresolved?: boolean;
}>();
const emit = defineEmits<{ remove: [key: string]; preview: [key: string] }>();
</script>

<template>
  <div
    class="group relative flex max-w-full shrink-0 overflow-hidden rounded-lg border border-stroke-divider bg-surface-raised transition-[border-color,box-shadow] duration-150 focus-within:border-stroke-focus hover:border-stroke-default-hover hover:shadow-control"
    role="group"
    :aria-label="`${$t(attachment.kind === 'node' ? 'spaces.conversation.attachments.canvasReference.shortLabel' : 'spaces.conversation.attachments.localFile.shortLabel')}: ${attachment.title}`"
  >
    <button
      type="button"
      class="flex min-w-0 text-left focus-visible:outline-2 focus-visible:outline-stroke-focus"
      :title="attachment.title"
      :aria-label="
        unresolved
          ? attachment.title
          : $t('spaces.conversation.attachments.preview', { name: attachment.title })
      "
      :disabled="unresolved"
      @click="emit('preview', attachment.key)"
    >
      <SpaceConversationAttachmentFace
        :title="attachment.title"
        :image="attachment.isImage"
        :thumbnail="attachment.previewSource"
        :linked="attachment.kind === 'node'"
      />
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
