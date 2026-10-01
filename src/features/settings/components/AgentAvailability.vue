<script setup lang="ts">
import { Terminal } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { AgentDescriptor } from "../../../shared/contracts/generated/AgentDescriptor";
import type { AgentInstallation } from "../../../shared/contracts/generated/AgentInstallation";
import UiBadge from "../../../shared/ui/UiBadge.vue";
import UiButton from "../../../shared/ui/UiButton.vue";
import AgentConnectionHelp from "./AgentConnectionHelp.vue";

const props = defineProps<{
  agent: AgentDescriptor;
  installation?: AgentInstallation | undefined;
  enabled: boolean;
  checking: boolean;
  checkFailed: boolean;
}>();
const emit = defineEmits<{ toggle: [enabled: boolean] }>();
const { t } = useI18n();
const needsSetup = computed(() =>
  Boolean(props.installation && !props.installation.executablePath && !props.checking && !props.checkFailed),
);

const statusMessageKey = computed(() => {
  if (props.checking) return "settings.agents.status.checking";
  if (props.checkFailed) return "settings.agents.status.checkFailed";
  if (!props.installation) return "settings.agents.status.checking";
  if (!props.installation.executablePath) {
    return props.installation.detected
      ? "settings.agents.entry.applicationOnly"
      : "settings.agents.entry.missing";
  }
  return "settings.agents.entry.commandDetected";
});
</script>

<template>
  <div class="px-4 py-3">
    <div class="flex items-center justify-between gap-4">
      <div class="flex min-w-0 items-center gap-2">
        <Terminal class="size-3.5 shrink-0 text-fg-secondary" />
        <span class="truncate text-label text-fg-primary">{{ agent.displayName }}</span>
      </div>
      <div class="flex items-center gap-2">
        <AgentConnectionHelp v-if="needsSetup" :agent="agent" :label="t(statusMessageKey)" />
        <UiBadge
          v-else
          :variant="installation?.executablePath && !checking && !checkFailed ? 'success' : 'neutral'"
          size="sm"
        >
          {{ t(statusMessageKey) }}
        </UiBadge>
        <UiButton size="sm" :variant="enabled ? 'ghost' : 'secondary'" @click="emit('toggle', !enabled)">
          {{ t(enabled ? "settings.agents.disable" : "settings.agents.enable") }}
        </UiButton>
      </div>
    </div>
  </div>
</template>
