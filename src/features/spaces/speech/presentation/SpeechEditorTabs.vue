<script setup lang="ts">
import { TabsRoot } from "reka-ui";
import { watch } from "vue";

import SpeechSegmentedTabs from "./SpeechSegmentedTabs.vue";

const page = defineModel<string>({ required: true });
const props = defineProps<{
  label: string;
  items: readonly { value: string; label: string }[];
  invalidPage?: string | undefined;
}>();
watch(
  () => props.invalidPage,
  (value) => {
    if (value) page.value = value;
  },
  { immediate: true },
);
</script>

<template>
  <TabsRoot v-model="page" class="space-y-3">
    <div
      class="sticky top-0 z-10 -mx-3 -mt-3 border-b border-stroke-divider bg-surface-overlay px-3 pt-3 pb-2"
    >
      <SpeechSegmentedTabs :label="label" :items="items" />
    </div>
    <slot />
  </TabsRoot>
</template>
