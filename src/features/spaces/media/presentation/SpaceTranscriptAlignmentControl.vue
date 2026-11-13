<script setup lang="ts">
import { useI18n } from "vue-i18n";

import { canStartWith } from "../../../../foundation/ai/ai-readiness";
import AiReadinessNotice from "../../../../foundation/ai/AiReadinessNotice.vue";
import ModelChoiceTrigger from "../../../../foundation/ai/ModelChoiceTrigger.vue";
import { useAiReadiness } from "../../../../foundation/ai/use-ai-readiness";
import { useLocalModelOptions } from "../../../../foundation/ai/use-model-choice-options";
import type { LocalModelId } from "../../../../shared/contracts/generated/LocalModelId";
import UiButton from "../../../../shared/ui/UiButton.vue";

const props = defineProps<{ disabled: boolean; canAlign: boolean; selectedCount: number }>();
const emit = defineEmits<{ align: [model?: LocalModelId] }>();
const { t } = useI18n();
const readiness = useAiReadiness([{ kind: "localCapability", capability: "speechAlignment" }]);
const { options: alignmentModels } = useLocalModelOptions(() => "speechAlignment");
</script>

<template>
  <div class="space-y-1.5 border-b border-stroke-divider p-2">
    <div class="flex items-center gap-2">
      <ModelChoiceTrigger
        :options="alignmentModels"
        :disabled="props.disabled || !canAlign || !canStartWith(readiness)"
        @select="(id) => emit('align', id as LocalModelId)"
      >
        <template #trigger="{ multiple }">
          <UiButton
            size="sm"
            data-testid="transcript-align"
            :disabled="props.disabled || !canAlign || !canStartWith(readiness)"
            @click="!multiple && emit('align')"
          >
            {{ t("spaces.transcript.text.alignSelected") }} · {{ selectedCount }}
          </UiButton>
        </template>
      </ModelChoiceTrigger>
      <span class="text-caption-sm text-fg-tertiary">{{ t("spaces.transcript.text.alignmentLimit") }}</span>
    </div>
    <AiReadinessNotice :readiness="readiness" />
  </div>
</template>
