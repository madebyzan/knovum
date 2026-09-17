<script setup lang="ts">
import { useI18n } from "vue-i18n";

import type { CreationWorkflow } from "../../../../foundation/ai/ai-models";
import PresetSelectField from "./PresetSelectField.vue";

interface CreationWorkflowFieldProps {
  workflowId: string;
  workflowVersion: number;
  name: string;
  targets: CreationWorkflow["targets"];
  parameters: CreationWorkflow["parameters"];
  complete: boolean;
}

const props = defineProps<CreationWorkflowFieldProps>();
const emit = defineEmits<{ apply: [workflow: CreationWorkflow] }>();
const { t } = useI18n();

function createWorkflow(): CreationWorkflow | undefined {
  if (!props.name.trim() || !props.complete) return undefined;
  return {
    id: crypto.randomUUID(),
    name: props.name.trim(),
    workflowId: props.workflowId,
    workflowVersion: props.workflowVersion,
    targets: structuredClone(props.targets),
    parameters: structuredClone(props.parameters),
  };
}
</script>

<template>
  <PresetSelectField
    :label="t('spaces.creation.workflow')"
    :empty-label="t('spaces.creation.noWorkflows')"
    :placeholder="t('spaces.creation.selectWorkflow')"
    :save-label="t('spaces.creation.saveWorkflow')"
    :saved-success-message="t('spaces.creation.workflowSaved')"
    :save-disabled="!name.trim() || !complete"
    :filter-items="
      (s) =>
        s.creationWorkflows.filter(
          (item) => item.workflowId === workflowId && item.workflowVersion === workflowVersion,
        )
    "
    :create-item="createWorkflow"
    :add-item-to-settings="(s, item) => s.creationWorkflows.push(item)"
    @apply="(workflow) => emit('apply', workflow)"
  />
</template>
