<script setup lang="ts">
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { WorkflowDraft } from "../../workflows/workflow-client";
import SpaceWorkflowNodeEditor from "./SpaceWorkflowNodeEditor.vue";

const draft = defineModel<WorkflowDraft>({ required: true });
const emit = defineEmits<{
  remove: [nodeId: string];
  connect: [sourceNodeId: string, targetNodeId: string];
}>();

function predecessorOptions(nodeId: string) {
  return draft.value.definition.nodes
    .filter(({ id }) => id !== nodeId)
    .map(({ id, kind }) => ({ value: id, label: `${id} · ${kind}` }));
}

function incoming(nodeId: string) {
  return draft.value.definition.edges.filter(({ targetNodeId }) => targetNodeId === nodeId);
}

function removeEdge(edgeId: string) {
  draft.value.definition.edges = draft.value.definition.edges.filter(({ id }) => id !== edgeId);
}
</script>

<template>
  <ol class="space-y-2 p-2">
    <li
      v-for="(node, index) in draft.definition.nodes"
      :key="node.id"
      class="rounded-control border border-border/70 bg-raised p-2"
    >
      <div class="mb-2 flex items-center gap-2">
        <span class="w-5 text-micro text-foreground-tertiary tabular-nums">{{ index + 1 }}</span>
        <span class="min-w-0 flex-1 truncate text-label font-medium">{{ node.id }}</span>
        <span class="text-micro text-foreground-secondary">{{ node.kind }}</span>
        <UiButton
          v-if="node.kind !== 'input' && node.kind !== 'output'"
          size="sm"
          variant="ghost"
          @click="emit('remove', node.id)"
        >
          Remove
        </UiButton>
      </div>
      <div v-if="node.kind !== 'input'" class="space-y-1">
        <div v-for="edge in incoming(node.id)" :key="edge.id" class="text-meta flex items-center gap-2">
          <span class="min-w-0 flex-1 truncate"
            >{{ edge.sourceNodeId }}{{ edge.branch ? ` · ${edge.branch}` : "" }}</span
          >
          <UiButton size="sm" variant="ghost" @click="removeEdge(edge.id)">Disconnect</UiButton>
        </div>
        <UiSelect
          model-value=""
          label="Add dependency"
          :options="predecessorOptions(node.id)"
          size="sm"
          @update:model-value="emit('connect', $event, node.id)"
        />
      </div>
      <SpaceWorkflowNodeEditor v-model="draft.definition.nodes[index]!" class="mt-2" />
    </li>
  </ol>
</template>
