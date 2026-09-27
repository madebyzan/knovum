<script setup lang="ts">
import UiInput from "../../../shared/ui/UiInput.vue";
import UiNumberField from "../../../shared/ui/UiNumberField.vue";
import UiSwitch from "../../../shared/ui/UiSwitch.vue";

type ParameterValue = string | number | boolean;
const parameters = defineModel<Record<string, ParameterValue>>({ required: true });

function setValue(key: string, value: ParameterValue | undefined) {
  if (value !== undefined) parameters.value[key] = value;
}
</script>

<template>
  <div v-if="Object.keys(parameters).length" class="grid gap-2">
    <div
      v-for="(value, key) in parameters"
      :key="key"
      class="grid min-h-(--knovum-control-height-md) grid-cols-[minmax(8rem,1fr)_minmax(12rem,2fr)] items-center gap-3"
    >
      <span class="truncate text-caption-md text-fg-secondary">{{ key }}</span>
      <UiSwitch
        v-if="typeof value === 'boolean'"
        :model-value="value"
        :label="key"
        @update:model-value="setValue(key, $event)"
      />
      <UiNumberField
        v-else-if="typeof value === 'number'"
        :model-value="value"
        size="sm"
        @update:model-value="setValue(key, $event)"
      />
      <UiInput v-else :model-value="value" size="sm" @update:model-value="setValue(key, $event)" />
    </div>
  </div>
</template>
