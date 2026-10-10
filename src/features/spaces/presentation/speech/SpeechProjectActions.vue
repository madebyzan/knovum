<script setup lang="ts">
import { Save, Square, WandSparkles } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";

defineProps<{
  canGenerate: boolean;
  canSave: boolean;
  isRunning: boolean;
  isSubmitting: boolean;
  isSaving: boolean;
  isCancelling: boolean;
  isPreviewListening: boolean;
}>();
const emit = defineEmits<{ generate: []; save: []; cancel: []; stopListening: [] }>();
const { t } = useI18n();
</script>

<template>
  <div class="flex shrink-0 flex-wrap items-center gap-2 border-t border-stroke-divider px-3 py-2">
    <UiButton
      data-testid="speech-generate"
      size="sm"
      variant="primary"
      class="bg-surface-inverse text-fg-inverse hover:bg-surface-inverse hover:opacity-90"
      :disabled="!canGenerate"
      @click="emit('generate')"
    >
      <WandSparkles class="size-3.5" />{{ t("spaces.speech.project.generate") }}
    </UiButton>
    <UiButton size="sm" data-testid="speech-save" :disabled="!canSave" @click="emit('save')">
      <Save class="size-3.5" />{{ t("spaces.speech.project.saveRevision") }}
    </UiButton>
    <UiButton
      v-if="isRunning"
      size="sm"
      variant="dangerSubtle"
      :disabled="isCancelling"
      @click="emit('cancel')"
    >
      <Square class="size-3.5" />{{ t("common.cancel") }}
    </UiButton>
    <UiButton v-if="isPreviewListening" size="sm" variant="ghost" @click="emit('stopListening')">
      <Square class="size-3.5" />{{ t("spaces.speech.project.stopListening") }}
    </UiButton>
    <span v-if="isSubmitting || isSaving" class="text-caption-sm text-fg-tertiary">{{
      t(isSubmitting ? "spaces.speech.project.starting" : "spaces.speech.project.saving")
    }}</span>
  </div>
</template>
