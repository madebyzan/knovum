<script setup lang="ts">
import { Handle, Position, VueFlow, type Connection, type Edge, type Node } from "@vue-flow/core";
import { computed, shallowRef } from "vue";

import UiInput from "../../../../shared/ui/UiInput.vue";
import type { WorkflowDraft } from "../../workflows/workflow-client";

const draft = defineModel<WorkflowDraft>({ required: true });
const search = shallowRef("");
const nodes = computed<Node[]>(() =>
  draft.value.definition.nodes.map((node, index) => ({
    id: node.id,
    label: `${index + 1}. ${node.kind}`,
    data: {
      kind: node.kind,
      input: inputType(node),
      output: outputType(node),
      condition: node.kind === "condition",
    },
    class:
      search.value && !`${node.id} ${node.kind}`.toLowerCase().includes(search.value.toLowerCase())
        ? "opacity-30"
        : "",
    position: draft.value.definition.layout[node.id] ?? { x: 40 + index * 180, y: 80 },
  })),
);
const edges = computed<Edge[]>(() =>
  draft.value.definition.edges.map((edge) => ({
    id: edge.id,
    source: edge.sourceNodeId,
    target: edge.targetNodeId,
    ...(edge.branch ? { sourceHandle: edge.branch, label: edge.branch } : {}),
  })),
);

function move(event: { node: Node }) {
  draft.value.definition.layout[event.node.id] = { ...event.node.position };
}

function connect(connection: Connection) {
  if (!connection.source || !connection.target) return;
  draft.value.definition.edges.push({
    id: crypto.randomUUID(),
    sourceNodeId: connection.source,
    sourcePort: "output",
    targetNodeId: connection.target,
    targetPort: "input",
    branch:
      connection.sourceHandle === "true" || connection.sourceHandle === "false"
        ? connection.sourceHandle
        : null,
  });
}

function inputType(node: WorkflowDraft["definition"]["nodes"][number]) {
  if (node.kind === "input") return null;
  if (node.kind === "output") return node.inputType;
  if (node.kind === "for_each") return "collection";
  return "value";
}

function outputType(node: WorkflowDraft["definition"]["nodes"][number]) {
  if (node.kind === "output") return null;
  if (node.kind === "input") return node.outputType;
  if (node.kind === "condition") return "boolean";
  if (node.kind === "for_each") return "collection";
  return "value";
}
</script>

<template>
  <div class="relative min-h-72 bg-canvas">
    <UiInput v-model="search" class="absolute top-3 left-3 z-10 w-52" placeholder="Search steps" />
    <VueFlow
      class="h-full"
      :nodes="nodes"
      :edges="edges"
      fit-view-on-init
      :nodes-draggable="true"
      :nodes-connectable="true"
      @node-drag-stop="move"
      @connect="connect"
    >
      <template #node-default="{ label, data }">
        <Handle v-if="data.input" type="target" :position="Position.Left" />
        <div class="min-w-32 px-3 py-2">
          <p class="text-label font-medium">{{ label }}</p>
          <p class="text-micro text-foreground-tertiary">
            {{ data.input ?? "start" }} → {{ data.output ?? "end" }}
          </p>
        </div>
        <template v-if="data.condition">
          <Handle id="true" type="source" :position="Position.Right" :style="{ top: '38%' }" />
          <Handle id="false" type="source" :position="Position.Right" :style="{ top: '72%' }" />
        </template>
        <Handle v-else-if="data.output" type="source" :position="Position.Right" />
      </template>
    </VueFlow>
  </div>
</template>
