<script setup lang="ts">
import UiInput from "../../../../shared/ui/UiInput.vue";
import type { VideoTextOverlay } from "../../video-editing/project";

const props = defineProps<{ item: VideoTextOverlay; durationMs: number }>();
const emit = defineEmits<{ change: [patch: Partial<VideoTextOverlay>] }>();
function formatSeconds(milliseconds: number) {
  return (milliseconds / 1_000).toFixed(3);
}

function changeTime(event: Event, edge: "start" | "end") {
  const input = event.currentTarget as HTMLInputElement;
  const value = Math.round(Number(input.value) * 1_000);
  if (input.value.trim() && Number.isFinite(value) && value >= 0 && value <= props.durationMs)
    emit("change", { [edge === "start" ? "startMs" : "endMs"]: value });
  input.value = formatSeconds(edge === "start" ? props.item.startMs : props.item.endMs);
}
</script>

<template>
  <section class="grid grid-cols-2 gap-2 border-t border-stroke-divider pt-4">
    <label class="text-caption text-fg-secondary" for="video-edit-text-start">
      {{ $t("spaces.media.videoEditor.clip.startsAt") }}
      <UiInput
        id="video-edit-text-start"
        class="mt-1 tabular-nums"
        size="sm"
        type="number"
        min="0"
        :max="formatSeconds(item.endMs - 100)"
        step="0.001"
        :model-value="formatSeconds(item.startMs)"
        @change="changeTime($event, 'start')"
      />
    </label>
    <label class="text-caption text-fg-secondary" for="video-edit-text-end">
      {{ $t("spaces.media.videoEditor.clip.endsAt") }}
      <UiInput
        id="video-edit-text-end"
        class="mt-1 tabular-nums"
        size="sm"
        type="number"
        :min="formatSeconds(item.startMs + 100)"
        :max="formatSeconds(durationMs)"
        step="0.001"
        :model-value="formatSeconds(item.endMs)"
        @change="changeTime($event, 'end')"
      />
    </label>
  </section>
</template>
