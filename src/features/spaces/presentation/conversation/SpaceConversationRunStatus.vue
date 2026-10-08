<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { WaitReason } from "../../contracts/generated/WaitReason";

const props = defineProps<{
  canPause: boolean;
  state?: string | undefined;
  waitReason?: WaitReason | null | undefined;
}>();
const emit = defineEmits<{ pause: []; resume: []; retryModel: [] }>();
const { t } = useI18n();
const recoveryMessage = computed(() => {
  switch (props.waitReason) {
    case "model_outcome_unknown":
      return t("spaces.conversation.tasks.modelOutcomeUnknown");
    case "model_response_uncommitted":
      return t("spaces.conversation.tasks.modelResponseUncommitted");
    case "model_usage_unknown":
      return t("spaces.conversation.tasks.modelUsageUnknown");
    default:
      return undefined;
  }
});
const toolWaitMessage = computed(() => {
  switch (props.waitReason) {
    case "tool_approval":
      return t("spaces.conversation.toolOperations.approval.title");
    case "tool_outcome_unknown":
      return t("spaces.conversation.toolOperations.outcomeUnknown.title");
    case "mcp_elicitation":
      return t("spaces.conversation.interactions.title");
    default:
      return undefined;
  }
});
</script>

<template>
  <div v-if="canPause" class="flex shrink-0 justify-end px-3 py-2">
    <UiButton size="sm" variant="ghost" @click="emit('pause')">{{ t("common.pause") }}</UiButton>
  </div>
  <div
    v-if="state === 'waiting' && recoveryMessage"
    class="shrink-0 space-y-2 px-3 py-2 text-caption text-fg-warning"
  >
    <p>{{ recoveryMessage }}</p>
    <p>{{ t("spaces.conversation.tasks.modelRetryCost") }}</p>
    <UiButton size="sm" variant="secondary" @click="emit('retryModel')">{{
      t("spaces.conversation.tasks.retryModel")
    }}</UiButton>
  </div>
  <div
    v-else-if="state === 'paused'"
    class="flex shrink-0 items-center gap-2 px-3 py-2 text-caption text-fg-warning"
  >
    <span class="min-w-0 flex-1">{{ t("spaces.conversation.tasks.paused") }}</span>
    <UiButton size="sm" variant="secondary" @click="emit('resume')">{{
      t("spaces.conversation.tasks.resume")
    }}</UiButton>
  </div>
  <p v-else-if="state === 'waiting'" class="shrink-0 px-3 py-2 text-caption text-fg-secondary">
    {{ toolWaitMessage ?? t("spaces.conversation.tasks.waitingForResult") }}
  </p>
</template>
