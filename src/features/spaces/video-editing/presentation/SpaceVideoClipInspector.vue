<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSlider from "../../../../shared/ui/UiSlider.vue";
import { closeGapBefore, swapClip } from "../edits/arrange";
import { setClipFades, setClipVolume, setClipWindow } from "../edits/trim";
import type { VideoClip, VideoEditProject } from "../project";
import { clipEndMs, clipIndexOf, clipLengthMs } from "../timing";
import SpaceVideoFadeFields from "./SpaceVideoFadeFields.vue";
import SpaceVideoTimeField from "./SpaceVideoTimeField.vue";
import SpaceVideoTransformFields from "./SpaceVideoTransformFields.vue";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor; clip: VideoClip }>();
const { t } = useI18n();
const { project, commit, actions } = props.editor;

const index = computed(() => (project.value ? clipIndexOf(project.value, props.clip.id) : -1));
const asset = computed(() => project.value?.assets.find(({ nodeId }) => nodeId === props.clip.assetId));
const maximumOut = computed(() =>
  asset.value && asset.value.kind !== "image" ? asset.value.durationMs : 86_400_000,
);
const hasGap = computed(() => {
  const current = project.value;
  if (!current || index.value < 0) return false;
  const previous = index.value > 0 ? current.videoTrack.clips[index.value - 1] : undefined;
  return props.clip.timelineStartMs > (previous ? clipEndMs(previous) : 0);
});

function apply(next: () => VideoEditProject, mergeKey?: string) {
  try {
    commit(next(), mergeKey);
  } catch (error) {
    toast.error(getErrorMessage(error));
  }
}
</script>

<template>
  <div class="space-y-5 p-3">
    <section class="grid grid-cols-2 gap-2">
      <SpaceVideoTimeField
        :label="t('spaces.media.videoEditor.clip.inPoint')"
        :value="clip.sourceInMs"
        :max="clip.sourceOutMs - 100"
        @change="apply(() => setClipWindow(project!, clip.id, { sourceInMs: $event }))"
      />
      <SpaceVideoTimeField
        :label="t('spaces.media.videoEditor.clip.outPoint')"
        :value="clip.sourceOutMs"
        :min="clip.sourceInMs + 100"
        :max="maximumOut"
        @change="apply(() => setClipWindow(project!, clip.id, { sourceOutMs: $event }))"
      />
    </section>
    <p class="text-caption text-fg-tertiary tabular-nums">
      {{ t("spaces.media.videoEditor.clip.placement") }}
      {{ (clip.timelineStartMs / 1000).toFixed(3) }}s · {{ (clipLengthMs(clip) / 1000).toFixed(3) }}s
    </p>
    <section class="flex flex-wrap gap-1">
      <UiButton size="sm" :disabled="index <= 0" @click="apply(() => swapClip(project!, clip.id, -1))">{{
        t("spaces.media.videoEditor.clip.moveLeft")
      }}</UiButton>
      <UiButton
        size="sm"
        :disabled="index < 0 || index === (project?.videoTrack.clips.length ?? 0) - 1"
        @click="apply(() => swapClip(project!, clip.id, 1))"
        >{{ t("spaces.media.videoEditor.clip.moveRight") }}</UiButton
      >
      <UiButton size="sm" @click="actions.duplicate">{{
        t("spaces.media.videoEditor.clip.duplicate")
      }}</UiButton>
      <UiButton v-if="hasGap" size="sm" @click="apply(() => closeGapBefore(project!, clip.id))">{{
        t("spaces.media.videoEditor.clip.closeGap")
      }}</UiButton>
    </section>
    <section v-if="asset?.kind === 'video'">
      <label class="block text-caption text-fg-secondary"
        >{{ t("spaces.media.videoEditor.clip.volume") }} · {{ Math.round(clip.volume * 100) }}%</label
      >
      <UiSlider
        :model-value="clip.volume"
        :min="0"
        :max="2"
        :step="0.05"
        :label="t('spaces.media.videoEditor.clip.volume')"
        @update:model-value="apply(() => setClipVolume(project!, clip.id, $event), `volume:${clip.id}`)"
      />
    </section>
    <SpaceVideoFadeFields
      :fade-in-ms="clip.fadeInMs"
      :fade-out-ms="clip.fadeOutMs"
      :duration-ms="clipLengthMs(clip)"
      @change="apply(() => setClipFades(project!, clip.id, $event))"
    />
    <SpaceVideoTransformFields :editor="editor" :clip="clip" />
  </div>
</template>
