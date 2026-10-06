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
      return t("spaces.assistant.tasks.modelOutcomeUnknown");
    case "model_response_uncommitted":
      return t("spaces.assistant.tasks.modelResponseUncommitted");
    case "model_usage_unknown":
      return t("spaces.assistant.tasks.modelUsageUnknown");
    default:
      return undefined;
  }
});
</script>

<template>
  <div v-if="canPause" class="flex shrink-0 justify-end px-3 py-2">
    <UiButton size="sm" variant="ghost" @click="emit('pause')">{{
      t("spaces.assistant.tasks.pause")
    }}</UiButton>
  </div>
  <div
    v-if="state === 'waiting' && recoveryMessage"
    class="shrink-0 space-y-2 px-3 py-2 text-caption text-fg-warning"
  >
    <p>{{ recoveryMessage }}</p>
    <p>{{ t("spaces.assistant.tasks.modelRetryCost") }}</p>
    <UiButton size="sm" variant="secondary" @click="emit('retryModel')">{{
      t("spaces.assistant.tasks.retryModel")
    }}</UiButton>
  </div>
  <div
    v-else-if="state === 'paused'"
    class="flex shrink-0 items-center gap-2 px-3 py-2 text-caption text-fg-warning"
  >
    <span class="min-w-0 flex-1">{{ t("spaces.assistant.tasks.paused") }}</span>
    <UiButton size="sm" variant="secondary" @click="emit('resume')">{{
      t("spaces.assistant.tasks.resume")
    }}</UiButton>
  </div>
  <p v-else-if="state === 'waiting'" class="shrink-0 px-3 py-2 text-caption text-fg-secondary">
    {{ t("spaces.assistant.tasks.waitingForResult") }}
  </p>
</template>
