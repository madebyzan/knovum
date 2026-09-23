<script setup lang="ts">
import { computed, onMounted, shallowRef } from "vue";

import { listSpaceActions, type SpaceActionDefinition } from "../../../../foundation/ai/space-action-catalog";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { WorkflowNode } from "../../workflows/workflow-client";
import WorkflowJsonField from "./WorkflowJsonField.vue";

const node = defineModel<WorkflowNode>({ required: true });
const joinPolicies = ["all_succeeded", "active_branches", "collect_results"].map((value) => ({
  value,
  label: value,
}));
const itemPolicies = ["stop", "collect_errors"].map((value) => ({ value, label: value }));
const publishModes = ["create_resource", "update_expected_revision", "export"].map((value) => ({
  value,
  label: value,
}));
const valueTypes = [
  "value",
  "record",
  "boolean",
  "document",
  "image",
  "audio",
  "video",
  "transcript",
  "subtitle",
  "presentation",
  "resource_revision",
  "collection",
].map((value) => ({ value, label: value }));
const numberFieldClass =
  "h-(--knovum-control-height-md) rounded-control border border-border bg-control px-2.5 text-label text-foreground";
const actions = shallowRef<SpaceActionDefinition[]>([]);
const actionOptions = computed(() =>
  actions.value.map((action) => ({
    value: action.id,
    label: `${action.id} · v${action.version}`,
  })),
);
onMounted(async () => {
  actions.value = await listSpaceActions();
});
</script>

<template>
  <div class="grid grid-cols-2 gap-2">
    <UiSelect
      v-if="node.kind === 'input'"
      v-model="node.outputType"
      label="Output type"
      :options="valueTypes"
    />
    <UiSelect
      v-else-if="node.kind === 'output'"
      v-model="node.inputType"
      label="Input type"
      :options="valueTypes"
    />
    <template v-else-if="node.kind === 'action'">
      <UiSelect v-model="node.actionId" label="Space Action" :options="actionOptions" />
      <input v-model.number="node.actionVersion" :class="numberFieldClass" min="1" type="number" />
      <WorkflowJsonField v-model="node.parameters" label="Parameters" />
    </template>
    <UiInput
      v-else-if="node.kind === 'condition' || node.kind === 'data_transform'"
      v-model="node.expression"
      class="col-span-2"
      placeholder="CEL expression"
    />
    <UiSelect
      v-else-if="node.kind === 'join'"
      v-model="node.policy"
      label="Join policy"
      :options="joinPolicies"
    />
    <template v-else-if="node.kind === 'for_each'">
      <input v-model.number="node.maxItems" :class="numberFieldClass" min="1" max="1000" type="number" />
      <input v-model.number="node.parallelism" :class="numberFieldClass" min="1" type="number" />
      <UiSelect v-model="node.errorPolicy" label="Item error policy" :options="itemPolicies" />
    </template>
    <template v-else-if="node.kind === 'iterate'">
      <input v-model.number="node.maxIterations" :class="numberFieldClass" min="1" max="20" type="number" />
      <UiInput v-model="node.stopExpression" placeholder="Stop CEL expression" />
    </template>
    <template v-else-if="node.kind === 'subworkflow'">
      <UiInput v-model="node.workflowId" placeholder="Workflow ID" />
      <input v-model.number="node.version" :class="numberFieldClass" min="1" type="number" />
    </template>
    <UiInput
      v-else-if="node.kind === 'review'"
      v-model="node.purpose"
      class="col-span-2"
      placeholder="Review purpose"
    />
    <input
      v-else-if="node.kind === 'delay'"
      v-model.number="node.durationSeconds"
      :class="numberFieldClass"
      min="0"
      type="number"
    />
    <template v-else-if="node.kind === 'external_tool'">
      <UiInput v-model="node.bindingId" placeholder="Tool binding ID" />
      <UiInput v-model="node.schemaFingerprint" placeholder="Reviewed schema fingerprint" />
    </template>
    <UiInput
      v-else-if="node.kind === 'delegate_task'"
      v-model="node.runtimeId"
      class="col-span-2"
      placeholder="External runtime ID"
    />
    <template v-else-if="node.kind === 'publish'">
      <UiSelect v-model="node.mode" label="Publication mode" :options="publishModes" />
      <UiInput
        v-if="node.mode === 'export'"
        :model-value="node.destination ?? ''"
        placeholder="Export destination"
        @update:model-value="node.destination = $event || null"
      />
    </template>
    <WorkflowJsonField v-else-if="node.kind === 'human_input'" v-model="node.schema" label="Input schema" />
  </div>
</template>
