<script setup lang="ts">
import type { ConversationMessageAttachment } from "../../contracts/generated/ConversationMessageAttachment";
import { useSpaceStore } from "../../session/space-store";
import SpaceConversationAttachmentFace from "./SpaceConversationAttachmentFace.vue";

const props = defineProps<{ attachment: ConversationMessageAttachment }>();
const store = useSpaceStore();

function openSource() {
  if (props.attachment.source === "node") store.requestReveal([props.attachment.nodeId], "focus");
}
</script>

<template>
  <component
    :is="attachment.source === 'node' ? 'button' : 'div'"
    :type="attachment.source === 'node' ? 'button' : undefined"
    class="flex max-w-full shrink-0 overflow-hidden rounded-lg border border-stroke-divider bg-surface-raised transition-colors duration-100 focus-visible:outline-2 focus-visible:outline-stroke-focus enabled:hover:border-stroke-default-hover enabled:hover:bg-surface-control-hover"
    :aria-label="attachment.title"
    :title="attachment.title"
    @click="openSource"
  >
    <SpaceConversationAttachmentFace
      :title="attachment.title"
      :image="attachment.kind === 'image'"
      :thumbnail="attachment.thumbnail"
      :linked="attachment.source === 'node'"
    />
  </component>
</template>
