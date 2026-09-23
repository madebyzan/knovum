<script setup lang="ts">
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import type { AgentKind } from "../../../../foundation/ai/agent-models";
import UiBadge from "../../../../shared/ui/UiBadge.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect, { type SelectOption } from "../../../../shared/ui/UiSelect.vue";

defineProps<{
  selectedId: string;
  selectedState?: string | undefined;
  conversationOptions: readonly SelectOption[];
  agentOptions: readonly SelectOption[];
  busy: boolean;
}>();
const emit = defineEmits<{
  select: [id: string];
  start: [input: { runtimeKind: AgentKind; title: string; maxDurationSeconds: number }];
  close: [];
  remove: [];
}>();
const { t } = useI18n();
const runtimeKind = shallowRef("");
const title = shallowRef("");
const maxDurationSeconds = shallowRef("3600");
const canStart = computed(() => {
  const duration = Number(maxDurationSeconds.value);
  return Boolean(
    runtimeKind.value &&
    title.value.trim() &&
    Number.isInteger(duration) &&
    duration >= 30 &&
    duration <= 7_200,
  );
});

function start() {
  if (!canStart.value) return;
  emit("start", {
    runtimeKind: runtimeKind.value as AgentKind,
    title: title.value.trim(),
    maxDurationSeconds: Number(maxDurationSeconds.value),
  });
}
</script>

<template>
  <UiSelect
    :model-value="selectedId"
    :label="t('spaces.assistant.externalConversations')"
    :options="conversationOptions"
    @update:model-value="emit('select', $event)"
  />
  <div class="grid grid-cols-2 gap-1.5">
    <UiSelect v-model="runtimeKind" :label="t('spaces.assistant.externalRuntime')" :options="agentOptions" />
    <UiInput v-model="title" :placeholder="t('spaces.assistant.externalConversationTitle')" />
  </div>
  <UiInput
    v-model="maxDurationSeconds"
    type="number"
    min="30"
    max="7200"
    :label="t('spaces.assistant.maxDurationSeconds')"
  />
  <div class="flex items-center gap-2">
    <UiButton size="sm" :disabled="busy || !canStart" @click="start">
      {{ t("spaces.assistant.startExternalConversation") }}
    </UiButton>
    <UiBadge v-if="selectedState" size="sm">{{ selectedState }}</UiBadge>
    <UiButton
      v-if="selectedState === 'connected'"
      size="sm"
      variant="dangerSubtle"
      :disabled="busy"
      @click="emit('close')"
    >
      {{ t("spaces.assistant.closeExternalConversation") }}
    </UiButton>
    <UiButton
      v-if="selectedState && selectedState !== 'connected'"
      size="sm"
      variant="dangerSubtle"
      :disabled="busy"
      @click="emit('remove')"
    >
      {{ t("spaces.assistant.deleteExternalConversation") }}
    </UiButton>
  </div>
</template>
