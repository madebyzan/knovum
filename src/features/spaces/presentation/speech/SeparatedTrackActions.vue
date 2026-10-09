<script setup lang="ts">
import { Download, Plus } from "@lucide/vue";
import { shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import { exportSpaceAssetFile } from "../../assets/export";
import type { SpeechSourceSeparationCandidate } from "../../contracts/generated/SpeechSourceSeparationCandidate";
import { useSeparatedTrackInsertion } from "../../speech/use-separated-track-insertion";

const props = defineProps<{
  candidate: SpeechSourceSeparationCandidate;
  stem: "vocals" | "background";
  disabled: boolean;
}>();
const { t } = useI18n();
const insertion = useSeparatedTrackInsertion();
const exporting = shallowRef(false);
const error = shallowRef<string>();
async function exportTrack() {
  if (exporting.value) return;
  const candidate = props.candidate;
  exporting.value = true;
  error.value = undefined;
  try {
    await exportSpaceAssetFile(
      candidate[props.stem].assetPath,
      props.stem === "background" ? "accompaniment.wav" : "vocals.wav",
    );
  } catch (cause) {
    if (props.candidate.separationId === candidate.separationId) error.value = getErrorMessage(cause);
  } finally {
    exporting.value = false;
  }
}
</script>

<template>
  <div class="space-y-2">
    <div class="flex flex-wrap gap-2">
      <UiButton
        size="sm"
        :disabled="disabled || insertion.busy.value"
        :data-testid="`insert-separated-${stem}`"
        @click="insertion.insert(candidate, stem, t(`spaces.speech.separation.stems.${stem}`))"
      >
        <Plus class="size-3.5" />{{ t("spaces.speech.separation.addToSpace") }}
      </UiButton>
      <UiButton
        size="sm"
        :disabled="disabled || exporting"
        :data-testid="`export-separated-${stem}`"
        @click="exportTrack"
      >
        <Download class="size-3.5" />{{ t(`spaces.speech.separation.export.${stem}`) }}
      </UiButton>
    </div>
    <p v-if="error || insertion.error.value" role="alert" class="text-caption text-fg-danger">
      {{ error || insertion.error.value }}
    </p>
  </div>
</template>
