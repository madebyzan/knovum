<script setup lang="ts">
import { TabsList, TabsRoot, TabsTrigger } from "reka-ui";
import { watch } from "vue";

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
    <TabsList
      :aria-label="label"
      class="sticky top-0 z-10 flex gap-0.5 rounded-panel bg-surface-control p-0.5"
    >
      <TabsTrigger
        v-for="item in items"
        :key="item.value"
        :value="item.value"
        class="flex h-(--knovum-control-height-md) min-w-0 flex-1 items-center justify-center rounded-md border border-transparent px-2 text-label text-fg-secondary transition-colors hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus data-[state=active]:border-stroke-default data-[state=active]:bg-surface-raised data-[state=active]:text-fg-primary data-[state=active]:shadow-control"
        >{{ item.label }}</TabsTrigger
      >
    </TabsList>
    <slot />
  </TabsRoot>
</template>
