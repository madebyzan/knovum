<script setup lang="ts">
import { AudioLines, Check, Square, X } from "@lucide/vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import type { SpeechMediaSource } from "../../contracts/generated/SpeechMediaSource";
import type { SpeechSourceSeparationCandidate } from "../../contracts/generated/SpeechSourceSeparationCandidate";
import { useSourceSeparationCandidate } from "../../speech/use-source-separation-candidate";

const props = defineProps<{
  spaceId: string;
  source: SpeechMediaSource;
  disabled: boolean;
}>();
const emit = defineEmits<{ accepted: [candidate: SpeechSourceSeparationCandidate] }>();
const { t } = useI18n();
const separation = useSourceSeparationCandidate();
const auditioned = shallowRef(false);
const sourceTooLong = computed(() => props.source.durationMs > 600_000);

watch(
  () => props.source.sourceRevision,
  () => {
    auditioned.value = false;
    separation.reset();
  },
);
watch(
  () => separation.candidate.value?.separationId,
  () => {
    auditioned.value = false;
  },
);

async function review(accepted: boolean) {
  await separation.review(accepted);
  const candidate = separation.candidate.value;
  if (candidate?.reviewStatus === "accepted") emit("accepted", candidate);
}
</script>

<template>
  <section class="space-y-2 rounded-md border border-stroke-divider p-2">
    <p class="text-caption text-fg-secondary">{{ t("spaces.speech.sourceSeparationDescription") }}</p>
    <div class="flex gap-2">
      <UiButton
        v-if="!separation.activeRunId.value"
        size="sm"
        :disabled="
          disabled ||
          sourceTooLong ||
          separation.busy.value ||
          separation.candidate.value?.reviewStatus === 'pending'
        "
        @click="separation.start(spaceId, source)"
        ><AudioLines class="size-3.5" />{{ t("spaces.speech.separateBackground") }}</UiButton
      >
      <UiButton
        v-else
        size="sm"
        variant="dangerSubtle"
        :disabled="separation.busy.value"
        @click="separation.cancel"
        ><Square class="size-3.5" />{{ t("spaces.speech.cancelSeparation") }}</UiButton
      >
    </div>
    <p v-if="sourceTooLong" class="text-caption text-fg-danger">
      {{ t("spaces.speech.sourceSeparationTooLong") }}
    </p>
    <template v-if="separation.candidate.value">
      <p class="text-caption text-fg-danger">{{ t("spaces.speech.sourceSeparationReviewWarning") }}</p>
      <audio
        class="h-8 w-full"
        controls
        :src="convertFileSrc(separation.candidate.value.background.assetPath)"
      />
      <template v-if="separation.candidate.value.reviewStatus === 'pending'">
        <label class="flex items-center gap-2 text-caption text-fg-secondary">
          <UiCheckbox
            v-model="auditioned"
            :label="t('spaces.speech.separationAuditioned')"
            :disabled="disabled || separation.busy.value"
          />
          {{ t("spaces.speech.separationAuditioned") }}
        </label>
        <div class="flex gap-2">
          <UiButton
            size="sm"
            :disabled="disabled || separation.busy.value || !auditioned"
            @click="review(true)"
            ><Check class="size-3.5" />{{ t("spaces.speech.acceptSeparatedBackground") }}</UiButton
          >
          <UiButton
            size="sm"
            variant="dangerSubtle"
            :disabled="disabled || separation.busy.value"
            @click="review(false)"
            ><X class="size-3.5" />{{ t("spaces.speech.rejectSeparatedBackground") }}</UiButton
          >
        </div>
      </template>
      <p v-else class="text-caption text-fg-secondary">
        {{ t(`spaces.speech.separationStatus.${separation.candidate.value.reviewStatus}`) }}
      </p>
    </template>
    <p v-if="separation.error.value" class="text-caption text-fg-danger">{{ separation.error.value }}</p>
  </section>
</template>
