<script setup lang="ts">
import { CircleStop, Pause, Play, X } from "@lucide/vue";
import { error as logError } from "@tauri-apps/plugin-log";
import { computed, onBeforeUnmount, shallowRef, toRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { formatErrorDiagnostic } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { PersistedAudioSpaceAsset } from "../audio-recording";
import { useAudioRecorderController } from "../use-audio-recorder-controller";
import SpaceAudioRecorderMeter from "./SpaceAudioRecorderMeter.vue";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{
  spaceId?: string;
  save: (asset: PersistedAudioSpaceAsset, spaceId: string) => Promise<void>;
}>();
const emit = defineEmits<{ failed: [] }>();
const { t } = useI18n();
const confirmingDiscard = shallowRef(false);
const controller = useAudioRecorderController({
  open,
  spaceId: toRef(props, "spaceId"),
  save: props.save,
  failed: () => emit("failed"),
});
const captureProblemText = computed(() =>
  controller.captureProblem.value ? t(`spaces.recording.problems.${controller.captureProblem.value}`) : "",
);
const statusText = computed(() =>
  controller.isSaving.value
    ? t("spaces.recording.saving")
    : controller.isRequestingPermission.value
      ? t("spaces.recording.starting")
      : controller.captureProblem.value
        ? captureProblemText.value
        : controller.isPaused.value
          ? t("spaces.recording.paused")
          : t("spaces.recording.active"),
);
const deviceOptions = computed(() => [
  { value: "", label: t("spaces.recording.microphone.default") },
  ...controller.inputDevices.value.map((device, index) => ({
    value: device.deviceId,
    label: device.label || `${t("spaces.recording.microphone.select")} ${index + 1}`,
  })),
]);
let transition = Promise.resolve();
let disposed = false;
onBeforeUnmount(() => {
  disposed = true;
});
function reportLifecycleError(error: unknown) {
  void logError(`Audio recording lifecycle failed: ${formatErrorDiagnostic(error)}`);
  emit("failed");
}
function requestClose() {
  if (controller.isRecording.value && controller.elapsedMs.value >= 2_000 && !confirmingDiscard.value) {
    confirmingDiscard.value = true;
    return;
  }
  confirmingDiscard.value = false;
  open.value = false;
}
function onDeviceChange(deviceId: string) {
  confirmingDiscard.value = false;
  void controller.switchDevice(deviceId).catch(reportLifecycleError);
}
watch(
  () => [open.value, props.spaceId] as const,
  ([isOpen, spaceId]) => {
    confirmingDiscard.value = false;
    transition = transition
      .then(async () => {
        await controller.invalidate();
        if (!disposed && isOpen && open.value && props.spaceId === spaceId)
          return controller.start().catch(reportLifecycleError);
        return undefined;
      })
      .catch(reportLifecycleError);
  },
  { immediate: true },
);
</script>

<template>
  <Transition
    enter-active-class="transition-[opacity,transform] duration-150 ease-standard motion-reduce:transition-none"
    enter-from-class="translate-y-1 opacity-0"
    leave-active-class="transition-[opacity,transform] duration-100 ease-standard motion-reduce:transition-none"
    leave-to-class="translate-y-1 opacity-0"
  >
    <section
      v-if="open"
      class="w-76 overflow-hidden rounded-md border border-stroke-default bg-surface-overlay shadow-none"
      :aria-label="t('spaces.recording.note')"
    >
      <header class="flex h-8 items-center gap-1.5 border-b border-stroke-divider px-2.5">
        <span class="text-label font-medium text-fg-primary">{{ t("spaces.recording.note") }}</span>
        <UiSelect
          v-if="controller.inputDevices.value.length > 1 && !controller.isSaving.value"
          :model-value="controller.selectedDeviceId.value"
          size="sm"
          class="ml-auto h-6 max-w-36 border-transparent bg-transparent px-1 text-caption-sm text-fg-secondary shadow-none"
          :label="t('spaces.recording.microphone.select')"
          :options="deviceOptions"
          @update:model-value="onDeviceChange"
        />
        <UiIconButton
          :label="t('common.cancel')"
          size="sm"
          class="size-6"
          :class="controller.inputDevices.value.length <= 1 && 'ml-auto'"
          :disabled="controller.isSaving.value"
          @click="requestClose"
        >
          <X class="size-3" :stroke-width="1.7" />
        </UiIconButton>
      </header>
      <div class="flex h-14 items-center px-2.5">
        <SpaceAudioRecorderMeter
          :loading="controller.isRequestingPermission.value || controller.isSaving.value"
          :recording="controller.isRecording.value"
          :paused="controller.isPaused.value"
          :has-problem="Boolean(controller.captureProblem.value)"
          :elapsed-label="controller.elapsedLabel.value"
          :levels="controller.inputLevels.value"
          :status-text="statusText"
        />
        <div class="mx-2 h-6 w-px shrink-0 bg-stroke-divider" />
        <template v-if="controller.isRecording.value">
          <UiIconButton
            :label="controller.isPaused.value ? t('common.resume') : t('common.pause')"
            size="sm"
            class="size-7"
            :disabled="controller.isSaving.value"
            @click="controller.togglePause"
          >
            <Play v-if="controller.isPaused.value" class="size-3.5" />
            <Pause v-else class="size-3.5" />
          </UiIconButton>
          <UiButton
            size="sm"
            variant="secondary"
            class="ml-1.5 px-2"
            :disabled="controller.isSaving.value"
            @click="controller.stop"
          >
            <CircleStop class="size-3.5 text-fg-danger" />{{ t("spaces.recording.stopAndSave") }}
          </UiButton>
        </template>
        <UiButton
          v-else-if="controller.captureProblem.value"
          size="sm"
          variant="secondary"
          :disabled="
            controller.isRequestingPermission.value ||
            controller.isSaving.value ||
            controller.captureProblem.value === 'unsupported' ||
            !spaceId
          "
          @click="controller.start().catch(reportLifecycleError)"
        >
          {{ t("common.retry") }}
        </UiButton>
      </div>
      <div
        v-if="confirmingDiscard"
        class="flex items-center justify-between gap-2 border-t border-stroke-divider bg-surface-control/70 px-2.5 py-1.5"
      >
        <span class="text-caption-sm text-fg-secondary">{{ t("spaces.recording.discard.prompt") }}</span>
        <div class="flex items-center gap-1">
          <UiButton size="sm" variant="ghost" @click="confirmingDiscard = false">{{
            t("spaces.recording.discard.keep")
          }}</UiButton>
          <UiButton size="sm" variant="danger" @click="open = false">{{
            t("spaces.recording.discard.confirm")
          }}</UiButton>
        </div>
      </div>
      <p
        v-if="controller.captureProblem.value === 'denied'"
        class="border-t border-stroke-divider px-2.5 py-2 text-caption-sm leading-4 text-fg-secondary"
      >
        {{ t("spaces.recording.microphone.settingsHint") }}
      </p>
    </section>
  </Transition>
</template>
