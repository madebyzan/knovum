<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import {
  DEFAULT_VIDEO_TEXT_STYLE,
  projectDuration,
  type VideoEditProject,
  type VideoTextOverlay,
  type VideoTextStyle,
} from "../../video-editing/project";
import SpaceVideoTextTiming from "./SpaceVideoTextTiming.vue";

const props = defineProps<{ project: VideoEditProject; item: VideoTextOverlay }>();
const emit = defineEmits<{ edit: [project: VideoEditProject, mergeKey?: string] }>();
const duration = computed(() => projectDuration(props.project));
const style = computed(() => props.item.style ?? DEFAULT_VIDEO_TEXT_STYLE);
const textDraft = shallowRef(props.item.text);
const colorError = shallowRef(false);
const presets = ["plain", "outline", "shadow", "badge"] as const;
const placements = ["top", "center", "bottom"] as const;

watch(
  () => props.item.text,
  (value) => {
    textDraft.value = value;
  },
);

function patch(patch: Partial<VideoTextOverlay>, mergeKey?: string) {
  const next = { ...props.item, ...patch };
  if (
    !next.text.trim() ||
    next.text.length > 500 ||
    next.endMs - next.startMs < 100 ||
    next.endMs > duration.value
  )
    return;
  emit(
    "edit",
    {
      ...props.project,
      textTrack: { items: props.project.textTrack.items.map((item) => (item.id === next.id ? next : item)) },
    },
    mergeKey,
  );
}

function patchStyle(value: Partial<VideoTextStyle>, mergeKey?: string) {
  patch({ style: { ...style.value, ...value } }, mergeKey);
}

function changeSize(event: Event) {
  const input = event.currentTarget as HTMLInputElement;
  const value = Number(input.value);
  if (Number.isInteger(value) && value >= 16 && value <= 96) patchStyle({ size: value });
  input.value = String(style.value.size);
}

function changeColor(value: string) {
  colorError.value = !/^#[0-9A-Fa-f]{6}$/.test(value);
  if (!colorError.value) patchStyle({ color: value.toUpperCase() }, `text-color:${props.item.id}`);
}

function commitColor(event: Event) {
  const input = event.currentTarget as HTMLInputElement;
  changeColor(input.value);
  if (colorError.value) input.value = style.value.color;
}
</script>

<template>
  <div class="space-y-5 p-4">
    <section>
      <label class="mb-1.5 block text-caption font-medium" for="video-edit-text">
        {{ $t("spaces.media.textContent") }}
      </label>
      <UiTextarea
        id="video-edit-text"
        v-model="textDraft"
        class="min-h-24 resize-none"
        :rows="4"
        @update:model-value="(value) => patch({ text: value }, `text:${item.id}`)"
        @blur="textDraft = item.text"
      />
    </section>
    <section class="grid grid-cols-2 gap-2">
      <div>
        <span class="mb-1.5 block text-caption text-foreground-secondary">{{
          $t("spaces.media.fontFamily")
        }}</span>
        <div
          class="flex h-(--knovum-control-height-sm) items-center rounded-control border border-border bg-field px-2 text-caption text-foreground-secondary"
        >
          Noto Sans CJK
          <span class="ml-auto text-micro text-foreground-tertiary">{{
            $t("spaces.media.bundledFont")
          }}</span>
        </div>
      </div>
      <label class="text-caption text-foreground-secondary" for="video-edit-text-size">
        {{ $t("spaces.media.fontSize") }}
        <UiInput
          id="video-edit-text-size"
          class="mt-1 tabular-nums"
          size="sm"
          type="number"
          min="16"
          max="96"
          step="1"
          :model-value="String(style.size)"
          @change="changeSize"
        />
      </label>
    </section>
    <section>
      <label class="mb-1.5 block text-caption text-foreground-secondary" for="video-edit-text-color">
        {{ $t("spaces.media.fontColor") }}
      </label>
      <div class="flex items-center gap-2 rounded-control border border-border bg-field px-2">
        <input
          :value="style.color"
          type="color"
          class="size-7 shrink-0 cursor-pointer border-0 bg-transparent"
          :aria-label="$t('spaces.media.fontColor')"
          @input="changeColor(($event.target as HTMLInputElement).value)"
        />
        <UiInput
          id="video-edit-text-color"
          class="border-0 bg-transparent font-mono shadow-none"
          size="sm"
          :model-value="style.color"
          @change="commitColor"
        />
      </div>
      <p v-if="colorError" class="mt-1 text-micro text-danger">{{ $t("spaces.media.invalidTextColor") }}</p>
    </section>
    <section>
      <h3 class="mb-2 text-caption text-foreground-secondary">{{ $t("spaces.media.textPresets") }}</h3>
      <div class="grid grid-cols-4 gap-1.5">
        <button
          v-for="preset in presets"
          :key="preset"
          type="button"
          class="flex h-14 flex-col items-center justify-center gap-0.5 rounded-control border text-micro focus-visible:outline-2 focus-visible:outline-focus-ring"
          :class="
            style.preset === preset
              ? 'border-accent bg-selected text-accent'
              : 'border-border bg-field text-foreground-secondary hover:bg-control-hover'
          "
          :aria-pressed="style.preset === preset"
          @click="patchStyle({ preset })"
        >
          <span
            class="text-label font-semibold"
            :class="preset === 'badge' && 'rounded-control bg-foreground px-1 text-canvas'"
            >T</span
          >
          {{ $t(`spaces.media.textPreset.${preset}`) }}
        </button>
      </div>
    </section>
    <section>
      <h3 class="mb-2 text-caption text-foreground-secondary">{{ $t("spaces.media.textPlacement") }}</h3>
      <div class="grid grid-cols-3 gap-1">
        <UiButton
          v-for="placement in placements"
          :key="placement"
          size="sm"
          :variant="item.placement === placement ? 'primary' : 'secondary'"
          @click="patch({ placement })"
          >{{ $t(`spaces.media.${placement}`) }}</UiButton
        >
      </div>
    </section>
    <SpaceVideoTextTiming :item="item" :duration-ms="duration" @change="patch" />
  </div>
</template>
