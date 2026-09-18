<script setup lang="ts">
import { AlertTriangle, CircleStop, LoaderCircle, Mic, Pause, Play, Radio, X } from "@lucide/vue";
import { error as logError } from "@tauri-apps/plugin-log";
import { computed, onBeforeUnmount, toRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { createSpaceAudioRecorderController } from "../../media/audio-recorder-controller";
import type { PersistedAudioSpaceAsset } from "../../media/audio-recording";

const open = defineModel<boolean>("open", { required: true });
const props = defineProps<{
  spaceId?: string;
  save: (asset: PersistedAudioSpaceAsset, spaceId: string) => Promise<void>;
}>();
const emit = defineEmits<{ failed: [] }>();
const { t } = useI18n();
const controller = createSpaceAudioRecorderController({
  open,
  spaceId: toRef(props, "spaceId"),
  save: props.save,
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
  <Transition
    enter-active-class="transition-[opacity,transform] duration-150 ease-standard motion-reduce:transition-none"
    enter-from-class="translate-y-1 opacity-0"
    leave-active-class="transition-[opacity,transform] duration-100 ease-standard motion-reduce:transition-none"
    leave-to-class="translate-y-1 opacity-0"
  >
    <section
      v-if="open"
      class="w-72 overflow-hidden rounded-control border border-border bg-overlay shadow-toolbar"
      :aria-label="t('spaces.media.recordAudio')"
    >
      <header class="flex h-8 items-center border-b border-divider px-2.5">
        <span class="text-label font-medium text-foreground">{{ t("spaces.media.recordAudio") }}</span>
        <UiIconButton
          :label="t('common.cancel')"
          :tooltip="false"
          size="sm"
          class="ml-auto size-6"
          :disabled="controller.isSaving.value"
          @click="open = false"
        >
          <X class="size-(--knovum-icon-size-xs)" :stroke-width="1.7" />
        </UiIconButton>
      </header>

      <div class="flex h-14 items-center px-2.5">
        <span class="grid size-6 shrink-0 place-items-center">
          <LoaderCircle
            v-if="controller.isRequestingPermission.value || controller.isSaving.value"
            class="size-(--knovum-icon-size-sm) animate-spin text-foreground-secondary"
          />
          <Radio
            v-else-if="controller.isRecording.value"
            class="size-(--knovum-icon-size-sm)"
            :class="controller.isPaused.value ? 'text-foreground-tertiary' : 'text-danger'"
            :stroke-width="1.7"
          />
          <AlertTriangle
            v-else-if="controller.captureProblem.value"
            class="size-(--knovum-icon-size-sm) text-warning"
          />
          <Mic v-else class="size-(--knovum-icon-size-sm) text-foreground-secondary" />
        </span>

        <div class="ml-2 min-w-0 flex-1">
          <div class="text-body font-medium leading-5 tabular-nums">{{ controller.elapsedLabel.value }}</div>
          <div role="status" class="truncate text-micro leading-4 text-foreground-tertiary">
            {{ statusText }}
          </div>
        </div>

        <div class="mx-2 h-6 w-px shrink-0 bg-divider" />

        <template v-if="controller.isRecording.value">
          <UiIconButton
            :label="controller.isPaused.value ? t('common.resume') : t('common.pause')"
            size="sm"
            class="size-7"
            :disabled="controller.isSaving.value"
            @click="controller.togglePause"
          >
            <Play v-if="controller.isPaused.value" class="size-(--knovum-icon-size-sm)" />
            <Pause v-else class="size-(--knovum-icon-size-sm)" />
          </UiIconButton>
          <UiButton
            size="sm"
            variant="secondary"
            class="ml-1.5 px-2"
            :disabled="controller.isSaving.value"
            @click="controller.stop"
          >
            <CircleStop class="size-(--knovum-icon-size-sm) text-danger" />
            {{ t("spaces.recording.stopAndSave") }}
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

      <p
        v-if="controller.captureProblem.value === 'denied'"
        class="border-t border-divider px-2.5 py-2 text-micro leading-4 text-foreground-secondary"
      >
        {{ t("spaces.media.microphoneSettingsHint") }}
      </p>
    </section>
  </Transition>
</template>
