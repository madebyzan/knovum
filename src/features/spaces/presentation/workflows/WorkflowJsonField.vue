<script setup lang="ts">
import { ref, watch } from "vue";

import UiTextarea from "../../../../shared/ui/UiTextarea.vue";

const props = defineProps<{ label: string }>();
const value = defineModel<unknown>({ required: true });
const source = ref(JSON.stringify(value.value, null, 2));
const error = ref("");

watch(value, (next) => {
  const encoded = JSON.stringify(next, null, 2);
  if (encoded !== source.value) source.value = encoded;
});

function commit() {
  try {
    value.value = JSON.parse(source.value) as unknown;
    error.value = "";
  } catch (reason) {
    error.value = reason instanceof Error ? reason.message : String(reason);
  }
}
</script>

<template>
  <label class="col-span-2 space-y-1">
    <span class="text-micro text-foreground-secondary">{{ props.label }}</span>
    <UiTextarea v-model="source" :rows="4" class="font-mono text-micro" @blur="commit" />
    <span v-if="error" class="block text-micro text-danger">{{ error }}</span>
  </label>
</template>
