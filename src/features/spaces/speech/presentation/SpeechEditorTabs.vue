<script setup lang="ts">
import { TabsRoot } from "reka-ui";
import { watch } from "vue";

import SpeechSegmentedTabs, { type SpeechSegmentedTabItem } from "./SpeechSegmentedTabs.vue";

const page = defineModel<string>({ required: true });
const props = defineProps<{
  label: string;
  items: readonly SpeechSegmentedTabItem[];
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
      class="sticky top-0 z-10 -mx-3 -mt-3 border-b border-stroke-divider bg-surface-overlay/90 px-3 py-1.5 backdrop-blur-md"
    >
      <SpeechSegmentedTabs :label="label" :items="items" />
    </div>
    <slot />
  </TabsRoot>
</template>
