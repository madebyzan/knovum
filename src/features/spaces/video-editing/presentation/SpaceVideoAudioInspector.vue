<script setup lang="ts">
import { computed } from "vue";

import UiSlider from "../../../../shared/ui/UiSlider.vue";
import type { VideoAudioClip, VideoEditProject } from "../project";
import { projectDurationMs } from "../timing";
import SpaceVideoFadeControls from "./SpaceVideoFadeControls.vue";
import SpaceVideoTimeField from "./SpaceVideoTimeField.vue";
const props = defineProps<{ project: VideoEditProject; clip: VideoAudioClip }>();
const emit = defineEmits<{ edit: [project: VideoEditProject, mergeKey?: string] }>();
const duration = computed(() => projectDurationMs(props.project));
const assetDuration = computed(
  () => props.project.assets.find(({ nodeId }) => nodeId === props.clip.assetId)!.durationMs,
);
function patch(change: Partial<VideoAudioClip>, mergeKey?: string) {
  emit(
    "edit",
    {
      ...props.project,
      audioTrack: {
        clips: props.project.audioTrack.clips.map((clip) =>
          clip.id === props.clip.id ? Object.assign({}, clip, change) : clip,
        ),
      },
    },
    mergeKey,
  );
}
</script>
<template>
  <div class="space-y-5 p-3">
    <SpaceVideoTimeField
      :label="$t('spaces.media.videoEditor.clip.startsAt')"
      :value="clip.timelineStartMs"
      :max="duration - clip.endMs + clip.startMs"
      @change="patch({ timelineStartMs: $event })"
    />
    <div class="grid grid-cols-2 gap-2">
      <SpaceVideoTimeField
        :label="$t('spaces.media.videoEditor.clip.inPoint')"
        :value="clip.startMs"
        :max="clip.endMs - Math.max(100, clip.fadeInMs + clip.fadeOutMs)"
        @change="patch({ startMs: $event })"
      />
      <SpaceVideoTimeField
        :label="$t('spaces.media.videoEditor.clip.outPoint')"
        :value="clip.endMs"
        :min="clip.startMs + Math.max(100, clip.fadeInMs + clip.fadeOutMs)"
        :max="Math.min(assetDuration, clip.startMs + duration - clip.timelineStartMs)"
        @change="patch({ endMs: $event })"
      />
    </div>
    <section>
      <label class="block text-caption text-fg-secondary"
        >{{ $t("spaces.media.videoEditor.clip.volume") }} · {{ Math.round(clip.volume * 100) }}%</label
      >
      <UiSlider
        :model-value="clip.volume"
        :min="0"
        :max="2"
        :step="0.05"
        :label="$t('spaces.media.videoEditor.clip.volume')"
        @update:model-value="patch({ volume: $event }, `volume:${clip.id}`)"
      />
    </section>
    <SpaceVideoFadeControls
      :fade-in-ms="clip.fadeInMs"
      :fade-out-ms="clip.fadeOutMs"
      :duration-ms="clip.endMs - clip.startMs"
      @change="patch($event)"
    />
  </div>
</template>
