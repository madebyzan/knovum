import { computed, nextTick, type Ref } from "vue";

import { clipTimelineStart, type VideoEditProject } from "../../video-editing/project";
import type { useVideoEditSelection } from "./use-video-edit-selection";
import type { useVideoProjectPlayback } from "./use-video-project-playback";

interface VideoEditorInspectorOptions {
  project: Ref<VideoEditProject | undefined>;
  video: Ref<HTMLVideoElement | null>;
  editorRoot: Ref<HTMLElement | null>;
  selection: ReturnType<typeof useVideoEditSelection>;
  playback: ReturnType<typeof useVideoProjectPlayback>;
}

export function useVideoEditorInspector(options: VideoEditorInspectorOptions) {
  const { project, video, editorRoot, selection, playback } = options;
  const visible = computed(() => !!selection.selectedClipId.value || !!selection.selectedTextId.value);

  function selectText(textId: string) {
    const item = project.value?.textTrack.items.find((entry) => entry.id === textId);
    if (!item) return;
    video.value?.pause();
    selection.selectText(textId);
    if (playback.timelineMs.value < item.startMs || playback.timelineMs.value >= item.endMs)
      playback.seek(item.startMs);
  }

  function selectClip(clipId: string) {
    if (!project.value) return;
    video.value?.pause();
    selection.selectClip(clipId);
    playback.seek(clipTimelineStart(project.value, clipId));
  }

  async function close() {
    selection.clear();
    await nextTick();
    editorRoot.value?.querySelector<HTMLElement>("[data-video-timeline-seek]")?.focus();
  }

  return { visible, selectText, selectClip, close };
}
