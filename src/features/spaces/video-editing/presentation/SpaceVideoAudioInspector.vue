<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiSlider from "../../../../shared/ui/UiSlider.vue";
import { moveAudioClip, setAudioClipGain, setAudioClipLane, trimAudioClip } from "../edits/audio";
import type { VideoAudioClip } from "../project";
import { AUDIO_LANE_LIMIT } from "../timeline/track-geometry";
import { clipEndMs, clipLengthMs, projectDurationMs } from "../timing";
import SpaceVideoFadeFields from "./SpaceVideoFadeFields.vue";
import SpaceVideoTimeField from "./SpaceVideoTimeField.vue";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor; clip: VideoAudioClip }>();
const { t } = useI18n();
const { project, commit } = props.editor;
const asset = computed(() => project.value?.assets.find(({ nodeId }) => nodeId === props.clip.assetId));
const duration = computed(() => (project.value ? projectDurationMs(project.value) : 0));
const lanes = computed(() =>
  Array.from({ length: AUDIO_LANE_LIMIT }, (_, index) => ({
    value: String(index),
    label: `${t("spaces.media.videoEditor.tracks.audio")} ${index + 1}`,
  })),
);

function setLane(value: string) {
  const lane = Number(value);
  if (project.value && !Number.isNaN(lane)) commit(setAudioClipLane(project.value, props.clip.id, lane));
}
</script>

<template>
  <div class="space-y-5 p-3">
    <SpaceVideoTimeField
      :label="t('spaces.media.videoEditor.clip.startsAt')"
      :value="clip.timelineStartMs"
      :max="Math.max(0, duration - clipLengthMs(clip))"
      @change="commit(moveAudioClip(project!, clip.id, $event))"
    />
    <section class="grid grid-cols-2 gap-2">
      <SpaceVideoTimeField
        :label="t('spaces.media.videoEditor.clip.inPoint')"
        :value="clip.sourceInMs"
        :max="clip.sourceOutMs - 100"
        @change="
          commit(trimAudioClip(project!, clip.id, 'start', clip.timelineStartMs + ($event - clip.sourceInMs)))
        "
      />
      <SpaceVideoTimeField
        :label="t('spaces.media.videoEditor.clip.outPoint')"
        :value="clip.sourceOutMs"
        :min="clip.sourceInMs + 100"
        :max="asset?.durationMs ?? clip.sourceOutMs"
        @change="
          commit(trimAudioClip(project!, clip.id, 'end', clipEndMs(clip) + ($event - clip.sourceOutMs)))
        "
      />
    </section>
    <label class="block space-y-1 text-caption text-fg-secondary">
      {{ t("spaces.media.videoEditor.tracks.lane") }}
      <UiSelect
        size="sm"
        :model-value="String(clip.lane)"
        :options="lanes"
        :label="t('spaces.media.videoEditor.tracks.lane')"
        @update:model-value="setLane"
      />
    </label>
    <section>
      <label class="block text-caption text-fg-secondary"
        >{{ t("spaces.media.videoEditor.clip.volume") }} · {{ Math.round(clip.volume * 100) }}%</label
      >
      <UiSlider
        :model-value="clip.volume"
        :min="0"
        :max="2"
        :step="0.05"
        :label="t('spaces.media.videoEditor.clip.volume')"
        @update:model-value="
          commit(setAudioClipGain(project!, clip.id, { volume: $event }), `volume:${clip.id}`)
        "
      />
    </section>
    <SpaceVideoFadeFields
      :fade-in-ms="clip.fadeInMs"
      :fade-out-ms="clip.fadeOutMs"
      :duration-ms="clipLengthMs(clip)"
      @change="commit(setAudioClipGain(project!, clip.id, $event))"
    />
  </div>
</template>
