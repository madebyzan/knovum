<script setup lang="ts">
import { useId } from "reka-ui";
import { shallowRef, watch } from "vue";

import UiInput from "../../../../shared/ui/UiInput.vue";
const props = withDefaults(defineProps<{ label: string; value: number; min?: number; max: number }>(), {
  min: 0,
});
const emit = defineEmits<{ change: [milliseconds: number] }>();
const id = useId();
const draft = shallowRef((props.value / 1000).toFixed(3));
const invalid = shallowRef(false);
watch(
  () => props.value,
  (value) => {
    draft.value = (value / 1000).toFixed(3);
    invalid.value = false;
  },
);
function commit() {
  const value = Math.round(Number(draft.value) * 1000);
  invalid.value = !draft.value.trim() || !Number.isFinite(value) || value < props.min || value > props.max;
  if (!invalid.value) emit("change", value);
}
</script>
<template>
  <label :for="id" class="block text-caption text-fg-secondary">
    {{ label }}
    <UiInput
      :id="id"
      v-model="draft"
      type="number"
      size="sm"
      class="mt-1 tabular-nums"
      :min="min / 1000"
      :max="max / 1000"
      step="0.001"
      :aria-invalid="invalid"
      :aria-describedby="invalid ? `${id}-error` : undefined"
      @change="commit"
      @commit.prevent="commit"
    />
    <span v-if="invalid" :id="`${id}-error`" role="alert" class="mt-1 block text-caption-sm text-fg-danger">{{
      $t("spaces.media.videoEditor.timing.invalid", {
        min: (min / 1000).toFixed(3),
        max: (max / 1000).toFixed(3),
      })
    }}</span>
  </label>
</template>
