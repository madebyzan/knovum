<script setup lang="ts">
import { useId } from "reka-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import type { AssistantModelSettings } from "../../../shared/contracts/generated/AssistantModelSettings";
import type { ChatModelProtocol } from "../../../shared/contracts/generated/ChatModelProtocol";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import UiSwitch from "../../../shared/ui/UiSwitch.vue";

const model = defineModel<AssistantModelSettings | null>({ required: true });
const props = defineProps<{
  protocols: readonly ChatModelProtocol[];
  disabled: boolean;
  showErrors: boolean;
}>();
const { t } = useI18n();
const switchId = useId();
const labels: Record<ChatModelProtocol, string> = {
  open_ai_chat: "OpenAI Chat Completions",
  open_ai_responses: "OpenAI Responses",
  anthropic: "Anthropic Messages",
  gemini: "Gemini GenerateContent",
  ollama: "Ollama Chat",
  deep_seek: "DeepSeek Chat",
  groq: "Groq Chat",
};
const options = computed(() => props.protocols.map((value) => ({ value, label: labels[value] })));
const enabled = computed({
  get: () => model.value !== null,
  set: (value: boolean) => {
    if (!value) {
      model.value = null;
      return;
    }
    const protocol = props.protocols[0];
    if (protocol) model.value = { protocol, contextWindow: 0, maxOutputTokens: 0 };
  },
});
const contextWindow = computed({
  get: () => String(model.value?.contextWindow ?? ""),
  set: (value: string) => {
    if (model.value) model.value.contextWindow = Number(value);
  },
});
const outputLimit = computed({
  get: () => String(model.value?.maxOutputTokens ?? ""),
  set: (value: string) => {
    if (model.value) model.value.maxOutputTokens = Number(value);
  },
});
</script>

<template>
  <section v-if="protocols.length" class="mt-5 grid gap-3 border-t border-stroke-divider pt-4">
    <div class="flex items-center justify-between gap-3">
      <label :for="switchId" class="text-label text-fg-primary">{{
        t("settings.ai.assistantEnabled")
      }}</label>
      <UiSwitch
        :id="switchId"
        v-model="enabled"
        :disabled="disabled"
        :label="t('settings.ai.assistantEnabled')"
      />
    </div>
    <p class="text-caption text-fg-secondary">{{ t("settings.ai.assistantProfileHelp") }}</p>
    <div v-if="model" class="grid gap-3">
      <UiSelect
        v-model="model.protocol"
        :label="t('settings.ai.assistantProtocol')"
        :options="options"
        :disabled="disabled"
      />
      <label class="grid gap-1 text-caption text-fg-secondary">
        {{ t("settings.ai.contextWindow") }}
        <UiInput v-model="contextWindow" type="number" :disabled="disabled" />
      </label>
      <label class="grid gap-1 text-caption text-fg-secondary">
        {{ t("settings.ai.maxOutputTokens") }}
        <UiInput v-model="outputLimit" type="number" :disabled="disabled" />
      </label>
      <p
        v-if="
          showErrors &&
          (model.contextWindow < 8192 ||
            model.maxOutputTokens <= 0 ||
            model.maxOutputTokens >= model.contextWindow)
        "
        class="text-caption text-fg-danger"
        role="alert"
      >
        {{ t("settings.ai.assistantLimitsInvalid") }}
      </p>
    </div>
  </section>
</template>
