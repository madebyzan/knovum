<script setup lang="ts">
import type { AssistantMessageAttachment } from "../../contracts/generated/AssistantMessageAttachment";
import SpaceConversationSentAttachment from "./SpaceConversationSentAttachment.vue";

defineProps<{ attachments: readonly AssistantMessageAttachment[] }>();
</script>

<template>
  <div class="flex max-w-full flex-wrap justify-end gap-2">
    <SpaceConversationSentAttachment
      v-for="(attachment, index) in attachments"
      :key="index"
      :attachment="attachment"
    />
  </div>
</template>
