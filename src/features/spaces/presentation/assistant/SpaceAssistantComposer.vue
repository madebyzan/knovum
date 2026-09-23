<script setup lang="ts">
import { ArrowUp, CircleStop } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { ExecutionTarget } from "../../../../foundation/ai/ai-models";
import AiExecutionTargetField from "../../../../foundation/ai/AiExecutionTargetField.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { AssistantMode } from "../../assistant/assistant-client";

const prompt = defineModel<string>({ required: true });
const mode = defineModel<AssistantMode>("mode", { required: true });
const target = defineModel<ExecutionTarget | undefined>("target");
const maxTurns = defineModel<string>("maxTurns", { required: true });
const maxToolCalls = defineModel<string>("maxToolCalls", { required: true });
const maxTotalTokens = defineModel<string>("maxTotalTokens", { required: true });
const maxDurationSeconds = defineModel<string>("maxDurationSeconds", { required: true });
const props = defineProps<{ busy: boolean }>();
const emit = defineEmits<{ send: []; cancel: [] }>();
const { t } = useI18n();
const canSend = computed(
  () =>
    Boolean(prompt.value.trim() && target.value) &&
    within(maxTurns.value, 1, 32) &&
    within(maxToolCalls.value, 1, 128) &&
    within(maxTotalTokens.value, 1_000, 1_000_000) &&
    within(maxDurationSeconds.value, 30, 7_200) &&
    !props.busy,
);
const modeOptions = computed(() => [
  { value: "ask", label: t("spaces.assistant.mode.ask") },
  { value: "plan", label: t("spaces.assistant.mode.plan") },
  { value: "execute", label: t("spaces.assistant.mode.execute") },
]);

function submit() {
  if (canSend.value) emit("send");
}

function within(value: string, minimum: number, maximum: number) {
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed >= minimum && parsed <= maximum;
}
</script>

<template>
  <footer class="shrink-0 border-t border-divider bg-panel p-2.5">
    <div class="rounded-panel border border-border bg-raised focus-within:border-focus-ring">
      <UiTextarea
        v-model="prompt"
        :rows="3"
        class="min-h-20 resize-none rounded-b-none border-0 bg-transparent px-3 py-2.5"
        :placeholder="$t('spaces.assistant.builtInPlaceholder')"
        @keydown.meta.enter.prevent="submit"
        @keydown.ctrl.enter.prevent="submit"
      />
      <div class="flex min-w-0 items-center gap-1.5 px-2 pb-2">
        <UiSelect
          v-model="mode"
          class="w-24"
          size="sm"
          :label="$t('spaces.assistant.modeLabel')"
          :options="modeOptions"
          :disabled="busy"
        />
        <AiExecutionTargetField
          v-model="target"
          class="min-w-0 flex-1"
          purpose="knowledge"
          compact
          :disabled="busy"
        />
        <UiIconButton v-if="busy" :label="$t('spaces.assistant.stop')" size="sm" @click="emit('cancel')">
          <CircleStop class="size-(--knovum-icon-size-sm)" />
        </UiIconButton>
        <UiIconButton
          v-else
          :label="$t('spaces.assistant.send')"
          size="sm"
          :disabled="!canSend"
          @click="submit"
        >
          <ArrowUp class="size-(--knovum-icon-size-sm)" />
        </UiIconButton>
      </div>
      <details class="border-t border-divider px-2 py-1.5 text-caption text-foreground-secondary">
        <summary class="cursor-pointer select-none">{{ t("spaces.assistant.taskBudget") }}</summary>
        <div class="grid grid-cols-2 gap-1.5 pt-2">
          <label class="space-y-1"
            >{{ t("spaces.assistant.maxTurns") }}<UiInput v-model="maxTurns" type="number" size="sm"
          /></label>
          <label class="space-y-1"
            >{{ t("spaces.assistant.maxToolCalls") }}<UiInput v-model="maxToolCalls" type="number" size="sm"
          /></label>
          <label class="space-y-1"
            >{{ t("spaces.assistant.maxTotalTokens")
            }}<UiInput v-model="maxTotalTokens" type="number" size="sm"
          /></label>
          <label class="space-y-1"
            >{{ t("spaces.assistant.maxDurationSeconds")
            }}<UiInput v-model="maxDurationSeconds" type="number" size="sm"
          /></label>
        </div>
      </details>
    </div>
  </footer>
</template>
