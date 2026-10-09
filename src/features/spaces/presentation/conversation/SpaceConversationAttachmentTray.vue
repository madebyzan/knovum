<script setup lang="ts">
import { computed } from "vue";

import type { ConversationAttachmentPresentation } from "../../conversation/conversation-attachment-presentation";
import SpaceConversationAttachment from "./SpaceConversationAttachment.vue";

const props = defineProps<{
  attachments: readonly ConversationAttachmentPresentation[];
  unresolvedFileNames: readonly string[];
}>();
const emit = defineEmits<{
  discardReference: [name: string];
  remove: [key: string];
  preview: [key: string];
}>();
const unresolvedAttachments = computed<ConversationAttachmentPresentation[]>(() =>
  props.unresolvedFileNames.map((name) => ({
    key: `unresolved:${name}`,
    kind: "file",
    title: name,
    isImage: false,
    previewSource: "",
  })),
);
</script>

<template>
  <div
    v-if="attachments.length || unresolvedAttachments.length"
    class="min-w-0 overflow-x-auto px-3 pt-3 pb-1 focus-visible:outline-2 focus-visible:outline-stroke-focus"
    role="group"
    tabindex="0"
    :aria-label="$t('spaces.conversation.attachments.attached')"
  >
    <div class="flex w-max gap-2">
      <SpaceConversationAttachment
        v-for="attachment in attachments"
        :key="attachment.key"
        :attachment="attachment"
        @remove="emit('remove', $event)"
        @preview="emit('preview', $event)"
      />
      <SpaceConversationAttachment
        v-for="attachment in unresolvedAttachments"
        :key="attachment.key"
        :attachment="attachment"
        unresolved
        @remove="emit('discardReference', attachment.title)"
      />
    </div>
  </div>
</template>
