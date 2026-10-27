<script setup lang="ts">
import type { ConversationMessageOutput } from "../../contracts/generated/ConversationMessageOutput";
import SpaceConversationSentAttachment from "./SpaceConversationSentAttachment.vue";

defineProps<{ outputs: readonly ConversationMessageOutput[] }>();
</script>

<template>
  <div class="max-w-full space-y-2">
    <p class="text-caption-sm font-medium text-fg-secondary">
      {{ $t("spaces.conversation.outputs.added", { count: outputs.length }) }}
    </p>
    <div class="flex max-w-full gap-2 overflow-x-auto pb-1">
      <SpaceConversationSentAttachment
        v-for="output in outputs"
        :key="output.nodeId"
        :attachment="{ source: 'node', nodeId: output.nodeId, title: output.title, kind: output.kind }"
      />
    </div>
  </div>
</template>
