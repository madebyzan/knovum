<script setup lang="ts">
import { ArrowUp, Square, X } from "@lucide/vue";
import { useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import AiExecutionTargetField from "../../../../foundation/ai/AiExecutionTargetField.vue";
import type { EditorContextReference } from "../../../../shared/contracts/generated/EditorContextReference";
import type { TaskTarget } from "../../../../shared/contracts/generated/TaskTarget";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiKnovumAiIcon from "../../../../shared/ui/UiKnovumAiIcon.vue";
import type { SpaceAiReferenceCandidate } from "../tiptap-space-references";
import SpaceTiptapAiSourcePicker from "./SpaceTiptapAiSourcePicker.vue";

defineProps<{
  isStreaming: boolean;
  isReview: boolean;
  canSubmit: boolean;
  references: readonly SpaceAiReferenceCandidate[];
  attachedReferences: readonly EditorContextReference[];
}>();

const promptText = defineModel<string>("promptText", { required: true });
const writingTarget = defineModel<TaskTarget | undefined>("writingTarget", { required: true });

const emit = defineEmits<{
  syncCaret: [];
  keydown: [event: KeyboardEvent];
  toggleReference: [candidate: SpaceAiReferenceCandidate];
  cancel: [];
  submit: [];
  close: [];
}>();

const { t } = useI18n();
const inputEl = useTemplateRef<HTMLInputElement>("inputEl");

defineExpose({ inputEl });
</script>

<template>
  <div class="flex items-center gap-1.5 px-3 py-2">
    <UiKnovumAiIcon class="size-4 shrink-0 text-fg-secondary" />
    <input
      ref="inputEl"
      v-model="promptText"
      data-testid="tiptap-ai-input"
      type="text"
      :disabled="isStreaming"
      :placeholder="
        isReview
          ? t('spaces.richtext.ai.panel.followUpPlaceholder')
          : t('spaces.richtext.ai.panel.placeholder')
      "
      class="h-7 min-w-0 flex-1 bg-transparent text-body-lg text-fg-primary outline-none placeholder:text-fg-tertiary disabled:opacity-60"
      @input="emit('syncCaret')"
      @click="emit('syncCaret')"
      @keyup="emit('syncCaret')"
      @keydown="(event) => emit('keydown', event)"
    />
    <SpaceTiptapAiSourcePicker
      :candidates="references"
      :attached="attachedReferences"
      @toggle="(candidate) => emit('toggleReference', candidate)"
    />
    <AiExecutionTargetField v-model="writingTarget" purpose="writing" compact hide-when-single />
    <UiIconButton
      v-if="isStreaming"
      data-testid="tiptap-ai-stop"
      :label="t('spaces.richtext.ai.panel.stop')"
      tone="primary"
      size="sm"
      @click="emit('cancel')"
    >
      <Square class="size-2.5 fill-current" />
    </UiIconButton>
    <UiIconButton
      v-else
      data-testid="tiptap-ai-submit"
      :label="t('spaces.richtext.ai.panel.ask')"
      :tone="canSubmit ? 'primary' : 'neutral'"
      :class="!canSubmit && 'rounded-full bg-surface-control text-fg-tertiary'"
      size="sm"
      :disabled="!canSubmit"
      @click="emit('submit')"
    >
      <ArrowUp class="size-4" :stroke-width="2.1" />
    </UiIconButton>
    <UiIconButton data-testid="tiptap-ai-close" :label="t('common.close')" size="sm" @click="emit('close')">
      <X class="size-3.5" :stroke-width="1.6" />
    </UiIconButton>
  </div>
</template>
