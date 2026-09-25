<script setup lang="ts">
import { Link2, Unlink2 } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import {
  centeredCrop,
  resizeCrop,
  scaleCropToPixelSize,
  type ImageCrop,
} from "../../image-editing/image-edit";
import SpaceImageCropPresets from "./SpaceImageCropPresets.vue";

const props = defineProps<{
  crop: ImageCrop;
  sourceWidth: number;
  sourceHeight: number;
  lockedAspect?: number | undefined;
}>();
const emit = defineEmits<{
  update: [crop: ImageCrop];
  updateLockedAspect: [ratio?: number];
  cancel: [];
  apply: [];
}>();
const { t } = useI18n();
const widthPixels = computed(() => Math.max(1, Math.round(props.crop.width * props.sourceWidth)));
const heightPixels = computed(() => Math.max(1, Math.round(props.crop.height * props.sourceHeight)));

function setPixels(axis: "width" | "height", event: Event) {
  const input = event.target as HTMLInputElement;
  const value = Number(input.value);
  const sourceSize = axis === "width" ? props.sourceWidth : props.sourceHeight;
  if (!Number.isInteger(value) || value < Math.ceil(sourceSize * 0.01) || value > sourceSize) {
    input.value = String(axis === "width" ? widthPixels.value : heightPixels.value);
    return;
  }
  if (props.lockedAspect) {
    emit(
      "update",
      scaleCropToPixelSize({
        crop: props.crop,
        sourceWidth: props.sourceWidth,
        sourceHeight: props.sourceHeight,
        axis,
        pixels: value,
      }),
    );
    return;
  }
  emit(
    "update",
    resizeCrop(
      props.crop,
      axis === "width" ? value / props.sourceWidth : props.crop.width,
      axis === "height" ? value / props.sourceHeight : props.crop.height,
    ),
  );
}

function setAspect(ratio?: number) {
  emit(
    "update",
    ratio
      ? centeredCrop(props.sourceWidth / props.sourceHeight, ratio)
      : {
          x: 0,
          y: 0,
          width: 1,
          height: 1,
        },
  );
  emit("updateLockedAspect", ratio);
}

function toggleAspect() {
  emit(
    "updateLockedAspect",
    props.lockedAspect
      ? undefined
      : (props.crop.width * props.sourceWidth) / (props.crop.height * props.sourceHeight),
  );
}
</script>

<template>
  <section class="w-60 text-foreground" :aria-label="t('spaces.media.crop')">
    <h3 class="px-1 pb-3 text-body-lg font-semibold">{{ t("spaces.media.crop") }}</h3>
    <div class="flex items-center gap-2 pb-3">
      <label
        class="flex h-(--knovum-control-height-lg) min-w-0 flex-1 items-center gap-1 rounded-control bg-control px-2 text-label text-foreground-secondary focus-within:ring-2 focus-within:ring-focus-ring"
      >
        <span>W</span>
        <input
          type="number"
          :min="Math.ceil(sourceWidth * 0.01)"
          :max="sourceWidth"
          :value="widthPixels"
          class="crop-dimension min-w-0 flex-1 bg-transparent text-label text-foreground tabular-nums outline-none"
          @change="setPixels('width', $event)"
        />
      </label>
      <UiIconButton
        :label="t('spaces.media.lockAspect')"
        :active="Boolean(lockedAspect)"
        class="size-(--knovum-control-height-lg)"
        @click="toggleAspect"
      >
        <Link2 v-if="lockedAspect" class="size-(--knovum-icon-size-sm)" />
        <Unlink2 v-else class="size-(--knovum-icon-size-sm)" />
      </UiIconButton>
      <label
        class="flex h-(--knovum-control-height-lg) min-w-0 flex-1 items-center gap-1 rounded-control bg-control px-2 text-label text-foreground-secondary focus-within:ring-2 focus-within:ring-focus-ring"
      >
        <span>H</span>
        <input
          type="number"
          :min="Math.ceil(sourceHeight * 0.01)"
          :max="sourceHeight"
          :value="heightPixels"
          class="crop-dimension min-w-0 flex-1 bg-transparent text-label text-foreground tabular-nums outline-none"
          @change="setPixels('height', $event)"
        />
      </label>
    </div>
    <SpaceImageCropPresets
      :active-ratio="lockedAspect"
      :full-crop="crop.x === 0 && crop.y === 0 && crop.width === 1 && crop.height === 1"
      @select="setAspect"
    />
    <footer class="mt-3 flex gap-2 border-t border-divider pt-3">
      <UiButton
        variant="secondary"
        size="sm"
        class="min-w-0 flex-1 border border-border bg-raised"
        @click="emit('cancel')"
        >{{ t("common.cancel") }}</UiButton
      >
      <UiButton
        variant="primary"
        size="sm"
        class="min-w-0 flex-1 bg-foreground text-canvas hover:bg-foreground hover:opacity-90"
        @click="emit('apply')"
        >{{ t("spaces.media.cropComplete") }}</UiButton
      >
    </footer>
  </section>
</template>

<style scoped>
.crop-dimension {
  appearance: textfield;
}

.crop-dimension::-webkit-inner-spin-button,
.crop-dimension::-webkit-outer-spin-button {
  appearance: none;
  margin: 0;
}
</style>
