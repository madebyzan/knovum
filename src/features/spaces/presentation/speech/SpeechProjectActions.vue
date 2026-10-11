<script setup lang="ts">
import { LoaderCircle, Square, WandSparkles } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import type { SpeechProjectOverview } from "../../contracts/generated/SpeechProjectOverview";

const props = defineProps<{
  canGenerate: boolean;
  projectType: SpeechProjectOverview["projectType"];
  isRunning: boolean;
  isSubmitting: boolean;
  isSaving: boolean;
  isCancelling: boolean;
  isPreviewListening: boolean;
}>();
const emit = defineEmits<{ generate: []; cancel: []; stopListening: [] }>();
const { t } = useI18n();
const generationLabelKey = computed(() => {
  switch (props.projectType) {
    case "short_dubbing":
      return "spaces.speech.project.actions.shortDubbing";
    case "audiobook":
      return "spaces.speech.project.actions.audiobook";
    case "video_dubbing":
      return "spaces.speech.project.actions.videoDubbing";
    default:
      throw new Error("Unsupported speech project type");
  }
});
</script>

<template>
  <footer class="flex shrink-0 flex-wrap items-center gap-2 border-t border-stroke-divider px-3 py-2">
    <UiButton v-if="isPreviewListening" size="sm" variant="ghost" @click="emit('stopListening')">
      <Square class="size-3.5" />{{ t("spaces.speech.preview.stop") }}
    </UiButton>
    <div class="ml-auto flex items-center gap-2">
      <UiButton
        v-if="isRunning"
        size="sm"
        :disabled="isCancelling"
        data-testid="speech-cancel"
        @click="emit('cancel')"
      >
        <LoaderCircle v-if="isCancelling" class="size-3.5 animate-spin motion-reduce:animate-none" />
        <Square v-else class="size-3.5" />{{ t("common.cancel") }}
      </UiButton>
      <UiButton
        v-else
        data-testid="speech-generate"
        size="sm"
        variant="primary"
        class="bg-surface-inverse text-fg-inverse hover:bg-surface-inverse hover:opacity-90"
        :disabled="!canGenerate || isSaving || isSubmitting"
        @click="emit('generate')"
      >
        <LoaderCircle
          v-if="isSaving || isSubmitting"
          class="size-3.5 animate-spin motion-reduce:animate-none"
        />
        <WandSparkles v-else class="size-3.5" />{{
          t(
            isSaving
              ? "spaces.speech.project.saving"
              : isSubmitting
                ? "spaces.speech.project.starting"
                : generationLabelKey,
          )
        }}
      </UiButton>
    </div>
  </footer>
</template>
