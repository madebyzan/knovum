<script setup lang="ts">
import type { WorkflowDefinition } from "../../workflows/workflow-client";

const limits = defineModel<WorkflowDefinition["limits"]>({ required: true });
const fieldClass =
  "h-(--knovum-control-height-md) w-full rounded-control border border-border bg-control px-2.5 text-label text-foreground";
const fields = [
  ["maxExpandedSteps", "Expanded steps"],
  ["maxParallelism", "Parallel steps"],
  ["maxDurationSeconds", "Duration seconds"],
  ["maxStepAttempts", "Step attempts"],
  ["maxTotalTokens", "Model tokens"],
  ["maxInputBytes", "Input bytes"],
  ["maxOutputBytes", "Output bytes"],
] as const;
</script>

<template>
  <div class="grid grid-cols-2 gap-2 p-2">
    <label v-for="field in fields" :key="field[0]" class="space-y-1 text-micro text-foreground-secondary">
      <span>{{ field[1] }}</span>
      <input v-model.number="limits[field[0]]" type="number" min="1" :class="fieldClass" />
    </label>
  </div>
</template>
