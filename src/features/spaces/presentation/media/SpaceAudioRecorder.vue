<script setup lang="ts">
import { AlertTriangle, CircleStop, LoaderCircle, Mic, Pause, Play, Radio, X } from "@lucide/vue";
import { error as logError } from "@tauri-apps/plugin-log";
import { computed, onBeforeUnmount, toRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { formatErrorDiagnostic } from "../../../../foundation/errors/error-format";
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
  controller.captureProblem.value ? t(`spaces.recording.problems.${controller.captureProblem.value}`) : "",
);
const statusText = computed(() => {
  if (controller.isSaving.value) return t("spaces.recording.saving");
  if (controller.isRequestingPermission.value) return t("spaces.recording.starting");
  if (controller.captureProblem.value) return captureProblemText.value;
  return controller.isPaused.value ? t("spaces.recording.paused") : t("spaces.recording.active");
});
let transition = Promise.resolve();
let disposed = false;
onBeforeUnmount(() => {
  disposed = true;
});
function reportLifecycleError(error: unknown) {
  void logError(`Audio recording lifecycle failed: ${formatErrorDiagnostic(error)}`);
  emit("failed");
}

watch(
  () => [open.value, props.spaceId] as const,
  ([isOpen, spaceId]) => {
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
      class="w-72 overflow-hidden rounded-md border border-stroke-default bg-surface-overlay shadow-none"
      :aria-label="t('spaces.recording.note')"
    >
      <header class="flex h-8 items-center border-b border-stroke-divider px-2.5">
        <span class="text-label font-medium text-fg-primary">{{ t("spaces.recording.note") }}</span>
        <UiIconButton
          :label="t('common.cancel')"
          size="sm"
          class="ml-auto size-6"
          :disabled="controller.isSaving.value"
          @click="open = false"
        >
          <X class="size-3" :stroke-width="1.7" />
        </UiIconButton>
      </header>

      <div class="flex h-14 items-center px-2.5">
        <span class="grid size-6 shrink-0 place-items-center">
          <LoaderCircle
            v-if="controller.isRequestingPermission.value || controller.isSaving.value"
            class="size-3.5 animate-spin text-fg-secondary"
          />
          <Radio
            v-else-if="controller.isRecording.value"
            class="size-3.5"
            :class="controller.isPaused.value ? 'text-fg-tertiary' : 'text-fg-danger'"
            :stroke-width="1.7"
          />
          <AlertTriangle v-else-if="controller.captureProblem.value" class="size-3.5 text-fg-warning" />
          <Mic v-else class="size-3.5 text-fg-secondary" />
        </span>

        <div class="ml-2 min-w-0 flex-1">
          <div class="text-body leading-5 font-medium tabular-nums">
            {{ controller.elapsedLabel.value }}
          </div>
          <div role="status" class="truncate text-caption-sm leading-4 text-fg-tertiary">
            {{ statusText }}
          </div>
        </div>

        <div class="bg-border-divider mx-2 h-6 w-px shrink-0" />

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
            <CircleStop class="size-3.5 text-fg-danger" />
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
        class="border-t border-stroke-divider px-2.5 py-2 text-caption-sm leading-4 text-fg-secondary"
      >
        {{ t("spaces.recording.microphone.settingsHint") }}
      </p>
    </section>
  </Transition>
</template>
