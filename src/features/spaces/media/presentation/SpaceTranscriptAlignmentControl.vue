<script setup lang="ts">
import { useI18n } from "vue-i18n";

import { canStartWith } from "../../../../foundation/ai/ai-readiness";
import AiReadinessNotice from "../../../../foundation/ai/AiReadinessNotice.vue";
import { useAiReadiness } from "../../../../foundation/ai/use-ai-readiness";
import UiButton from "../../../../shared/ui/UiButton.vue";

const props = defineProps<{ disabled: boolean; canAlign: boolean; selectedCount: number }>();
const emit = defineEmits<{ align: [] }>();
const { t } = useI18n();
const readiness = useAiReadiness([{ kind: "localCapability", capability: "speechAlignment" }]);
</script>

<template>
  <div class="space-y-1.5 border-b border-stroke-divider p-2">
    <div class="flex items-center gap-2">
      <UiButton
        size="sm"
        data-testid="transcript-align"
        :disabled="props.disabled || !canAlign || !canStartWith(readiness)"
        @click="emit('align')"
      >
        {{ t("spaces.transcript.text.alignSelected") }} · {{ selectedCount }}
      </UiButton>
      <span class="text-caption-sm text-fg-tertiary">{{ t("spaces.transcript.text.alignmentLimit") }}</span>
    </div>
    <AiReadinessNotice :readiness="readiness" />
  </div>
</template>
