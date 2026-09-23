<script setup lang="ts">
import { ChevronUp } from "@lucide/vue";
import { computed } from "vue";

import type { ImageEdit } from "../../image-editing/image-edit";
import SpaceImageEditedThumbnail from "./SpaceImageEditedThumbnail.vue";

const props = defineProps<{
  path: string;
  revision?: string;
  edit: ImageEdit;
  yaw: number;
  pitch: number;
  zoom: "close" | "medium" | "wide";
}>();
const scale = computed(() => (props.zoom === "close" ? 1.12 : props.zoom === "wide" ? 0.86 : 1));
const orientation = computed(() => ({
  transform: `rotateY(${props.yaw}deg) rotateX(${-props.pitch}deg) scale(${scale.value})`,
}));
const face =
  "absolute inset-0 grid place-items-center rounded-control border border-border-strong bg-raised text-label font-medium text-foreground-tertiary [backface-visibility:hidden]";
</script>

<template>
  <div class="pointer-events-none relative size-44 [perspective:520px]" aria-hidden="true">
    <div
      class="absolute inset-6 transition-transform duration-150 ease-standard [transform-style:preserve-3d] motion-reduce:transition-none"
      :style="orientation"
    >
      <div :class="face" class="[transform:rotateY(-90deg)_translateZ(64px)]">L</div>
      <div :class="face" class="[transform:rotateY(90deg)_translateZ(64px)]">R</div>
      <div :class="face" class="[transform:rotateY(180deg)_translateZ(64px)]" />
      <div :class="face" class="[transform:rotateX(90deg)_translateZ(64px)]">
        <ChevronUp class="size-(--knovum-icon-size-xs) text-foreground-tertiary" :stroke-width="1.3" />
      </div>
      <div :class="face" class="[transform:rotateX(-90deg)_translateZ(64px)]">B</div>
      <div :class="face" class="[transform:translateZ(64px)] overflow-hidden p-1">
        <SpaceImageEditedThumbnail :path="path" :revision="revision" :edit="edit" />
      </div>
    </div>
  </div>
</template>
