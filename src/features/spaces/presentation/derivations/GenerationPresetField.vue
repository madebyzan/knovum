<script setup lang="ts">
import { useI18n } from "vue-i18n";

import type { ExecutionTarget, GenerationPreset } from "../../../../foundation/ai/ai-models";
import PresetSelectField from "./PresetSelectField.vue";

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
  <PresetSelectField
    :label="t('spaces.creation.preset')"
    :empty-label="t('spaces.creation.noPresets')"
    :placeholder="t('spaces.creation.selectPreset')"
    :save-label="t('spaces.creation.savePreset')"
    :saved-success-message="t('spaces.creation.presetSaved')"
    :save-disabled="!target || !name.trim()"
    :filter-items="(s) => s.generationPresets.filter((p) => p.actionId === actionId)"
    :create-item="createPreset"
    :add-item-to-settings="(s, item) => s.generationPresets.push(item)"
    @apply="(preset) => emit('apply', preset)"
  />
</template>
