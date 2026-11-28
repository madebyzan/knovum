<script setup lang="ts">
import { AlertTriangle, LoaderCircle, Mic, Radio } from "@lucide/vue";

defineProps<{
  loading: boolean;
  recording: boolean;
  paused: boolean;
  hasProblem: boolean;
  elapsedLabel: string;
  levels: readonly number[];
  statusText: string;
}>();
</script>

<template>
  <span class="grid size-6 shrink-0 place-items-center">
    <LoaderCircle v-if="loading" class="size-3.5 animate-spin text-fg-secondary" />
    <Radio
      v-else-if="recording"
      class="size-3.5"
      :class="paused ? 'text-fg-tertiary' : 'text-fg-danger'"
      :stroke-width="1.7"
    />
    <AlertTriangle v-else-if="hasProblem" class="size-3.5 text-fg-warning" />
    <Mic v-else class="size-3.5 text-fg-secondary" />
  </span>
  <div class="ml-2 min-w-0 flex-1">
    <div class="flex items-center gap-2">
      <span class="text-body leading-5 font-medium tabular-nums">{{ elapsedLabel }}</span>
      <span v-if="recording" class="flex h-4 items-center gap-0.5" aria-hidden="true">
        <span
          v-for="(level, index) in levels"
          :key="index"
          class="w-0.5 rounded-full transition-[height] duration-75"
          :class="paused ? 'bg-stroke-strong' : 'bg-fg-danger/80'"
          :style="{ height: `${Math.round(3 + level * 13)}px` }"
        />
      </span>
    </div>
    <div role="status" class="truncate text-caption-sm leading-4 text-fg-tertiary">{{ statusText }}</div>
  </div>
</template>
