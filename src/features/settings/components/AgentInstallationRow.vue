<script setup lang="ts">
import { Terminal } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { AgentDescriptor, AgentInstallation } from "../../../foundation/ai/agent-models";
import UiBadge from "../../../shared/ui/UiBadge.vue";
import UiButton from "../../../shared/ui/UiButton.vue";
import AgentConnectionHelp from "./AgentConnectionHelp.vue";

const props = defineProps<{
  agent: AgentDescriptor;
  installation?: AgentInstallation | undefined;
  enabled: boolean;
  protected: boolean;
  checking: boolean;
  checkFailed: boolean;
}>();
const emit = defineEmits<{ toggle: [enabled: boolean] }>();
const { t } = useI18n();
const needsSetup = computed(() =>
  Boolean(props.installation && !props.installation.executablePath && !props.checking && !props.checkFailed),
);

const statusKey = () => {
  if (props.checking) return "settings.ai.desktopAgentChecking";
  if (props.checkFailed) return "settings.ai.desktopAgentCheckFailed";
  if (!props.installation) return "settings.ai.desktopAgentChecking";
  if (!props.installation.executablePath) {
    return props.installation.detected ? "settings.ai.agentApplicationOnly" : "settings.ai.agentEntryMissing";
  }
  return "settings.ai.agentCommandDetected";
};
</script>

<template>
  <div class="px-4 py-3">
    <div class="flex items-center justify-between gap-4">
      <div class="flex min-w-0 items-center gap-2">
        <Terminal class="size-(--knovum-icon-size-sm) shrink-0 text-foreground-secondary" />
        <span class="truncate text-label text-foreground">{{ agent.displayName }}</span>
      </div>
      <div class="flex items-center gap-2">
        <AgentConnectionHelp v-if="needsSetup" :agent="agent" :label="t(statusKey())" />
        <UiBadge
          v-else
          :variant="installation?.executablePath && !checking && !checkFailed ? 'success' : 'neutral'"
          size="sm"
        >
          {{ t(statusKey()) }}
        </UiBadge>
        <UiButton
          size="sm"
          :variant="enabled ? 'ghost' : 'secondary'"
          :disabled="enabled && protected"
          :title="enabled && protected ? t('settings.ai.desktopAgentInUse') : undefined"
          @click="emit('toggle', !enabled)"
        >
          {{ t(enabled ? "settings.ai.desktopAgentDisable" : "settings.ai.desktopAgentEnable") }}
        </UiButton>
      </div>
    </div>
  </div>
</template>
