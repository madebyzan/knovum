<script setup lang="ts">
import { FlipHorizontal, FlipVertical, RotateCcw } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiSlider from "../../../../shared/ui/UiSlider.vue";
import { setClipTransform } from "../edits/trim";
import { defaultTransform, type VideoClip, type VideoTransform } from "../project";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor; clip: VideoClip }>();
const { t } = useI18n();
const { project, commit } = props.editor;
const transform = computed(() => props.clip.transform);
const fits = computed(() => [
  { value: "contain" as const, label: t("spaces.media.videoEditor.framing.contain") },
  { value: "cover" as const, label: t("spaces.media.videoEditor.framing.cover") },
]);

function patch(change: Partial<VideoTransform>, mergeKey?: string) {
  if (!project.value) return;
  commit(setClipTransform(project.value, props.clip.id, change), mergeKey);
}
function setFit(value: string) {
  if (value === "contain" || value === "cover") patch({ fit: value });
}
</script>

<template>
  <section class="space-y-3 border-t border-stroke-divider pt-4">
    <div class="flex items-center justify-between">
      <h3 class="text-caption font-semibold">{{ t("spaces.media.videoEditor.framing.title") }}</h3>
      <UiButton size="sm" @click="patch(defaultTransform())">
        <RotateCcw class="size-3" />{{ t("spaces.media.videoEditor.framing.reset") }}
      </UiButton>
    </div>
    <UiSelect
      :model-value="transform.fit"
      :options="fits"
      size="sm"
      :label="t('spaces.media.videoEditor.framing.title')"
      @update:model-value="setFit"
    />
    <label class="block text-caption text-fg-secondary"
      >{{ t("spaces.media.videoEditor.framing.scale") }} · {{ Math.round(transform.scale * 100) }}%</label
    >
    <UiSlider
      :model-value="transform.scale"
      :min="0.1"
      :max="4"
      :step="0.01"
      :label="t('spaces.media.videoEditor.framing.scale')"
      @update:model-value="patch({ scale: $event }, `scale:${clip.id}`)"
    />
    <label class="block text-caption text-fg-secondary"
      >{{ t("spaces.media.videoEditor.framing.rotation") }} · {{ Math.round(transform.rotation) }}°</label
    >
    <UiSlider
      :model-value="transform.rotation"
      :min="-180"
      :max="180"
      :step="1"
      :label="t('spaces.media.videoEditor.framing.rotation')"
      @update:model-value="patch({ rotation: $event }, `rotation:${clip.id}`)"
    />
    <label class="block text-caption text-fg-secondary"
      >{{ t("spaces.media.videoEditor.framing.opacity") }} · {{ Math.round(transform.opacity * 100) }}%</label
    >
    <UiSlider
      :model-value="transform.opacity"
      :min="0"
      :max="1"
      :step="0.01"
      :label="t('spaces.media.videoEditor.framing.opacity')"
      @update:model-value="patch({ opacity: $event }, `opacity:${clip.id}`)"
    />
    <div class="grid grid-cols-2 gap-1">
      <UiButton
        size="sm"
        :aria-pressed="transform.flipHorizontal"
        @click="patch({ flipHorizontal: !transform.flipHorizontal })"
      >
        <FlipHorizontal class="size-3" />{{ t("spaces.media.transform.flipHorizontal") }}
      </UiButton>
      <UiButton
        size="sm"
        :aria-pressed="transform.flipVertical"
        @click="patch({ flipVertical: !transform.flipVertical })"
      >
        <FlipVertical class="size-3" />{{ t("spaces.media.transform.flipVertical") }}
      </UiButton>
    </div>
  </section>
</template>
