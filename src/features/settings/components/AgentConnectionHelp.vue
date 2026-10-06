<script setup lang="ts">
import { useI18n } from "vue-i18n";

import type { ExternalAgentDescriptor } from "../../../shared/contracts/generated/ExternalAgentDescriptor";
import UiPopover from "../../../shared/ui/UiPopover.vue";
import AgentConnectionRequirements from "./AgentConnectionRequirements.vue";

defineProps<{ agent: ExternalAgentDescriptor; label: string }>();
const { t } = useI18n();
</script>

<template>
  <UiPopover align="end" class="w-80 max-w-[calc(100vw-2rem)] p-3">
    <template #trigger>
      <button
        type="button"
        :aria-label="t('settings.agents.connectionHelp', { name: agent.displayName })"
        class="decoration-border-divider shrink-0 rounded-md text-caption text-fg-secondary underline underline-offset-2 hover:text-fg-primary hover:decoration-current focus-visible:outline-2 focus-visible:outline-stroke-accent"
      >
        {{ label }}
      </button>
    </template>
    <AgentConnectionRequirements :agent="agent" />
  </UiPopover>
</template>
