<script setup lang="ts">
import { useI18n } from "vue-i18n";

import type { ExecutionTarget, GenerationPreset } from "../../../../foundation/ai/ai-models";
import CreationLibraryRecordField from "./CreationLibraryRecordField.vue";

interface GenerationPresetFieldProps {
  actionId: string;
  name: string;
  target?: ExecutionTarget | undefined;
  parameters: GenerationPreset["parameters"];
}

const props = defineProps<GenerationPresetFieldProps>();
const emit = defineEmits<{ apply: [preset: GenerationPreset] }>();
const { t } = useI18n();

function createPreset(): GenerationPreset | undefined {
  if (!props.target || !props.name.trim()) return undefined;
  return {
    id: crypto.randomUUID(),
    name: props.name.trim(),
    actionId: props.actionId,
    target: structuredClone(props.target),
    parameters: structuredClone(props.parameters),
  };
}
</script>

<template>
  <CreationLibraryRecordField
    :label="t('spaces.creation.preset')"
    :empty-label="t('spaces.creation.noPresets')"
    :placeholder="t('spaces.creation.selectPreset')"
    :save-label="t('spaces.creation.savePreset')"
    :saved-success-message="t('spaces.creation.presetSaved')"
    :save-disabled="!target || !name.trim()"
    :filter-records="(library) => library.generationPresets.filter((preset) => preset.actionId === actionId)"
    :create-record="createPreset"
    :add-record-to-library="(library, preset) => library.generationPresets.push(preset)"
    @apply="(preset) => emit('apply', preset)"
  />
</template>
