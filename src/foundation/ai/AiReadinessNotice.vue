<script setup lang="ts">
import { CircleAlert, Download } from "@lucide/vue";
import { computed } from "vue";

import type { AiReadiness } from "./ai-readiness";
import { describeAiReadiness } from "./ai-readiness-prompt";
import { openAiSettings } from "./ai-settings-navigation";

const props = defineProps<{ readiness: AiReadiness }>();
const prompt = computed(() => describeAiReadiness(props.readiness, "before"));
</script>

<template>
  <p
    v-if="prompt"
    class="flex items-start gap-1.5 text-caption leading-relaxed"
    :class="prompt.tone === 'attention' ? 'text-fg-warning' : 'text-fg-secondary'"
    :role="prompt.tone === 'attention' ? 'alert' : 'status'"
    data-testid="ai-readiness-notice"
  >
    <CircleAlert v-if="prompt.tone === 'attention'" class="mt-0.5 size-3.5 shrink-0" :stroke-width="1.6" />
    <Download v-else class="mt-0.5 size-3.5 shrink-0" :stroke-width="1.6" />
    <span class="min-w-0">
      {{ prompt.message }}
      <button
        v-if="prompt.action"
        type="button"
        class="rounded-sm font-medium text-fg-accent outline-none hover:underline focus-visible:ring-2 focus-visible:ring-stroke-focus"
        @click="openAiSettings(prompt.action.pane)"
      >
        {{ prompt.action.label }}
      </button>
    </span>
  </p>
</template>
