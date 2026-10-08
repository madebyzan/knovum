<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { formatBytes } from "../../../../foundation/format/bytes";
import { useLocalModelsQuery } from "../../../../foundation/runtime/resource-query";
import UiButton from "../../../../shared/ui/UiButton.vue";

const props = defineProps<{ disabled: boolean; canAlign: boolean; selectedCount: number }>();
const emit = defineEmits<{ align: [] }>();
const { t } = useI18n();
const models = useLocalModelsQuery();
const selected = computed(() =>
  models.data.value?.find((model) => model.capability === "speechAlignment" && model.selected),
);
</script>

<template>
  <div class="space-y-1.5 border-b border-stroke-divider p-2">
    <div class="flex items-center gap-2">
      <UiButton
        size="sm"
        data-testid="transcript-align"
        :disabled="props.disabled || !canAlign || models.isPending.value || Boolean(models.error.value)"
        @click="emit('align')"
      >
        {{ t("spaces.transcript.text.alignSelected") }} · {{ selectedCount }}
      </UiButton>
      <span class="text-caption-sm text-fg-tertiary">{{ t("spaces.transcript.text.alignmentLimit") }}</span>
    </div>
    <p v-if="selected" data-testid="alignment-model" class="text-caption-sm text-fg-tertiary">
      {{ selected.name }} · {{ formatBytes(selected.totalBytes) }}
    </p>
    <p
      v-if="selected && !selected.verified"
      data-testid="alignment-first-use"
      class="text-caption-sm text-fg-tertiary"
    >
      {{ t("spaces.transcript.text.alignmentFirstUse") }}
    </p>
    <p v-if="models.error.value" role="alert" class="text-caption-sm text-fg-danger">
      {{ getErrorMessage(models.error.value) }}
    </p>
  </div>
</template>
