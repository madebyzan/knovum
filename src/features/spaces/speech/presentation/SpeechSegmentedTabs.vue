<script setup lang="ts">
import { TabsList, TabsTrigger } from "reka-ui";

defineProps<{ label: string; items: readonly { value: string; label: string }[] }>();
</script>

<template>
  <TabsList :aria-label="label" class="flex gap-0.5 rounded-panel bg-surface-control p-0.5">
    <TabsTrigger
      v-for="item in items"
      :key="item.value"
      :value="item.value"
      class="flex h-(--knovum-control-height-md) min-w-0 flex-1 items-center justify-center rounded-md border border-transparent px-2 text-label text-fg-secondary transition-colors hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus data-[state=active]:border-stroke-default data-[state=active]:bg-surface-raised data-[state=active]:text-fg-primary data-[state=active]:shadow-control"
      >{{ item.label }}</TabsTrigger
    >
  </TabsList>
</template>
