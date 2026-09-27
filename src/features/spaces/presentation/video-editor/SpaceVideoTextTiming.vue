<script setup lang="ts">
import UiInput from "../../../../shared/ui/UiInput.vue";
import type { VideoTextOverlay } from "../../video-editing/project";

const props = defineProps<{ item: VideoTextOverlay; durationMs: number }>();
const emit = defineEmits<{ change: [patch: Partial<VideoTextOverlay>] }>();
const seconds = (milliseconds: number) => (milliseconds / 1_000).toFixed(3);

function changeTime(event: Event, edge: "start" | "end") {
  const input = event.currentTarget as HTMLInputElement;
  const value = Math.round(Number(input.value) * 1_000);
  if (input.value.trim() && Number.isFinite(value) && value >= 0 && value <= props.durationMs)
    emit("change", { [edge === "start" ? "startMs" : "endMs"]: value });
  input.value = seconds(edge === "start" ? props.item.startMs : props.item.endMs);
}
</script>

<template>
  <section class="grid grid-cols-2 gap-2 border-t border-stroke-divider pt-4">
    <label class="text-caption-md text-fg-secondary" for="video-edit-text-start">
      {{ $t("spaces.media.videoEditor.clip.startsAt") }}
      <UiInput
        id="video-edit-text-start"
        class="mt-1 tabular-nums"
        size="sm"
        type="number"
        min="0"
        :max="seconds(item.endMs - 100)"
        step="0.001"
        :model-value="seconds(item.startMs)"
        @change="changeTime($event, 'start')"
      />
    </label>
    <label class="text-caption-md text-fg-secondary" for="video-edit-text-end">
      {{ $t("spaces.media.videoEditor.clip.endsAt") }}
      <UiInput
        id="video-edit-text-end"
        class="mt-1 tabular-nums"
        size="sm"
        type="number"
        :min="seconds(item.startMs + 100)"
        :max="seconds(durationMs)"
        step="0.001"
        :model-value="seconds(item.endMs)"
        @change="changeTime($event, 'end')"
      />
    </label>
  </section>
</template>
