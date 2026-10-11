<script setup lang="ts">
import { computed } from "vue";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSlider from "../../../../shared/ui/UiSlider.vue";
import { replaceClip, type VideoClip, type VideoEditProject } from "../../video-editing/project";

const props = defineProps<{
  project: VideoEditProject;
  clip: VideoClip;
  sourceDurationMs: number;
}>();
const emit = defineEmits<{ edit: [project: VideoEditProject, mergeKey?: string] }>();
function formatSeconds(milliseconds: number) {
  return (milliseconds / 1_000).toFixed(3);
}
const maximum = computed(() => Math.max(props.sourceDurationMs, props.clip.endMs));

function patch(change: Partial<VideoClip>, mergeKey?: string) {
  const next = { ...props.clip, ...change };
  if (next.endMs - next.startMs < 100) return;
  emit("edit", replaceClip(props.project, next), mergeKey);
}

function changeTime(event: Event, edge: "start" | "end") {
  const input = event.currentTarget as HTMLInputElement;
  const value = Math.round(Number(input.value) * 1_000);
  if (input.value.trim() && Number.isFinite(value) && value >= 0 && value <= maximum.value)
    patch({ [edge === "start" ? "startMs" : "endMs"]: value });
  input.value = formatSeconds(edge === "start" ? props.clip.startMs : props.clip.endMs);
}

function rotate() {
  const transform = props.project.transform;
  emit("edit", {
    ...props.project,
    transform: { ...transform, rotation: ((transform.rotation + 90) % 360) as 0 | 90 | 180 | 270 },
  });
}

function flip(edge: "flipHorizontal" | "flipVertical") {
  emit("edit", {
    ...props.project,
    transform: { ...props.project.transform, [edge]: !props.project.transform[edge] },
  });
}
</script>

<template>
  <div class="space-y-5 p-4">
    <section>
      <h3 class="text-caption font-semibold">{{ $t("spaces.media.videoEditor.clip.selected") }}</h3>
      <div class="mt-3 grid grid-cols-2 gap-2">
        <label class="text-caption text-fg-secondary" for="video-edit-clip-in">
          {{ $t("spaces.media.videoEditor.clip.inPoint") }}
          <UiInput
            id="video-edit-clip-in"
            class="mt-1 tabular-nums"
            size="sm"
            type="number"
            min="0"
            :max="formatSeconds(clip.endMs - 100)"
            step="0.001"
            :model-value="formatSeconds(clip.startMs)"
            @change="changeTime($event, 'start')"
          />
        </label>
        <label class="text-caption text-fg-secondary" for="video-edit-clip-out">
          {{ $t("spaces.media.videoEditor.clip.outPoint") }}
          <UiInput
            id="video-edit-clip-out"
            class="mt-1 tabular-nums"
            size="sm"
            type="number"
            :min="formatSeconds(clip.startMs + 100)"
            :max="formatSeconds(maximum)"
            step="0.001"
            :model-value="formatSeconds(clip.endMs)"
            @change="changeTime($event, 'end')"
          />
        </label>
      </div>
      <p class="mt-2 text-caption-sm text-fg-tertiary">
        {{ $t("spaces.media.videoEditor.trim.hint") }}
      </p>
    </section>
    <section>
      <label class="block text-caption text-fg-secondary">
        {{ $t("spaces.media.videoEditor.clip.volume") }} · {{ Math.round(clip.volume * 100) }}%
      </label>
      <UiSlider
        :model-value="clip.volume"
        :label="$t('spaces.media.videoEditor.clip.volume')"
        :min="0"
        :max="2"
        :step="0.05"
        @update:model-value="patch({ volume: $event }, `volume:${clip.id}`)"
      />
    </section>
    <section>
      <h3 class="mb-2 text-caption font-semibold">{{ $t("spaces.media.videoEditor.videoSettings") }}</h3>
      <div class="grid grid-cols-2 gap-1">
        <UiButton size="sm" @click="rotate">
          {{ $t("spaces.media.videoEditor.clip.rotate") }} {{ project.transform.rotation }}°
        </UiButton>
        <UiButton
          size="sm"
          :variant="project.transform.flipHorizontal ? 'primary' : 'secondary'"
          @click="flip('flipHorizontal')"
          >{{ $t("spaces.media.imageEditor.transform.flipHorizontal") }}</UiButton
        >
        <UiButton
          size="sm"
          :variant="project.transform.flipVertical ? 'primary' : 'secondary'"
          @click="flip('flipVertical')"
          >{{ $t("spaces.media.imageEditor.transform.flipVertical") }}</UiButton
        >
      </div>
    </section>
  </div>
</template>
