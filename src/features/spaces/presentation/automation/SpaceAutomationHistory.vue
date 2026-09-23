<script setup lang="ts">
import UiBadge from "../../../../shared/ui/UiBadge.vue";
import type { AutomationOccurrence } from "../../automation/automation-client";

defineProps<{ occurrences: readonly AutomationOccurrence[] }>();
</script>

<template>
  <section class="border-t border-divider p-2">
    <h3 class="mb-1 text-label font-medium">{{ $t("spaces.assistant.automationHistory") }}</h3>
    <ol class="max-h-40 overflow-y-auto">
      <li v-for="occurrence in occurrences" :key="occurrence.id" class="flex items-center gap-2 py-1">
        <time class="text-micro text-foreground-tertiary">
          {{ new Date(occurrence.scheduledAt).toLocaleString() }}
        </time>
        <span class="min-w-0 flex-1 truncate text-micro">{{ occurrence.occurrenceKey }}</span>
        <UiBadge size="sm">{{ occurrence.state }}</UiBadge>
      </li>
    </ol>
  </section>
</template>
