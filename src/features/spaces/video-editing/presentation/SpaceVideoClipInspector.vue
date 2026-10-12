<script setup lang="ts">
import { computed } from "vue";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSlider from "../../../../shared/ui/UiSlider.vue";
import { clipTrimBounds } from "../clip-bounds";
import { replaceClip } from "../clip-edits";
import { duplicateClip, moveClip } from "../clip-order";
import type { VideoClip, VideoEditProject } from "../project";
import SpaceVideoClipTransform from "./SpaceVideoClipTransform.vue";
import SpaceVideoFadeControls from "./SpaceVideoFadeControls.vue";
import SpaceVideoTimeField from "./SpaceVideoTimeField.vue";
const props = defineProps<{ project: VideoEditProject; clip: VideoClip }>();
const emit = defineEmits<{ edit: [project: VideoEditProject, mergeKey?: string] }>();
const index = computed(() => props.project.videoTrack.clips.findIndex(({ id }) => id === props.clip.id));
const asset = computed(() => props.project.assets.find(({ nodeId }) => nodeId === props.clip.assetId)!);
const bounds = computed(() => clipTrimBounds(props.project, props.clip));
function edit(next: () => VideoEditProject, mergeKey?: string) {
  try {
    emit("edit", next(), mergeKey);
  } catch (error) {
    toast.error(getErrorMessage(error));
  }
}
function patch(change: Partial<VideoClip>, mergeKey?: string) {
  edit(() => replaceClip(props.project, { ...props.clip, ...change }), mergeKey);
}
</script>
<template>
  <div class="space-y-5 p-3">
    <section class="grid grid-cols-2 gap-2">
      <SpaceVideoTimeField
        :label="$t('spaces.media.videoEditor.clip.inPoint')"
        :value="clip.startMs"
        :max="clip.endMs - bounds.minimum"
        @change="patch({ startMs: $event })"
      />
      <SpaceVideoTimeField
        :label="$t('spaces.media.videoEditor.clip.outPoint')"
        :value="clip.endMs"
        :min="clip.startMs + bounds.minimum"
        :max="bounds.maximum"
        @change="patch({ endMs: $event })"
      />
    </section>
    <section class="flex flex-wrap gap-1">
      <UiButton size="sm" :disabled="index === 0" @click="edit(() => moveClip(project, clip.id, -1))">{{
        $t("spaces.media.videoEditor.clip.moveLeft")
      }}</UiButton>
      <UiButton
        size="sm"
        :disabled="index === project.videoTrack.clips.length - 1"
        @click="edit(() => moveClip(project, clip.id, 1))"
        >{{ $t("spaces.media.videoEditor.clip.moveRight") }}</UiButton
      >
      <UiButton size="sm" @click="edit(() => duplicateClip(project, clip.id))">{{
        $t("spaces.media.videoEditor.clip.duplicate")
      }}</UiButton>
    </section>
    <section v-if="asset.kind === 'video'">
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
    <SpaceVideoClipTransform
      :transform="clip.transform"
      @change="patch({ transform: $event }, `transform:${clip.id}`)"
    />
  </div>
</template>
