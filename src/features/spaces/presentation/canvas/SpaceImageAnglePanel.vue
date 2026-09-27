<script setup lang="ts">
import { RotateCcw } from "@lucide/vue";
import { clamp } from "es-toolkit";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { canRunAction } from "../../../../shared/contracts/action-availability";
import UiActionHelp from "../../../../shared/ui/UiActionHelp.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import type { ImageEdit } from "../../image-editing/image-edit";
import type { ImageOperation } from "../../knowledge-derivations/image-operation";
import type { ImageActionAvailability } from "./image-action-availability";
import { IMAGE_ANGLE_PITCH, IMAGE_ANGLE_YAW } from "./image-angle-range";
import SpaceImageAnglePreview from "./SpaceImageAnglePreview.vue";
import SpaceImageRangeSlider from "./SpaceImageRangeSlider.vue";

defineProps<{
  path: string;
  revision?: string | undefined;
  edit: ImageEdit;
  busy: boolean;
  availability: ImageActionAvailability;
  help?: { label: string; actionLabel?: string | undefined } | undefined;
}>();
const emit = defineEmits<{ cancel: []; apply: [operation: ImageOperation]; openSettings: [] }>();
const { t } = useI18n();
const mode = shallowRef<"subject" | "camera">("subject");
const yaw = shallowRef(0);
const pitch = shallowRef(0);
const zoomIndex = shallowRef(1);
const zoom = computed(() => (zoomIndex.value === 0 ? "close" : zoomIndex.value === 2 ? "wide" : "medium"));
const changed = computed(() => yaw.value !== 0 || pitch.value !== 0 || zoomIndex.value !== 1);

function reset() {
  yaw.value = 0;
  pitch.value = 0;
  zoomIndex.value = 1;
}

function apply() {
  emit("apply", {
    kind: "multi-angle",
    mode: mode.value,
    yaw: yaw.value,
    pitch: pitch.value,
    zoom: zoom.value,
  });
}

function changeZoom(delta: number) {
  zoomIndex.value = clamp(zoomIndex.value + delta, 0, 2);
}
</script>

<template>
  <section class="w-72 space-y-3 text-fg-primary" :aria-label="t('spaces.media.imageEditor.angle.title')">
    <header class="flex items-center justify-between px-1">
      <h3 class="text-title-sm font-semibold">{{ t("spaces.media.imageEditor.angle.title") }}</h3>
      <UiButton
        variant="ghost"
        size="sm"
        class="shadow-none"
        :title="t('common.reset')"
        :aria-label="t('common.reset')"
        :disabled="busy || !changed"
        @click="reset"
      >
        <RotateCcw class="size-3.5" />
      </UiButton>
    </header>
    <div
      class="grid grid-cols-2 rounded-panel bg-surface-control p-0.5"
      role="group"
      :aria-label="t('spaces.media.imageEditor.angle.mode')"
    >
      <button
        v-for="item in ['subject', 'camera'] as const"
        :key="item"
        type="button"
        class="h-(--knovum-control-height-md) rounded-md text-label text-fg-secondary transition-colors focus-visible:ring-2 focus-visible:ring-stroke-focus"
        :class="
          mode === item
            ? 'border border-stroke-default bg-surface-raised text-fg-primary shadow-control'
            : 'hover:text-fg-primary'
        "
        :aria-pressed="mode === item"
        :disabled="busy"
        @click="mode = item"
      >
        {{ t(`spaces.media.imageEditor.angle.modes.${item}`) }}
      </button>
    </div>
    <SpaceImageAnglePreview
      :path="path"
      :revision="revision"
      :edit="edit"
      :mode="mode"
      :yaw="yaw"
      :pitch="pitch"
      :zoom="zoom"
      @update="
        (nextYaw, nextPitch) => {
          yaw = nextYaw;
          pitch = nextPitch;
        }
      "
      @zoom="changeZoom"
    />
    <div class="space-y-2">
      <SpaceImageRangeSlider
        :label="t('spaces.media.imageEditor.angle.yaw')"
        :value="yaw"
        :min="IMAGE_ANGLE_YAW.min"
        :max="IMAGE_ANGLE_YAW.max"
        @update="yaw = $event"
      />
      <SpaceImageRangeSlider
        :label="t('spaces.media.imageEditor.angle.pitch')"
        :value="pitch"
        :min="IMAGE_ANGLE_PITCH.min"
        :max="IMAGE_ANGLE_PITCH.max"
        @update="pitch = $event"
      />
      <SpaceImageRangeSlider
        :label="t('spaces.media.imageEditor.angle.zoom')"
        :display-value="t(`spaces.media.imageEditor.angle.zoomLevels.${zoom}`)"
        :value="zoomIndex"
        :min="0"
        :max="2"
        @update="zoomIndex = $event"
      />
    </div>
    <footer class="flex gap-2 pt-2">
      <UiButton
        variant="secondary"
        size="md"
        class="min-w-0 flex-1 border border-stroke-default bg-surface-raised"
        :disabled="busy"
        @click="emit('cancel')"
        >{{ t("common.cancel") }}</UiButton
      >
      <UiButton
        variant="primary"
        size="md"
        class="min-w-0 flex-1 bg-surface-inverse text-fg-inverse hover:bg-surface-inverse hover:opacity-90"
        :disabled="busy || !changed || !canRunAction(availability)"
        @click="apply"
        >{{ t("spaces.media.imageEditor.applyNow") }}</UiButton
      >
      <UiActionHelp
        v-if="help"
        :label="help.label"
        :action-label="help.actionLabel"
        @action="emit('openSettings')"
      />
    </footer>
  </section>
</template>
