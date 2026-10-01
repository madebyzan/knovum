<script setup lang="ts">
import { cloneDeep } from "es-toolkit";
import { useI18n } from "vue-i18n";

import type { ExecutionTarget, GenerationPreset } from "../../../../foundation/ai/schema/models";
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
    target: cloneDeep(props.target),
    parameters: cloneDeep(props.parameters),
  };
}
</script>

<template>
  <CreationLibraryRecordField
    :label="t('spaces.creation.shared.presets.label')"
    :empty-label="t('spaces.creation.shared.presets.empty')"
    :placeholder="t('spaces.creation.shared.presets.select')"
    :save-label="t('spaces.creation.shared.presets.save')"
    :saved-success-message="t('spaces.creation.shared.presets.saved')"
    :save-disabled="!target || !name.trim()"
    :filter-records="(library) => library.generationPresets.filter((preset) => preset.actionId === actionId)"
    :create-record="createPreset"
    :add-record-to-library="(library, preset) => library.generationPresets.push(preset)"
    @apply="(preset) => emit('apply', preset)"
  />
</template>
