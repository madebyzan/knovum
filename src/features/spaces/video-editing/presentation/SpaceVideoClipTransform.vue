<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiSlider from "../../../../shared/ui/UiSlider.vue";
import type { VideoClip } from "../project";
const props = defineProps<{ transform: VideoClip["transform"] }>();
const emit = defineEmits<{ change: [transform: VideoClip["transform"]] }>();
const { t } = useI18n();
const choices = computed(() => [
  { value: "contain", label: t("spaces.media.videoEditor.framing.contain") },
  { value: "cover", label: t("spaces.media.videoEditor.framing.cover") },
]);
function patch(change: Partial<VideoClip["transform"]>) {
  emit("change", { ...props.transform, ...change });
}
function setFit(value: string) {
  if (value === "contain" || value === "cover") patch({ fit: value });
}
</script>
<template>
  <section class="space-y-3">
    <h3 class="text-caption font-semibold">{{ $t("spaces.media.videoEditor.framing.title") }}</h3>
    <UiSelect
      :model-value="transform.fit"
      :options="choices"
      size="sm"
      :label="$t('spaces.media.videoEditor.framing.title')"
      @update:model-value="setFit"
    />
    <label class="block text-caption text-fg-secondary"
      >{{ $t("spaces.media.videoEditor.framing.zoom") }} · {{ Math.round(transform.zoom * 100) }}%</label
    >
    <UiSlider
      :model-value="transform.zoom"
      :min="1"
      :max="4"
      :step="0.05"
      :label="$t('spaces.media.videoEditor.framing.zoom')"
      @update:model-value="patch({ zoom: $event })"
    />
    <div class="grid grid-cols-2 gap-1">
      <UiButton
        size="sm"
        @click="patch({ rotation: ((transform.rotation + 90) % 360) as 0 | 90 | 180 | 270 })"
        >{{ $t("spaces.media.videoEditor.clip.rotate") }} {{ transform.rotation }}°</UiButton
      >
      <UiButton
        size="sm"
        :aria-pressed="transform.flipHorizontal"
        @click="patch({ flipHorizontal: !transform.flipHorizontal })"
        >{{ $t("spaces.media.transform.flipHorizontal") }}</UiButton
      >
      <UiButton
        size="sm"
        :aria-pressed="transform.flipVertical"
        @click="patch({ flipVertical: !transform.flipVertical })"
        >{{ $t("spaces.media.transform.flipVertical") }}</UiButton
      >
    </div>
  </section>
</template>
