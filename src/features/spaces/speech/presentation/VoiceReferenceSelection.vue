<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiInput from "../../../../shared/ui/UiInput.vue";
import VoiceReferenceRange from "./VoiceReferenceRange.vue";

const start = defineModel<string>("start", { required: true });
const end = defineModel<string>("end", { required: true });
defineProps<{ durationMs: number | undefined; disabled: boolean; valid: boolean }>();
const { t } = useI18n();
const durationSeconds = computed(() => Math.max(0, Number(end.value) - Number(start.value)));
function setRange(from: number, to: number) {
  start.value = String(from);
  end.value = String(to);
}
</script>

<template>
  <div class="space-y-3">
    <VoiceReferenceRange
      :start="Number(start)"
      :end="Number(end)"
      :duration-ms="durationMs ?? 0"
      :disabled="disabled || durationMs === undefined"
      @change="setRange"
    />
    <div class="grid grid-cols-2 gap-3">
      <label class="block min-w-0 space-y-1.5 text-caption text-fg-secondary">
        <span>{{ t("spaces.speech.reference.start") }}</span>
        <UiInput
          v-model="start"
          size="sm"
          inputmode="decimal"
          :disabled="disabled || durationMs === undefined"
          data-testid="reference-start"
        />
      </label>
      <label class="block min-w-0 space-y-1.5 text-caption text-fg-secondary">
        <span>{{ t("spaces.speech.reference.end") }}</span>
        <UiInput
          v-model="end"
          size="sm"
          inputmode="decimal"
          :disabled="disabled || durationMs === undefined"
          data-testid="reference-end"
        />
      </label>
    </div>
    <p
      class="text-caption"
      :class="durationMs !== undefined && !valid ? 'text-fg-danger' : 'text-fg-secondary'"
      role="status"
    >
      {{
        t(
          durationMs !== undefined && durationMs < 5000
            ? "spaces.speech.reference.tooShort"
            : "spaces.speech.reference.requirements",
        )
      }}
      <span v-if="valid" class="mt-1 block text-fg-tertiary">{{
        t("spaces.speech.reference.selection", { seconds: durationSeconds.toFixed(1) })
      }}</span>
    </p>
  </div>
</template>
