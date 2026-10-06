<script setup lang="ts">
import SpaceConversationAttachment from "./SpaceConversationAttachment.vue";

interface Attachment {
  key: string;
  kind: "node" | "file";
  title: string;
  isImage: boolean;
  previewSource: string;
}

defineProps<{ attachments: readonly Attachment[] }>();
const emit = defineEmits<{ remove: [key: string]; preview: [key: string] }>();
</script>

<template>
  <div
    v-if="attachments.length"
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
    </div>
  </div>
</template>
