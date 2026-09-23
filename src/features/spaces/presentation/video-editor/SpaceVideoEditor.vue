<script setup lang="ts">
import { Plus, Save, Scissors, Sparkles, Trash2, X } from "@lucide/vue";
import { computed, onBeforeUnmount, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import { formatDurationMs } from "../../media/format-duration";
import type { SpaceVideoPayload } from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";
import {
  addTextOverlay,
  createVideoEditProject,
  removeClip,
  splitClip,
  type VideoEditProject,
} from "../../video-editing/project";
import { renderVideoEdit } from "../../video-editing/render";
import SpaceMediaControls from "../media/SpaceMediaControls.vue";
import SpaceVideoInspector from "./SpaceVideoInspector.vue";
import SpaceVideoPreview from "./SpaceVideoPreview.vue";
import SpaceVideoTimeline from "./SpaceVideoTimeline.vue";
import { useVideoProjectPlayback } from "./use-video-project-playback";

const store = useSpaceStore();
const { t } = useI18n();
const node = computed(() => store.nodes.find(({ id }) => id === store.videoEditorNodeId));
const data = computed(() => (node.value?.data.kind === "video" ? node.value.data : undefined));
const project = shallowRef<VideoEditProject>();
const video = shallowRef<HTMLVideoElement | null>(null);
const selectedClipId = shallowRef<string>();
const selectedTextId = shallowRef<string>();
const playback = useVideoProjectPlayback(
  video,
  computed(() => project.value!),
);
let observedVideo: HTMLVideoElement | null = null;

watch(
  data,
  (value) => {
    project.value = value?.videoEditProject ? structuredClone(value.videoEditProject) : undefined;
    selectedClipId.value = project.value?.videoTrack.clips[0]?.id;
  },
  { immediate: true },
);
watch(video, (element) => {
  observedVideo?.removeEventListener("loadedmetadata", initializeProject);
  observedVideo = element;
  observedVideo?.addEventListener("loadedmetadata", initializeProject);
});
onBeforeUnmount(() => observedVideo?.removeEventListener("loadedmetadata", initializeProject));

function initializeProject() {
  if (project.value || !data.value || !video.value) return;
  const durationMs = Math.max(1, Math.round(video.value.duration * 1000));
  project.value = createVideoEditProject(data.value.sourceRevision ?? data.value.assetPath, durationMs);
  selectedClipId.value = project.value.videoTrack.clips[0]?.id;
}

function selectClip(id: string) {
  selectedClipId.value = id;
  selectedTextId.value = undefined;
}

function selectText(id: string) {
  selectedTextId.value = id;
  selectedClipId.value = undefined;
}

function removeSelection() {
  if (!project.value) return;
  if (selectedClipId.value) project.value = removeClip(project.value, selectedClipId.value);
  if (selectedTextId.value) {
    project.value = {
      ...project.value,
      textTrack: { items: project.value.textTrack.items.filter(({ id }) => id !== selectedTextId.value) },
    };
  }
  selectedClipId.value = project.value.videoTrack.clips[0]?.id;
  selectedTextId.value = undefined;
}

async function save(announce = true) {
  if (!node.value || !data.value || !project.value) return;
  const after: SpaceVideoPayload = { ...data.value, videoEditProject: project.value };
  await store.updateManyData([{ id: node.value.id, before: data.value, after }]);
  if (announce) toast.success(t("spaces.media.projectSaved"));
}

async function render() {
  if (!node.value || node.value.data.kind !== "video" || !project.value) return;
  await save(false);
  await renderVideoEdit(store, { ...node.value, data: node.value.data }, project.value);
}
</script>

<template>
  <section v-if="data" class="absolute inset-0 z-60 flex min-h-0 flex-col bg-canvas">
    <header class="flex h-11 shrink-0 items-center gap-2 border-b border-border bg-panel px-3">
      <UiIconButton :label="t('common.close')" size="sm" @click="store.closeVideoEditor"
        ><X class="size-(--knovum-icon-size-sm)"
      /></UiIconButton>
      <div class="min-w-0 flex-1">
        <h1 class="truncate text-label font-semibold">{{ data.title }}</h1>
        <p class="text-micro text-foreground-tertiary">{{ t("spaces.media.videoEditor") }}</p>
      </div>
      <UiButton size="sm" :disabled="!project" @click="save"
        ><Save class="size-(--knovum-icon-size-sm)" />{{ t("common.save") }}</UiButton
      >
      <UiButton size="sm" variant="primary" :disabled="!project" @click="render"
        ><Sparkles class="size-(--knovum-icon-size-sm)" />{{ t("spaces.media.renderVideo") }}</UiButton
      >
    </header>
    <div v-if="project" class="flex min-h-0 flex-1">
      <div class="flex min-w-0 flex-1 flex-col">
        <SpaceVideoPreview
          v-model:element="video"
          :data="data"
          :project="project"
          :timeline-ms="playback.timelineMs.value"
          @playing="playback.playing.value = $event"
        />
        <div class="flex h-11 shrink-0 items-center gap-1 border-t border-border bg-panel px-3">
          <UiButton size="sm" variant="ghost" @click="project = splitClip(project, playback.timelineMs.value)"
            ><Scissors class="size-(--knovum-icon-size-sm)" />{{ t("spaces.media.split") }}</UiButton
          >
          <UiButton
            size="sm"
            variant="ghost"
            @click="project = addTextOverlay(project, playback.timelineMs.value)"
            ><Plus class="size-(--knovum-icon-size-sm)" />{{ t("spaces.media.addText") }}</UiButton
          >
          <UiButton
            size="sm"
            variant="ghost"
            :disabled="!selectedClipId && !selectedTextId"
            @click="removeSelection"
            ><Trash2 class="size-(--knovum-icon-size-sm)" />{{ t("common.delete") }}</UiButton
          >
          <div class="mx-auto flex w-96 max-w-1/2 items-center gap-3">
            <SpaceMediaControls
              :playing="playback.playing.value"
              :current-ms="playback.timelineMs.value"
              :duration-ms="playback.durationMs.value"
              @toggle="playback.toggle"
              @seek="playback.seek"
            />
          </div>
          <span class="text-caption text-foreground-secondary tabular-nums"
            >{{ formatDurationMs(playback.timelineMs.value) }} /
            {{ formatDurationMs(playback.durationMs.value) }}</span
          >
        </div>
        <SpaceVideoTimeline
          :project="project"
          :playhead-ms="playback.timelineMs.value"
          :selected-clip-id="selectedClipId"
          :selected-text-id="selectedTextId"
          @seek="playback.seek"
          @select-clip="selectClip"
          @select-text="selectText"
        />
      </div>
      <SpaceVideoInspector
        v-model="project"
        :selected-clip-id="selectedClipId"
        :selected-text-id="selectedTextId"
      />
    </div>
    <div v-else class="grid min-h-0 flex-1 place-items-center text-body text-foreground-secondary">
      {{ t("spaces.media.loadingEditor") }}
    </div>
  </section>
</template>
