<script setup lang="ts">
import { AudioLines, Check, Download, Plus, Square, X } from "@lucide/vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { formatBytes } from "../../../../foundation/format/bytes";
import { useLocalModelsQuery } from "../../../../foundation/runtime/resource-query";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import { exportSpaceAssetFile } from "../../assets/export";
import type { SpeechMediaSource } from "../../contracts/generated/SpeechMediaSource";
import type { SpeechSourceSeparationCandidate } from "../../contracts/generated/SpeechSourceSeparationCandidate";
import { useSeparatedTrackInsertion } from "../../speech/use-separated-track-insertion";
import { useSourceSeparationCandidate } from "../../speech/use-source-separation-candidate";

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
const insertion = useSeparatedTrackInsertion();
const auditioned = shallowRef(false);
const exporting = shallowRef(false);
const exportError = shallowRef<string>();
const stems = ["vocals", "background"] as const;
function pauseOtherTracks(event: Event) {
  const audio = event.currentTarget as HTMLAudioElement;
  for (const other of audio.closest("section")!.querySelectorAll("audio")) if (other !== audio) other.pause();
}
const sourceTooLong = computed(() => props.source.durationMs > 600_000);

watch(
  [() => props.spaceId, () => props.source.assetPath, () => props.source.sourceRevision],
  () => {
    auditioned.value = false;
    exportError.value = undefined;
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

async function exportStem(kind: "background" | "vocals") {
  const candidate = separation.candidate.value;
  if (exporting.value || candidate?.reviewStatus !== "accepted") return;
  exporting.value = true;
  exportError.value = undefined;
  try {
    await exportSpaceAssetFile(
      candidate[kind].assetPath,
      kind === "background" ? "accompaniment.wav" : "vocals.wav",
    );
  } catch (cause) {
    if (separation.candidate.value?.separationId === candidate.separationId)
      exportError.value = getErrorMessage(cause);
  } finally {
    exporting.value = false;
  }
}
</script>

<template>
  <section class="space-y-2 rounded-md border border-stroke-divider p-2">
    <p class="text-caption text-fg-secondary">{{ t("spaces.speech.separation.description") }}</p>
    <p v-if="selectedModel" class="text-caption text-fg-tertiary" data-testid="separation-model">
      {{ selectedModel.name }} · {{ formatBytes(selectedModel.totalBytes) }}
    </p>
    <p v-if="selectedModel && !selectedModel.verified" class="text-caption text-fg-tertiary">
      {{ t("spaces.speech.models.firstUse") }}
    </p>
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
    <p v-if="sourceTooLong" class="text-caption text-fg-danger">
      {{ t("spaces.speech.separation.tooLong") }}
    </p>
    <template v-if="separation.candidate.value">
      <p class="text-caption text-fg-danger">{{ t("spaces.speech.separation.reviewWarning") }}</p>
      <div v-for="stem in stems" :key="stem" class="space-y-1">
        <p class="text-caption text-fg-secondary">{{ t(`spaces.speech.separation.stems.${stem}`) }}</p>
        <audio
          class="h-8 w-full"
          controls
          preload="metadata"
          :aria-label="t(`spaces.speech.separation.stems.${stem}`)"
          :src="convertFileSrc(separation.candidate.value[stem].assetPath)"
          @play="pauseOtherTracks"
        />
        <UiButton
          v-if="separation.candidate.value.reviewStatus === 'accepted'"
          size="sm"
          :disabled="disabled || insertion.busy.value"
          :data-testid="`insert-separated-${stem}`"
          @click="
            insertion.insert(separation.candidate.value, stem, t(`spaces.speech.separation.stems.${stem}`))
          "
          ><Plus class="size-3.5" />{{ t("spaces.speech.separation.addToSpace") }}</UiButton
        >
        <UiButton
          v-if="separation.candidate.value.reviewStatus === 'accepted'"
          size="sm"
          :disabled="disabled || exporting"
          :data-testid="`export-separated-${stem}`"
          @click="exportStem(stem)"
          ><Download class="size-3.5" />{{ t(`spaces.speech.separation.export.${stem}`) }}</UiButton
        >
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
    <p v-if="insertion.error.value" role="alert" class="text-caption text-fg-danger">
      {{ insertion.error.value }}
    </p>
    <p v-if="exportError" role="alert" class="text-caption text-fg-danger">{{ exportError }}</p>
  </section>
</template>
