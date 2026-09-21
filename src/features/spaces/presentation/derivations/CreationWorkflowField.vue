<script setup lang="ts">
import { useI18n } from "vue-i18n";

import type { CreationWorkflow } from "../../../../foundation/ai/ai-models";
import CreationLibraryRecordField from "./CreationLibraryRecordField.vue";

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
  <CreationLibraryRecordField
    :label="t('spaces.creation.workflow')"
    :empty-label="t('spaces.creation.noWorkflows')"
    :placeholder="t('spaces.creation.selectWorkflow')"
    :save-label="t('spaces.creation.saveWorkflow')"
    :saved-success-message="t('spaces.creation.workflowSaved')"
    :save-disabled="!name.trim() || !complete"
    :filter-records="
      (library) =>
        library.creationWorkflows.filter(
          (workflow) => workflow.workflowId === workflowId && workflow.workflowVersion === workflowVersion,
        )
    "
    :create-record="createWorkflow"
    :add-record-to-library="(library, workflow) => library.creationWorkflows.push(workflow)"
    @apply="(workflow) => emit('apply', workflow)"
  />
</template>
