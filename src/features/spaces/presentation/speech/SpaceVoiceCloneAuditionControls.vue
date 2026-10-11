<script setup lang="ts">
import { Headphones, LoaderCircle, WandSparkles } from "@lucide/vue";
import { ToolbarButton } from "reka-ui";
import { computed, nextTick, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import { CANVAS_ACTION_BUTTON_CLASSES } from "../canvas/canvas-action-classes";
import { NODE_ACTION_LABEL_CLASS, nodeActionLabelState } from "../canvas/node-action-label";
import type { useCanvasVoiceClone } from "./use-canvas-voice-clone";
import VoiceCandidateReviews from "./VoiceCandidateReviews.vue";

const props = defineProps<{
  form: ReturnType<typeof useCanvasVoiceClone>["form"];
  compact: boolean;
}>();
const emit = defineEmits<{ save: [candidateId: string, name: string] }>();
const openPopover = defineModel<"audition" | "candidates" | undefined>("openPopover", { required: true });
const { t } = useI18n();
const generating = computed(() => Boolean(props.form.activeRunId || props.form.pendingRequestId));
const auditionTriggerRef = useTemplateRef<HTMLButtonElement>("auditionTriggerRef");
function setPopover(popover: "audition" | "candidates", open: boolean) {
  openPopover.value = open ? popover : openPopover.value === popover ? undefined : openPopover.value;
}
watch(
  () => props.form.candidates.length,
  async (count, previous) => {
    if (count > previous && openPopover.value === "audition") openPopover.value = "candidates";
    if (!count && openPopover.value === "candidates") {
      openPopover.value = undefined;
      await nextTick();
      auditionTriggerRef.value?.focus({ preventScroll: true });
    }
  },
);
</script>

<template>
  <UiPopover
    :open="openPopover === 'audition'"
    align="end"
    class="w-80 space-y-3 p-3"
    force-mount
    @update:open="setPopover('audition', $event)"
  >
    <template #trigger>
      <ToolbarButton as-child>
        <button
          type="button"
          :class="CANVAS_ACTION_BUTTON_CLASSES"
          :aria-label="t('spaces.speech.voice.candidate.generate')"
          :title="t('spaces.speech.voice.candidate.generate')"
          ref="auditionTriggerRef"
          data-testid="clone-audition-options"
        >
          <LoaderCircle
            v-if="generating"
            class="size-4.5 animate-spin motion-reduce:animate-none"
            :stroke-width="1.6"
          />
          <WandSparkles v-else class="size-4.5" :stroke-width="1.6" />
          <span :class="[NODE_ACTION_LABEL_CLASS, nodeActionLabelState(compact)]">{{
            t("spaces.speech.voice.candidate.generate")
          }}</span>
        </button>
      </ToolbarButton>
    </template>
    <label class="block space-y-1.5 text-caption font-medium text-fg-secondary">
      <span>{{ t("spaces.speech.voice.candidate.text") }}</span>
      <UiTextarea v-model="form.text" :rows="3" :disabled="form.busy" />
    </label>
    <template v-if="form.referenceTranscript === 'optionalConfirmed'">
      <UiTextarea
        v-model="form.transcript"
        :rows="2"
        :placeholder="t('spaces.speech.reference.transcript')"
        :disabled="form.busy"
        @update:model-value="form.transcriptConfirmed = false"
      />
      <label v-if="form.transcript.trim()" class="flex items-center gap-2 text-caption text-fg-secondary">
        <UiCheckbox
          v-model="form.transcriptConfirmed"
          :label="t('spaces.speech.reference.transcriptConfirmed')"
          :disabled="form.busy"
        />{{ t("spaces.speech.reference.transcriptConfirmed") }}
      </label>
    </template>
    <div class="flex justify-end gap-2 border-t border-stroke-divider pt-2">
      <UiButton
        :variant="generating ? 'secondary' : 'primary'"
        size="sm"
        :class="!generating && 'bg-surface-inverse text-fg-inverse hover:bg-surface-inverse hover:opacity-90'"
        :disabled="generating ? form.isCancelling : form.busy || !form.ready || form.candidates.length >= 16"
        :data-testid="generating ? 'clone-cancel-audition' : 'clone-generate-audition'"
        @click="generating ? form.cancel() : form.generate()"
        >{{ t(generating ? "common.cancel" : "spaces.speech.voice.candidate.generate") }}</UiButton
      >
    </div>
  </UiPopover>
  <UiPopover
    :open="openPopover === 'candidates'"
    align="end"
    class="w-80 p-3"
    force-mount
    @update:open="setPopover('candidates', $event)"
  >
    <template #trigger>
      <ToolbarButton as-child>
        <button
          type="button"
          :class="CANVAS_ACTION_BUTTON_CLASSES"
          :disabled="!form.candidates.length"
          :aria-label="t('spaces.speech.voice.candidate.auditions')"
          :title="t('spaces.speech.voice.candidate.auditions')"
          data-testid="clone-auditions"
        >
          <Headphones class="size-4.5" :stroke-width="1.6" /><span
            :class="[NODE_ACTION_LABEL_CLASS, nodeActionLabelState(compact)]"
            >{{ t("spaces.speech.voice.candidate.auditions") }}</span
          ><span class="tabular-nums">{{ form.candidates.length }}</span>
        </button>
      </ToolbarButton>
    </template>
    <UiScrollArea class="max-h-72" viewport-class="max-h-72">
      <VoiceCandidateReviews
        :candidates="form.candidates"
        :disabled="form.busy"
        @audition="form.audition"
        @save="(candidateId, name) => emit('save', candidateId, name)"
        @discard="form.discard"
      />
    </UiScrollArea>
  </UiPopover>
</template>
