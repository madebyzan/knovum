<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { ImageExecutionTarget } from "../../contracts/generated/ImageExecutionTarget";
import type { ImageOperationTargetOption } from "../../contracts/generated/ImageOperationTargetOption";
const props = defineProps<{
  options: readonly ImageOperationTargetOption[];
  execution: ImageExecutionTarget | undefined;
  disabled?: boolean;
}>();
const emit = defineEmits<{ select: [execution: ImageExecutionTarget] }>();
const { t } = useI18n();
const identity = computed(() => JSON.stringify(props.execution));
const value = computed(() => {
  const index = props.options.findIndex((option) => JSON.stringify(option.execution) === identity.value);
  return index < 0 ? "" : String(index);
});
const choices = computed(() =>
  props.options.map((option, index) => ({
    value: String(index),
    label: option.label,
    disabled: option.availability.state === "blocked",
  })),
);
function select(selectedValue: string) {
  const option = props.options[Number(selectedValue)];
  if (option && option.availability.state !== "blocked") emit("select", option.execution);
}
</script>
<template>
  <UiSelect
    :model-value="value"
    :options="choices"
    :disabled="disabled"
    :label="t('spaces.media.imageEditor.operation.execution')"
    @update:model-value="select"
  />
</template>
