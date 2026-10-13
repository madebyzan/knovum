<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiNumberField from "../../../../shared/ui/UiNumberField.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import { updateTextOverlay, updateTextStyle } from "../edits/text";
import type { VideoTextOverlay, VideoTextStyle } from "../project";
import { projectDurationMs } from "../timing";
import SpaceVideoColorField from "./SpaceVideoColorField.vue";
import SpaceVideoFontControls from "./SpaceVideoFontControls.vue";
import SpaceVideoTimeField from "./SpaceVideoTimeField.vue";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor; item: VideoTextOverlay }>();
const { t } = useI18n();
const { project, commit } = props.editor;
const DECORATIONS = ["plain", "outline", "shadow", "box"] as const;
const ALIGNMENTS = ["left", "center", "right"] as const;
const draft = shallowRef(props.item.text);
const duration = computed(() => (project.value ? projectDurationMs(project.value) : 0));

watch(
  () => props.item.text,
  (value) => (draft.value = value),
);

function patch(change: Partial<VideoTextOverlay>, mergeKey?: string) {
  if (project.value) commit(updateTextOverlay(project.value, props.item.id, change), mergeKey);
}
function patchStyle(change: Partial<VideoTextStyle>, mergeKey?: string) {
  if (project.value) commit(updateTextStyle(project.value, props.item.id, change), mergeKey);
}
function patchSize(value: number | undefined) {
  patchStyle({ size: Math.round(value ?? props.item.style.size) }, `size:${props.item.id}`);
}
function patchStrength(value: number | undefined) {
  patchStyle({ decorationStrength: Math.round(value ?? props.item.style.decorationStrength) });
}
</script>

<template>
  <div class="space-y-4 p-3">
    <section>
      <label class="mb-1.5 block text-caption font-medium" for="video-edit-text">
        {{ t("spaces.media.videoEditor.text.content") }}
      </label>
      <UiTextarea
        id="video-edit-text"
        v-model="draft"
        class="min-h-20 resize-none"
        :rows="3"
        @update:model-value="patch({ text: draft }, `text:${item.id}`)"
        @blur="draft = item.text"
      />
    </section>
    <SpaceVideoFontControls :style="item.style" @change="patchStyle" />
    <section class="grid grid-cols-2 gap-2">
      <label class="text-caption text-fg-secondary">
        {{ t("spaces.media.videoEditor.font.size") }}
        <UiNumberField
          class="mt-1"
          size="sm"
          :min="12"
          :max="200"
          :step="1"
          :model-value="item.style.size"
          @update:model-value="patchSize($event)"
        />
      </label>
      <label class="text-caption text-fg-secondary">
        {{ t("spaces.media.videoEditor.text.alignment") }}
        <UiSelect
          class="mt-1"
          size="sm"
          :model-value="item.style.align"
          :label="t('spaces.media.videoEditor.text.alignment')"
          :options="
            ALIGNMENTS.map((value) => ({
              value,
              label: t(`spaces.media.videoEditor.text.alignments.${value}`),
            }))
          "
          @update:model-value="patchStyle({ align: $event })"
        />
      </label>
    </section>
    <SpaceVideoColorField
      :key="`color-${item.id}`"
      :label="t('spaces.media.videoEditor.font.color')"
      :color="item.style.color"
      @change="patchStyle({ color: $event }, `color:${item.id}`)"
    />
    <section>
      <h3 class="mb-2 text-caption text-fg-secondary">
        {{ t("spaces.media.videoEditor.text.presets.title") }}
      </h3>
      <div class="grid grid-cols-4 gap-1.5">
        <UiButton
          v-for="decoration in DECORATIONS"
          :key="decoration"
          size="sm"
          :variant="item.style.decoration === decoration ? 'primary' : 'secondary'"
          class="justify-center"
          @click="patchStyle({ decoration })"
          >{{ t(`spaces.media.videoEditor.text.presets.${decoration}`) }}</UiButton
        >
      </div>
    </section>
    <section v-if="item.style.decoration !== 'plain'" class="space-y-2">
      <label class="text-caption text-fg-secondary">
        {{ t("spaces.media.videoEditor.text.strength") }}
        <UiNumberField
          class="mt-1"
          size="sm"
          :min="0"
          :max="20"
          :step="1"
          :model-value="item.style.decorationStrength"
          @update:model-value="patchStrength($event)"
        />
      </label>
      <SpaceVideoColorField
        :key="`decoration-${item.id}`"
        :label="t('spaces.media.videoEditor.text.decorationColor')"
        :color="item.style.decorationColor"
        @change="patchStyle({ decorationColor: $event }, `decoration:${item.id}`)"
      />
    </section>
    <section class="grid grid-cols-2 gap-2 border-t border-stroke-divider pt-4">
      <SpaceVideoTimeField
        :label="t('spaces.media.videoEditor.clip.startsAt')"
        :value="item.startMs"
        :max="Math.max(0, duration - 100)"
        @change="patch({ startMs: $event, endMs: $event + (item.endMs - item.startMs) })"
      />
      <SpaceVideoTimeField
        :label="t('spaces.media.videoEditor.clip.endsAt')"
        :value="item.endMs"
        :min="item.startMs + 100"
        :max="duration"
        @change="patch({ endMs: $event })"
      />
    </section>
    <p class="text-caption-sm text-fg-tertiary">{{ t("spaces.media.videoEditor.text.dragHint") }}</p>
  </div>
</template>
