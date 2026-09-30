<script setup lang="ts">
import { spaceNodePreviewSource } from "../../assets/node-preview-source";
import type { AssistantMessageOutput } from "../../contracts/generated/AssistantMessageOutput";
import { useSpaceStore } from "../../session/space-store";
import SpaceConversationSentAttachment from "./SpaceConversationSentAttachment.vue";

defineProps<{ outputs: readonly AssistantMessageOutput[] }>();
const space = useSpaceStore();

function thumbnail(nodeId: string) {
  const node = space.nodes.find(({ id }) => id === nodeId);
  return node ? spaceNodePreviewSource(node.data) || null : null;
}
</script>

<template>
  <div class="max-w-full space-y-2">
    <p class="text-caption-sm font-medium text-fg-secondary">
      {{ $t("spaces.assistant.outputs.added", { count: outputs.length }) }}
    </p>
    <div class="flex max-w-full gap-2 overflow-x-auto pb-1">
      <SpaceConversationSentAttachment
        v-for="output in outputs"
        :key="output.nodeId"
        :attachment="{
          source: 'node',
          nodeId: output.nodeId,
          title: output.title,
          kind: output.kind,
          thumbnail: thumbnail(output.nodeId),
        }"
      />
    </div>
  </div>
</template>
