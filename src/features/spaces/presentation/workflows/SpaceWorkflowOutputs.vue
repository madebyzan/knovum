<script setup lang="ts">
import { useI18n } from "vue-i18n";

import type { WorkflowCaptureCandidate } from "../../workflows/captured-workflow-ipc";

defineProps<{ outputs: WorkflowCaptureCandidate["outputSlots"] }>();
const { t } = useI18n();
</script>

<template>
  <section class="space-y-1">
    <h3 class="text-caption font-semibold text-foreground-secondary">
      {{ t("spaces.creation.workflow.outputs.title") }}
    </h3>
    <ul class="space-y-1">
      <li v-for="output in outputs" :key="output.id" class="flex min-w-0 items-baseline gap-2 text-caption">
        <span class="min-w-0 flex-1 truncate text-foreground">{{ output.name }}</span>
        <span class="shrink-0 text-foreground-tertiary">{{ t(`spaces.nodeTypes.${output.kind}`) }}</span>
      </li>
    </ul>
  </section>
</template>
