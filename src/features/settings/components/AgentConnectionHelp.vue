<script setup lang="ts">
import { useI18n } from "vue-i18n";

import type { AgentDescriptor } from "../../../foundation/ai/agent-models";
import UiPopover from "../../../shared/ui/UiPopover.vue";
import AgentConnectionDetails from "./AgentConnectionDetails.vue";

defineProps<{ agent: AgentDescriptor; label: string }>();
const { t } = useI18n();
</script>

<template>
  <UiPopover align="end" class="w-80 max-w-[calc(100vw-2rem)] p-3">
    <template #trigger>
      <button
        type="button"
        :aria-label="t('settings.ai.agentConnectionHelp', { name: agent.displayName })"
        class="shrink-0 rounded-control text-caption text-foreground-secondary underline decoration-divider underline-offset-2 hover:text-foreground hover:decoration-current focus-visible:outline-2 focus-visible:outline-accent"
      >
        {{ label }}
      </button>
    </template>
    <AgentConnectionDetails :agent="agent" />
  </UiPopover>
</template>
