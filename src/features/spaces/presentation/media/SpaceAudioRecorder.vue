<script setup lang="ts">
import { AlertTriangle, CircleStop, LoaderCircle, Mic, Pause, Play, Radio } from "@lucide/vue";
import { error as logError } from "@tauri-apps/plugin-log";
import { computed, onBeforeUnmount, toRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiDialog from "../../../../shared/ui/UiDialog.vue";
import { createSpaceAudioRecorderController } from "../../media/audio-recorder-controller";
import type { PersistedAudioSpaceAsset } from "../../media/audio-recording";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{ spaceId?: string }>();
const emit = defineEmits<{
  saved: [asset: PersistedAudioSpaceAsset, spaceId: string];
  failed: [];
}>();
const { t } = useI18n();
const controller = createSpaceAudioRecorderController({
  open,
  spaceId: toRef(props, "spaceId"),
  saved: (asset, spaceId) => emit("saved", asset, spaceId),
  failed: () => emit("failed"),
  close: () => {
    open.value = false;
  },
});
const captureProblemText = computed(() =>
  controller.captureProblem.value ? t(`spaces.media.audioProblem.${controller.captureProblem.value}`) : "",
);
const statusText = computed(() => {
  if (controller.isSaving.value) return t("spaces.recording.savingRecording");
  if (controller.isRequestingPermission.value) return t("spaces.recording.startingRecording");
  if (controller.captureProblem.value) return captureProblemText.value;
  return controller.isPaused.value
    ? t("spaces.recording.pausedRecording")
    : t("spaces.recording.activeRecording");
});
let transition = Promise.resolve();
let disposed = false;
onBeforeUnmount(() => {
  disposed = true;
});
function reportLifecycleError(error: unknown) {
  void logError(`Audio recording lifecycle failed: ${String(error)}`);
  emit("failed");
}

watch(
  () => [open.value, props.spaceId] as const,
  ([isOpen, spaceId]) => {
    transition = transition
      .then(async () => {
        await controller.invalidate();
        if (!disposed && isOpen && open.value && props.spaceId === spaceId)
          void controller.start().catch(reportLifecycleError);
      })
      .catch(reportLifecycleError);
  },
  { immediate: true },
);
</script>

<template>
  <UiDialog
    :open="open"
    @update:open="!controller.isSaving.value && (open = $event)"
    :title="t('spaces.media.recordAudio')"
    class="bg-canvas"
  >
    <div class="flex items-center gap-2">
      <span class="grid size-8 place-items-center rounded-control bg-control">
        <LoaderCircle
          v-if="controller.isRequestingPermission.value || controller.isSaving.value"
          class="size-(--knovum-icon-size-sm) animate-spin text-foreground-secondary"
        />
        <Radio v-else-if="controller.isRecording.value" class="size-(--knovum-icon-size-sm) text-danger" />
        <AlertTriangle
          v-else-if="controller.captureProblem.value"
          class="size-(--knovum-icon-size-sm) text-warning"
        />
        <Mic v-else class="size-(--knovum-icon-size-sm) text-foreground-secondary" />
      </span>
      <div class="min-w-0 flex-1">
        <div class="font-mono text-title tabular-nums">{{ controller.elapsedLabel.value }}</div>
        <div role="status" class="text-caption text-foreground-secondary">{{ statusText }}</div>
      </div>
    </div>
    <p
      v-if="controller.captureProblem.value === 'denied'"
      class="mt-2 text-caption text-foreground-secondary"
    >
      {{ t("spaces.media.microphoneSettingsHint") }}
    </p>
    <template #footer>
      <UiButton variant="ghost" size="sm" :disabled="controller.isSaving.value" @click="open = false">{{
        t("common.cancel")
      }}</UiButton>
      <template v-if="controller.isRecording.value">
        <UiButton
          size="sm"
          variant="ghost"
          :disabled="controller.isSaving.value"
          @click="controller.togglePause"
        >
          <Play v-if="controller.isPaused.value" class="size-(--knovum-icon-size-sm)" />
          <Pause v-else class="size-(--knovum-icon-size-sm)" />
          {{ controller.isPaused.value ? t("common.resume") : t("common.pause") }}
        </UiButton>
        <UiButton size="sm" variant="primary" :disabled="controller.isSaving.value" @click="controller.stop">
          <CircleStop class="size-(--knovum-icon-size-sm)" />{{ t("spaces.recording.stopAndSave") }}
        </UiButton>
      </template>
      <UiButton
        v-else-if="controller.captureProblem.value"
        size="sm"
        variant="primary"
        :disabled="
          controller.isRequestingPermission.value ||
          controller.isSaving.value ||
          controller.captureProblem.value === 'unsupported' ||
          !spaceId
        "
        @click="controller.start().catch(reportLifecycleError)"
      >
        <Mic class="size-(--knovum-icon-size-sm)" />{{ t("common.retry") }}
      </UiButton>
    </template>
  </UiDialog>
</template>
