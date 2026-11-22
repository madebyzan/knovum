<script setup lang="ts">
import { computed, onBeforeUnmount, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiSearchInput from "../../../../shared/ui/UiSearchInput.vue";
import { versionedMediaSource, waitForMediaMetadata } from "../../media/media-element-source";
import type { SpaceAudioPayload, SpaceVideoPayload } from "../../model/nodes/node-payload";

const props = defineProps<{
  media: SpaceVideoPayload | SpaceAudioPayload;
  hasSegments: boolean;
}>();

const query = defineModel<string>("query", { required: true });
const emit = defineEmits<{ timeUpdate: [ms: number] }>();
const { t } = useI18n();
const mediaElement = useTemplateRef<HTMLMediaElement>("mediaElement");
let seekGeneration = 0;

const mediaSrc = computed(() => versionedMediaSource(props.media.assetPath, props.media.sourceRevision));
const posterSrc = computed(() =>
  props.media.kind === "video"
    ? versionedMediaSource(props.media.previewPath, props.media.sourceRevision)
    : undefined,
);

watch(mediaSrc, () => {
  seekGeneration += 1;
});
onBeforeUnmount(() => {
  seekGeneration += 1;
});

async function seekTo(startMs: number, autoplay = true) {
  const generation = ++seekGeneration;
  const element = mediaElement.value;
  if (!element) return;
  if (element.readyState < HTMLMediaElement.HAVE_METADATA) {
    try {
      await waitForMediaMetadata(element, "Media metadata could not be loaded");
    } catch {
      return;
    }
    if (generation !== seekGeneration || mediaElement.value !== element) return;
  }
  const duration = Number.isFinite(element.duration) ? element.duration : Number.POSITIVE_INFINITY;
  element.currentTime = Math.min(Math.max(startMs / 1_000, 0), duration);
  if (autoplay) void element.play().catch(() => {});
}
function togglePlayback() {
  const player = mediaElement.value;
  if (!player) return;
  if (player.paused) void player.play().catch(() => {});
  else player.pause();
}
function onTimeUpdate(event: Event) {
  const target = event.target as HTMLMediaElement | null;
  if (target) emit("timeUpdate", Math.round(target.currentTime * 1_000));
}

defineExpose({ seekTo, togglePlayback });
</script>

<template>
  <div
    data-reader-ignore-selection
    class="shrink-0 border-b border-stroke-divider/70 bg-surface-panel p-3 select-none"
  >
    <video
      v-if="media.kind === 'video'"
      ref="mediaElement"
      :src="mediaSrc"
      :poster="posterSrc"
      controls
      preload="metadata"
      class="block aspect-video max-h-56 w-full rounded-lg bg-surface-stage object-contain"
      @timeupdate="onTimeUpdate"
    />
    <audio
      v-else
      ref="mediaElement"
      :src="mediaSrc"
      controls
      preload="metadata"
      class="block w-full"
      @timeupdate="onTimeUpdate"
    />
    <UiSearchInput
      v-if="hasSegments"
      v-model="query"
      size="sm"
      class="mt-2.5 w-full"
      :placeholder="t('spaces.transcript.text.search')"
    />
  </div>
</template>
