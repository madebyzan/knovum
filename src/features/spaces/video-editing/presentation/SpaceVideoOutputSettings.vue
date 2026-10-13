<script setup lang="ts">
import { Settings2 } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { VideoEditProject } from "../project";
import SpaceVideoColorField from "./SpaceVideoColorField.vue";

const props = defineProps<{ project: VideoEditProject }>();
const emit = defineEmits<{ edit: [project: VideoEditProject] }>();
const { t } = useI18n();

const FRAMES = [24, 25, 30, 60] as const;
const SIZES = [
  { value: "1080x1920", label: "1080 × 1920 · 9:16" },
  { value: "1920x1080", label: "1920 × 1080 · 16:9" },
  { value: "1080x1080", label: "1080 × 1080 · 1:1" },
  { value: "1280x720", label: "1280 × 720 · 16:9" },
  { value: "3840x2160", label: "3840 × 2160 · 16:9" },
];
const size = computed(() => `${props.project.output.width}x${props.project.output.height}`);
const sizes = computed(() =>
  SIZES.some(({ value }) => value === size.value)
    ? SIZES
    : [{ value: size.value, label: size.value.replace("x", " × ") }, ...SIZES],
);
const rates = computed(() => FRAMES.map((value) => ({ value: String(value), label: `${value} fps` })));

function patch(output: Partial<VideoEditProject["output"]>) {
  emit("edit", { ...props.project, output: { ...props.project.output, ...output } });
}
function setSize(value: string) {
  const [width, height] = value.split("x").map(Number);
  if (width && height) patch({ width, height });
}
function setRate(value: string) {
  const rate = Number(value);
  if (FRAMES.some((candidate) => candidate === rate)) patch({ frameRate: rate as 24 | 25 | 30 | 60 });
}
</script>

<template>
  <UiPopover align="end" class="w-72 space-y-3 p-3">
    <template #trigger>
      <UiIconButton size="sm" :label="t('spaces.media.videoEditor.output.title')" tooltip>
        <Settings2 class="size-3.5" />
      </UiIconButton>
    </template>
    <h2 class="text-label font-semibold">{{ t("spaces.media.videoEditor.output.title") }}</h2>
    <label class="block space-y-1 text-caption text-fg-secondary">
      {{ t("spaces.media.videoEditor.output.size") }}
      <UiSelect
        :label="t('spaces.media.videoEditor.output.size')"
        :model-value="size"
        :options="sizes"
        size="sm"
        @update:model-value="setSize"
      />
    </label>
    <label class="block space-y-1 text-caption text-fg-secondary">
      {{ t("spaces.media.videoEditor.output.frameRate") }}
      <UiSelect
        :label="t('spaces.media.videoEditor.output.frameRate')"
        :model-value="String(project.output.frameRate)"
        :options="rates"
        size="sm"
        @update:model-value="setRate"
      />
    </label>
    <SpaceVideoColorField
      :label="t('spaces.media.videoEditor.output.background')"
      :color="project.output.background"
      @change="patch({ background: $event })"
    />
  </UiPopover>
</template>
