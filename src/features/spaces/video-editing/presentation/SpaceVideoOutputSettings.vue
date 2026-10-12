<script setup lang="ts">
import { Settings2 } from "@lucide/vue";
import { PopoverContent, PopoverPortal, PopoverRoot, PopoverTrigger } from "reka-ui";
import { computed } from "vue";

import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { VideoEditProject } from "../project";
const props = defineProps<{ project: VideoEditProject }>();
const emit = defineEmits<{ edit: [project: VideoEditProject] }>();
const sizes = [
  { value: "1280x720", label: "720p · 16:9" },
  { value: "1920x1080", label: "1080p · 16:9" },
  { value: "1080x1920", label: "1080p · 9:16" },
  { value: "1080x1080", label: "1080p · 1:1" },
  { value: "3840x2160", label: "4K · 16:9" },
];
const size = computed(() => `${props.project.output.width}x${props.project.output.height}`);
const choices = computed(() =>
  sizes.some(({ value }) => value === size.value)
    ? sizes
    : [{ value: size.value, label: size.value }, ...sizes],
);
function setSize(value: string) {
  const choice = choices.value.find((candidate) => candidate.value === value);
  if (!choice) return;
  const [width, height] = value.split("x").map(Number);
  emit("edit", { ...props.project, output: { ...props.project.output, width: width!, height: height! } });
}
function setRate(value: string) {
  const rate = Number(value);
  if ([24, 25, 30, 60].includes(rate))
    emit("edit", {
      ...props.project,
      output: { ...props.project.output, frameRate: rate as 24 | 25 | 30 | 60 },
    });
}
</script>
<template>
  <PopoverRoot>
    <PopoverTrigger as-child
      ><UiIconButton size="sm" :label="$t('spaces.media.videoEditor.output.title')" tooltip
        ><Settings2 class="size-3.5" /></UiIconButton
    ></PopoverTrigger>
    <PopoverPortal
      ><PopoverContent
        align="end"
        :side-offset="8"
        class="z-70 w-64 space-y-3 rounded-lg border border-stroke-default bg-surface-overlay p-3 shadow-popover"
      >
        <h2 class="text-label font-semibold">{{ $t("spaces.media.videoEditor.output.title") }}</h2>
        <label class="block space-y-1 text-caption text-fg-secondary"
          >{{ $t("spaces.media.videoEditor.output.size")
          }}<UiSelect
            :label="$t('spaces.media.videoEditor.output.size')"
            :model-value="size"
            :options="choices"
            size="sm"
            @update:model-value="setSize"
        /></label>
        <label class="block space-y-1 text-caption text-fg-secondary"
          >{{ $t("spaces.media.videoEditor.output.frameRate")
          }}<UiSelect
            :label="$t('spaces.media.videoEditor.output.frameRate')"
            :model-value="String(project.output.frameRate)"
            :options="[24, 25, 30, 60].map((value) => ({ value: String(value), label: `${value} fps` }))"
            size="sm"
            @update:model-value="setRate"
        /></label> </PopoverContent
    ></PopoverPortal>
  </PopoverRoot>
</template>
