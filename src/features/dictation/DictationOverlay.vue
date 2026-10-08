<script setup lang="ts">
import { AudioLines, Mic, Square } from "@lucide/vue";
import { computed, onMounted, onScopeDispose } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../foundation/errors/error-format";
import { resourceProgressMessage } from "../../foundation/runtime/resource-progress";
import UiButton from "../../shared/ui/UiButton.vue";
import UiDialog from "../../shared/ui/UiDialog.vue";
import { useDictationSession } from "./use-dictation-session";

const { t } = useI18n();
const session = useDictationSession();
const errorMessages: Record<string, string> = {
  microphone_permission_denied: "dictation.errors.microphonePermissionDenied",
  microphone_unavailable: "dictation.errors.microphoneUnavailable",
  recording_limit_reached: "dictation.errors.recordingLimitReached",
  audio_capture_empty: "dictation.errors.audioCaptureEmpty",
  unsupported_audio_format: "dictation.errors.unsupportedAudioFormat",
  unsupported_device: "dictation.errors.unsupportedDevice",
  dictation_focus_changed: "dictation.errors.dictationFocusChanged",
  dictation_permission_denied: "dictation.errors.dictationPermissionDenied",
  dictation_empty_result: "dictation.errors.dictationEmptyResult",
  dictation_insertion_failed: "dictation.errors.dictationInsertionFailed",
  dictation_failed: "dictation.errors.dictationFailed",
};
const busy = computed(() =>
  ["recording", "preparing_resources", "transcribing"].includes(session.state.value.phase),
);
const isTranscriptDialogOpen = computed({
  get: () =>
    session.state.value.phase === "failed" ||
    (session.state.value.phase === "result" && session.state.value.insertion !== "inserted"),
  set: (open) => {
    if (!open) void dismiss();
  },
});
const status = computed(() => {
  const phase = session.state.value.phase;
  return t(
    phase === "recording"
      ? "dictation.recording"
      : phase === "preparing_resources"
        ? "dictation.preparingResources"
        : "dictation.transcribing",
  );
});
const resourceProgress = computed(() =>
  session.state.value.resources ? resourceProgressMessage(session.state.value.resources) : "",
);
const transcriptDescription = computed(() => {
  const value = session.state.value;
  const key = value.errorCode ? errorMessages[value.errorCode] : undefined;
  return t(key ?? "dictation.result.description");
});

onMounted(() => {
  void session.start().catch(reportFailure);
});
onScopeDispose(session.stop);

async function copy() {
  try {
    await session.copy();
    toast.success(t("dictation.result.copied"));
  } catch (error) {
    reportFailure(error);
  }
}

async function dismiss() {
  try {
    await session.dismiss();
  } catch (error) {
    reportFailure(error);
  }
}

async function cancel() {
  try {
    await session.cancel();
  } catch (error) {
    reportFailure(error);
  }
}

function reportFailure(error: unknown) {
  toast.error(getErrorMessage(error));
}
</script>

<template>
  <div
    v-if="busy"
    class="fixed bottom-5 left-1/2 z-30 flex -translate-x-1/2 items-center gap-3 rounded-xl border border-stroke-default bg-surface-overlay px-3 py-2 shadow-dialog"
    role="status"
    aria-live="polite"
  >
    <Mic v-if="session.state.value.phase === 'recording'" class="size-4 text-fg-danger" />
    <AudioLines v-else class="size-4 animate-pulse text-fg-accent" />
    <div class="min-w-0">
      <p class="text-label font-medium text-fg-primary">{{ status }}</p>
      <p
        v-if="resourceProgress"
        data-testid="dictation-resource-progress"
        class="text-caption text-fg-secondary"
      >
        {{ resourceProgress }}
      </p>
      <p v-if="session.state.value.targetApplication" class="truncate text-caption text-fg-secondary">
        {{ session.state.value.targetApplication }}
      </p>
    </div>
    <UiButton size="sm" variant="ghost" @click="cancel">
      <Square class="size-3" />
      {{ t("common.cancel") }}
    </UiButton>
  </div>

  <UiDialog
    v-model:open="isTranscriptDialogOpen"
    :title="t('dictation.result.title')"
    :description="transcriptDescription"
  >
    <textarea
      v-if="session.state.value.text"
      readonly
      :aria-label="t('dictation.result.title')"
      :value="session.state.value.text"
      class="min-h-32 w-full resize-y rounded-lg border border-stroke-default bg-surface-canvas p-3 text-body text-fg-primary outline-none focus-visible:ring-2 focus-visible:ring-stroke-focus"
    />
    <p v-else class="text-body text-fg-secondary">{{ t("dictation.result.noText") }}</p>
    <template #footer>
      <UiButton variant="ghost" @click="dismiss">{{ t("common.close") }}</UiButton>
      <UiButton v-if="session.state.value.text" variant="primary" @click="copy">
        {{ t("dictation.result.copy") }}
      </UiButton>
    </template>
  </UiDialog>
</template>
