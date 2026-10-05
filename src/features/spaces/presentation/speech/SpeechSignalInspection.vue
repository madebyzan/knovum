<script setup lang="ts">
import { onScopeDispose, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import type { SpeechSignalReport } from "../../contracts/generated/SpeechSignalReport";
import { cancelSpeechGeneration } from "../../speech/ipc";
import { inspectSpeechSignal } from "../../speech/signal-inspection-ipc";

const props = defineProps<{ projectId: string; revisionId: string }>();
const emit = defineEmits<{ seek: [positionMs: number] }>();
const { t } = useI18n();
const report = shallowRef<SpeechSignalReport>();
const error = shallowRef<string>();
const jobId = shallowRef<string>();
const scannedMs = shallowRef(0);
let epoch = 0;
async function cancel() {
  if (jobId.value) await cancelSpeechGeneration(jobId.value);
}
function reset() {
  epoch += 1;
  report.value = undefined;
  void cancel().catch((cause) => {
    error.value = getErrorMessage(cause);
  });
}
watch(() => props.revisionId, reset);
onScopeDispose(reset);
async function inspect() {
  if (jobId.value) return;
  const version = ++epoch;
  const job = crypto.randomUUID();
  jobId.value = job;
  error.value = undefined;
  report.value = undefined;
  scannedMs.value = 0;
  try {
    const inspection = await inspectSpeechSignal(
      { projectId: props.projectId, revisionId: props.revisionId, jobId: job },
      (value) => {
        if (version === epoch) scannedMs.value = value;
      },
    );
    if (version === epoch) report.value = inspection;
  } catch (cause) {
    if (version === epoch) error.value = getErrorMessage(cause);
  } finally {
    if (jobId.value === job) jobId.value = undefined;
  }
}
const timestamp = (ms: number) =>
  `${Math.floor(ms / 60_000)}:${String(Math.floor(ms / 1000) % 60).padStart(2, "0")}.${String(ms % 1000).padStart(3, "0")}`;
</script>

<template>
  <section class="space-y-2">
    <div class="flex items-center gap-2">
      <UiButton size="sm" :disabled="Boolean(jobId)" @click="inspect">{{
        t("spaces.speech.inspectSignal")
      }}</UiButton>
      <UiButton v-if="jobId" size="sm" @click="cancel">{{ t("spaces.speech.cancel") }}</UiButton>
      <span v-if="jobId" class="text-caption text-fg-tertiary">{{ timestamp(scannedMs) }}</span>
    </div>
    <p class="text-caption text-fg-tertiary">{{ t("spaces.speech.signalAdvisory") }}</p>
    <p v-if="error" class="text-caption text-fg-danger">{{ error }}</p>
    <template v-if="report">
      <p v-if="!report.warnings.length" class="text-caption text-fg-secondary">
        {{ t("spaces.speech.signalNoFindings") }}
      </p>
      <p v-if="report.truncated" class="text-caption text-fg-tertiary">
        {{ t("spaces.speech.signalTruncated") }}
      </p>
      <ul class="space-y-1">
        <li v-for="(warning, index) in report.warnings" :key="index">
          <UiButton size="sm" variant="ghost" @click="emit('seek', warning.startMs)">
            {{ timestamp(warning.startMs) }}–{{ timestamp(warning.endMs) }} ·
            {{ t(`spaces.speech.signal.${warning.kind}`) }}
          </UiButton>
        </li>
      </ul>
    </template>
  </section>
</template>
