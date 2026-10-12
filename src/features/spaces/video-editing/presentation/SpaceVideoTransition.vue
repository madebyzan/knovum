<script setup lang="ts">
import { Blend } from "@lucide/vue";
import { PopoverRoot, PopoverTrigger, PopoverContent, PopoverPortal } from "reka-ui";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import { setTransition } from "../clip-edits";
import type { VideoClip, VideoEditProject, VideoTransition } from "../project";
import SpaceVideoTimeField from "./SpaceVideoTimeField.vue";
const props = defineProps<{ project: VideoEditProject; clip: VideoClip; index: number }>();
const emit = defineEmits<{ edit: [project: VideoEditProject] }>();
const { t } = useI18n();
const maximum = computed(() => {
  const clips = props.project.videoTrack.clips;
  const next = clips[props.index + 1]!;
  return Math.min(
    5000,
    props.clip.endMs - props.clip.startMs - (clips[props.index - 1]?.transition?.durationMs ?? 0) - 100,
    next.endMs - next.startMs - (next.transition?.durationMs ?? 0) - 100,
  );
});
const choices = computed(() => [
  { value: "dissolve", label: t("spaces.media.videoEditor.transitions.dissolve") },
  { value: "wipeLeft", label: t("spaces.media.videoEditor.transitions.wipeLeft") },
  { value: "wipeRight", label: t("spaces.media.videoEditor.transitions.wipeRight") },
]);
function choose(value: string) {
  if (value !== "dissolve" && value !== "wipeLeft" && value !== "wipeRight") return;
  edit({ kind: value, durationMs: props.clip.transition?.durationMs ?? Math.min(500, maximum.value) });
}
function edit(transition: VideoTransition | null) {
  emit("edit", setTransition(props.project, props.clip.id, transition));
}
</script>
<template>
  <PopoverRoot>
    <PopoverTrigger as-child>
      <button
        type="button"
        :disabled="maximum < 100"
        :aria-label="$t('spaces.media.videoEditor.transitions.title')"
        class="grid size-6 place-items-center rounded-md border border-stroke-default bg-surface-overlay text-fg-secondary shadow-control hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus disabled:opacity-40"
        :class="clip.transition && 'border-stroke-accent text-fg-accent'"
      >
        <Blend class="size-3.5" />
      </button>
    </PopoverTrigger>
    <PopoverPortal
      ><PopoverContent
        :side-offset="8"
        class="z-70 w-60 space-y-3 rounded-lg border border-stroke-default bg-surface-overlay p-3 shadow-popover"
      >
        <h3 class="text-label font-semibold">{{ $t("spaces.media.videoEditor.transitions.title") }}</h3>
        <UiSelect
          :label="$t('spaces.media.videoEditor.transitions.choose')"
          :model-value="clip.transition?.kind ?? ''"
          :options="choices"
          :placeholder="$t('spaces.media.videoEditor.transitions.choose')"
          size="sm"
          @update:model-value="choose"
        />
        <SpaceVideoTimeField
          v-if="clip.transition"
          :label="$t('spaces.media.videoEditor.transitions.duration')"
          :value="clip.transition.durationMs"
          :min="100"
          :max="maximum"
          @change="edit({ kind: clip.transition!.kind, durationMs: $event })"
        />
        <p class="text-caption-sm text-fg-secondary">{{ $t("spaces.media.videoEditor.transitions.hint") }}</p>
        <UiButton v-if="clip.transition" size="sm" @click="edit(null)">{{ $t("common.delete") }}</UiButton>
      </PopoverContent></PopoverPortal
    >
  </PopoverRoot>
</template>
