<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import { useSpeechSignalInspection } from "../use-speech-signal-inspection";

const props = defineProps<{ projectId: string; revisionId: string }>();
const { t } = useI18n();
const { cancel, error, inspect, jobId, report, scannedMs } = useSpeechSignalInspection({
  projectId: computed(() => props.projectId),
  revisionId: computed(() => props.revisionId),
});
function formatTimestamp(ms: number) {
  return `${Math.floor(ms / 60_000)}:${String(Math.floor(ms / 1000) % 60).padStart(2, "0")}.${String(ms % 1000).padStart(3, "0")}`;
}
</script>

<template>
  <section class="space-y-2">
    <div class="flex items-center gap-2">
      <UiButton size="sm" :disabled="Boolean(jobId)" @click="inspect">{{
        t("spaces.speech.signal.inspect")
      }}</UiButton>
      <UiButton v-if="jobId" size="sm" @click="cancel">{{ t("common.cancel") }}</UiButton>
      <span v-if="jobId" class="text-caption text-fg-tertiary">{{ formatTimestamp(scannedMs) }}</span>
    </div>
    <p class="text-caption text-fg-tertiary">{{ t("spaces.speech.signal.advisory") }}</p>
    <p v-if="error" class="text-caption text-fg-danger">{{ error }}</p>
    <template v-if="report">
      <p v-if="!report.warnings.length" class="text-caption text-fg-secondary">
        {{ t("spaces.speech.signal.noFindings") }}
      </p>
      <p v-if="report.truncated" class="text-caption text-fg-tertiary">
        {{ t("spaces.speech.signal.truncated") }}
      </p>
      <ul class="space-y-1">
        <li v-for="(warning, index) in report.warnings" :key="index" class="text-caption text-fg-secondary">
          {{ formatTimestamp(warning.startMs) }}–{{ formatTimestamp(warning.endMs) }} ·
          {{ t(`spaces.speech.signal.issues.${warning.kind}`) }}
        </li>
      </ul>
    </template>
  </section>
</template>
