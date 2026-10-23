<script setup lang="ts">
import { AlertCircle } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { WaitReason } from "../../contracts/generated/WaitReason";

const props = defineProps<{
  state?: string | undefined;
  waitReason?: WaitReason | null | undefined;
  error?: string | undefined;
}>();
const emit = defineEmits<{ resume: []; retryModel: [] }>();
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

<!--
  Everything the exchange has to say that is not part of the transcript: why it is waiting, that it
  is paused, and what failed. It sits against the composer rather than inside the scroller, because
  a person who has scrolled up to read must not be the last to know that the answer stopped.
-->
<template>
  <div
    v-if="state === 'waiting' && recoveryMessage"
    class="shrink-0 space-y-2 border-t border-stroke-divider px-3 py-2 text-caption text-fg-warning"
  >
    <p>{{ recoveryMessage }}</p>
    <p>{{ t("spaces.conversation.tasks.modelRetryCost") }}</p>
    <UiButton size="sm" variant="secondary" @click="emit('retryModel')">{{
      t("spaces.conversation.tasks.retryModel")
    }}</UiButton>
  </div>
  <div
    v-else-if="state === 'paused'"
    class="flex shrink-0 items-center gap-2 border-t border-stroke-divider px-3 py-2 text-caption text-fg-warning"
  >
    <span class="min-w-0 flex-1">{{ t("spaces.conversation.tasks.paused") }}</span>
    <UiButton size="sm" variant="secondary" @click="emit('resume')">{{
      t("spaces.conversation.tasks.resume")
    }}</UiButton>
  </div>
  <p
    v-else-if="state === 'waiting'"
    class="shrink-0 border-t border-stroke-divider px-3 py-2 text-caption text-fg-secondary"
  >
    {{ toolWaitMessage ?? t("spaces.conversation.tasks.waitingForResult") }}
  </p>
  <p
    v-if="error"
    role="alert"
    class="flex shrink-0 items-start gap-2 border-t border-stroke-danger/30 bg-surface-danger-subtle px-3 py-2 text-caption text-fg-danger"
  >
    <AlertCircle class="mt-0.5 size-3.5 shrink-0" />
    <span class="min-w-0 wrap-break-word">{{ error }}</span>
  </p>
</template>
