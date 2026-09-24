<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import {
  projectDuration,
  type VideoEditProject,
  type VideoTextOverlay,
  type VideoTextStyle,
} from "../../video-editing/project";
import SpaceVideoFontControls from "./SpaceVideoFontControls.vue";
import SpaceVideoTextColorField from "./SpaceVideoTextColorField.vue";
import SpaceVideoTextTiming from "./SpaceVideoTextTiming.vue";

const props = defineProps<{ project: VideoEditProject; item: VideoTextOverlay }>();
const emit = defineEmits<{ edit: [project: VideoEditProject, mergeKey?: string] }>();
const duration = computed(() => projectDuration(props.project));
const style = computed(() => props.item.style);
const textDraft = shallowRef(props.item.text);
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
    <SpaceVideoFontControls :style="style" @change="patchStyle" />
    <section class="grid grid-cols-2 gap-2">
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
      <SpaceVideoTextColorField
        :key="item.id"
        :color="style.color"
        @change="patchStyle({ color: $event }, `text-color:${item.id}`)"
      />
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
