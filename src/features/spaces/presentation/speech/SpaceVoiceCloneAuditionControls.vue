<script setup lang="ts">
import { Headphones, LoaderCircle } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import { CANVAS_ACTION_BUTTON_CLASSES } from "../canvas/canvas-action-classes";
import type { useCanvasVoiceClone } from "./use-canvas-voice-clone";
import VoiceCandidateReviews from "./VoiceCandidateReviews.vue";

defineProps<{
  form: ReturnType<typeof useCanvasVoiceClone>["form"];
  save: ReturnType<typeof useCanvasVoiceClone>["save"];
}>();
const { t } = useI18n();
</script>
<template>
  <ToolbarButton v-if="form.activeRunId || form.pendingRequestId" as-child>
    <button type="button" :class="CANVAS_ACTION_BUTTON_CLASSES" @click="form.cancel">
      <LoaderCircle class="size-4 animate-spin motion-reduce:animate-none" :stroke-width="1.6" />{{
        t("common.cancel")
      }}
    </button>
  </ToolbarButton>
  <UiPopover v-else align="end" class="w-80 space-y-3 p-3">
    <template #trigger
      ><ToolbarButton as-child
        ><button type="button" :class="CANVAS_ACTION_BUTTON_CLASSES">
          {{ t("spaces.speech.voice.candidate.generate") }}
        </button></ToolbarButton
      ></template
    >
    <label class="block space-y-1.5 text-caption font-medium text-fg-secondary"
      ><span>{{ t("spaces.speech.voice.candidate.text") }}</span
      ><UiTextarea v-model="form.text" :rows="3" :disabled="form.busy"
    /></label>
    <template v-if="form.referenceTranscript === 'optionalConfirmed'">
      <UiTextarea
        v-model="form.transcript"
        :rows="2"
        :placeholder="t('spaces.speech.reference.transcript')"
        :disabled="form.busy"
        @update:model-value="form.transcriptConfirmed = false"
      />
      <label v-if="form.transcript.trim()" class="flex items-center gap-2 text-caption text-fg-secondary"
        ><UiCheckbox
          v-model="form.transcriptConfirmed"
          :label="t('spaces.speech.reference.transcriptConfirmed')"
        />{{ t("spaces.speech.reference.transcriptConfirmed") }}</label
      >
    </template>
    <label class="flex items-center gap-2 text-caption text-fg-secondary"
      ><UiCheckbox
        v-model="form.authorized"
        :label="t('spaces.speech.reference.authorization')"
        :disabled="form.busy"
      />{{ t("spaces.speech.reference.authorization") }}</label
    >
    <div class="flex justify-end gap-2 border-t border-stroke-divider pt-2">
      <UiButton v-if="form.activeRunId || form.pendingRequestId" size="sm" @click="form.cancel">{{
        t("common.cancel")
      }}</UiButton>
      <UiButton
        variant="primary"
        size="sm"
        class="bg-surface-inverse text-fg-inverse hover:bg-surface-inverse hover:opacity-90"
        :disabled="form.busy || !form.ready || form.candidates.length >= 16"
        @click="form.generate"
        >{{ t("spaces.speech.voice.candidate.generate") }}</UiButton
      >
    </div>
  </UiPopover>
  <UiPopover v-if="form.candidates.length" align="end" class="w-80 p-3" force-mount>
    <template #trigger
      ><ToolbarButton as-child
        ><button type="button" :class="CANVAS_ACTION_BUTTON_CLASSES">
          <Headphones class="size-4" :stroke-width="1.6" /><span class="sr-only">{{
            t("spaces.speech.voice.candidate.auditions")
          }}</span
          >{{ form.candidates.length }}
        </button></ToolbarButton
      ></template
    >
    <UiScrollArea class="h-72"
      ><VoiceCandidateReviews
        :candidates="form.candidates"
        :disabled="form.busy"
        :authorized="form.authorized"
        @audition="form.audition"
        @save="save"
        @discard="form.discard"
    /></UiScrollArea>
  </UiPopover>
</template>
