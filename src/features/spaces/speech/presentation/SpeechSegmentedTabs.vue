<script setup lang="ts">
import { TabsList, TabsTrigger } from "reka-ui";

export interface SpeechSegmentedTabItem {
  value: string;
  label: string;
  badge?: string | number | undefined;
  invalid?: boolean | undefined;
}

defineProps<{ label: string; items: readonly SpeechSegmentedTabItem[] }>();
</script>

<template>
  <TabsList :aria-label="label" class="flex gap-0.5 rounded-lg bg-surface-control p-0.5">
    <TabsTrigger
      v-for="item in items"
      :key="item.value"
      :value="item.value"
      class="flex h-(--knovum-control-height-sm) min-w-0 flex-1 items-center justify-center gap-1.5 rounded-md border border-transparent px-2 text-caption font-medium text-fg-secondary transition-[color,background-color,box-shadow] duration-100 ease-standard hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus data-[state=active]:border-stroke-default data-[state=active]:bg-surface-raised data-[state=active]:text-fg-primary data-[state=active]:shadow-control"
    >
      <span class="truncate">{{ item.label }}</span>
      <span
        v-if="item.badge !== undefined"
        class="rounded px-1 py-px text-caption-sm text-fg-tertiary tabular-nums"
        >{{ item.badge }}</span
      >
      <span v-if="item.invalid" class="size-1.5 shrink-0 rounded-full bg-surface-danger" aria-hidden="true" />
    </TabsTrigger>
  </TabsList>
</template>
