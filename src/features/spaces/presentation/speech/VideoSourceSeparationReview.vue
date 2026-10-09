<script setup lang="ts">
import { AudioLines, Check, LoaderCircle, Square, X } from "@lucide/vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { formatBytes } from "../../../../foundation/format/bytes";
import { useLocalModelsQuery } from "../../../../foundation/runtime/use-resource-queries";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import type { SpeechMediaSource } from "../../contracts/generated/SpeechMediaSource";
import type { SpeechSourceSeparationCandidate } from "../../contracts/generated/SpeechSourceSeparationCandidate";
import { useSourceSeparationCandidate } from "../../speech/use-source-separation-candidate";
import SeparatedTrackActions from "./SeparatedTrackActions.vue";
import SpeechMediaPlayer from "./SpeechMediaPlayer.vue";

const props = defineProps<{
  spaceId: string;
  source: SpeechMediaSource;
  disabled: boolean;
}>();
const emit = defineEmits<{ accepted: [candidate: SpeechSourceSeparationCandidate] }>();
const { t } = useI18n();
const separation = useSourceSeparationCandidate();
const models = useLocalModelsQuery();
const selectedModel = computed(() =>
  models.data.value?.find((model) => model.capability === "speechSeparation" && model.selected),
);
const auditioned = shallowRef(false);
const stems = ["vocals", "background"] as const;
const players = useTemplateRef<InstanceType<typeof SpeechMediaPlayer>[]>("players");
function pauseOtherTracks(index: number) {
  players.value?.forEach((player, other) => {
    if (index !== other) player.pause();
  });
}
const sourceTooLong = computed(() => props.source.durationMs > 600_000);

watch(
  [() => props.spaceId, () => props.source.assetPath, () => props.source.sourceRevision],
  () => {
    auditioned.value = false;
    void separation.restore(props.spaceId, props.source);
  },
  { immediate: true },
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
  <section class="space-y-3">
    <p class="text-caption text-fg-secondary">{{ t("spaces.speech.separation.description") }}</p>
    <p v-if="selectedModel" class="text-caption text-fg-tertiary" data-testid="separation-model">
      {{ selectedModel.name }} · {{ formatBytes(selectedModel.totalBytes) }}
    </p>
    <p v-if="selectedModel && !selectedModel.verified" class="text-caption text-fg-tertiary">
      {{ t("spaces.speech.models.firstUse") }}
    </p>
    <div class="flex justify-end gap-2 border-t border-stroke-divider pt-3">
      <UiButton
        v-if="!separation.activeRunId.value"
        size="sm"
        variant="primary"
        :disabled="
          disabled ||
          sourceTooLong ||
          separation.busy.value ||
          separation.candidate.value?.reviewStatus === 'pending'
        "
        @click="separation.start(spaceId, source)"
        ><AudioLines class="size-3.5" />{{ t("spaces.speech.separation.generate") }}</UiButton
      >
      <UiButton
        v-else
        size="sm"
        variant="dangerSubtle"
        :disabled="separation.busy.value"
        @click="separation.cancel"
        ><Square class="size-3.5" />{{ t("spaces.speech.separation.cancel") }}</UiButton
      >
    </div>
    <p
      v-if="separation.activeRunId.value || separation.busy.value"
      role="status"
      class="flex items-center gap-2 text-caption text-fg-secondary"
    >
      <LoaderCircle class="size-3.5 animate-spin motion-reduce:animate-none" />{{
        t(separation.activeRunId.value ? "spaces.speech.separation.running" : "common.loading")
      }}
    </p>
    <p v-if="sourceTooLong" class="text-caption text-fg-danger">
      {{ t("spaces.speech.separation.tooLong") }}
    </p>
    <template v-if="separation.candidate.value">
      <p class="text-caption text-fg-danger">{{ t("spaces.speech.separation.reviewWarning") }}</p>
      <div v-for="(stem, index) in stems" :key="stem" class="space-y-2 border-t border-stroke-divider pt-3">
        <p class="text-caption text-fg-secondary">{{ t(`spaces.speech.separation.stems.${stem}`) }}</p>
        <SpeechMediaPlayer
          ref="players"
          kind="audio"
          :label="t(`spaces.speech.separation.stems.${stem}`)"
          :source="convertFileSrc(separation.candidate.value[stem].assetPath)"
          @play="pauseOtherTracks(index)"
        />
        <SeparatedTrackActions
          v-if="separation.candidate.value.reviewStatus === 'accepted'"
          :candidate="separation.candidate.value"
          :stem="stem"
          :disabled="disabled"
        />
      </div>
      <template v-if="separation.candidate.value.reviewStatus === 'pending'">
        <label class="flex items-center gap-2 text-caption text-fg-secondary">
          <UiCheckbox
            v-model="auditioned"
            :label="t('spaces.speech.separation.auditioned')"
            :disabled="disabled || separation.busy.value"
          />
          {{ t("spaces.speech.separation.auditioned") }}
        </label>
        <div class="flex gap-2">
          <UiButton
            size="sm"
            :disabled="disabled || separation.busy.value || !auditioned"
            @click="review(true)"
            ><Check class="size-3.5" />{{ t("spaces.speech.separation.accept") }}</UiButton
          >
          <UiButton
            size="sm"
            variant="dangerSubtle"
            :disabled="disabled || separation.busy.value"
            @click="review(false)"
            ><X class="size-3.5" />{{ t("spaces.speech.separation.reject") }}</UiButton
          >
        </div>
      </template>
      <p v-else class="text-caption text-fg-secondary">
        {{ t(`spaces.speech.separation.status.${separation.candidate.value.reviewStatus}`) }}
      </p>
    </template>
    <p v-if="separation.error.value" role="alert" class="text-caption text-fg-danger">
      {{ separation.error.value }}
    </p>
  </section>
</template>
