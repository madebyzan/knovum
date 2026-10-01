<script setup lang="ts">
import { useI18n } from "vue-i18n";

import type { CaptureCandidate } from "../../contracts/generated/CaptureCandidate";

defineProps<{ outputs: CaptureCandidate["outputSlots"] }>();
const { t } = useI18n();
</script>

<template>
  <section class="space-y-1">
    <h3 class="text-caption font-semibold text-fg-secondary">
      {{ t("spaces.creation.workflow.outputs.title") }}
    </h3>
    <ul class="space-y-1">
      <li v-for="output in outputs" :key="output.id" class="flex min-w-0 items-baseline gap-2 text-caption">
        <span class="min-w-0 flex-1 truncate text-fg-primary">{{ output.name }}</span>
        <span class="shrink-0 text-fg-tertiary">{{ t(`spaces.nodeTypes.${output.kind}`) }}</span>
      </li>
    </ul>
  </section>
</template>
