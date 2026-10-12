import { computed, nextTick, type Ref } from "vue";

import { type VideoEditProject } from "../project";
import { clipTimelineStartMs } from "../timing";
import type { useVideoEditSelection } from "./use-video-edit-selection";
import type { useVideoProjectPlayback } from "./use-video-project-playback";

interface VideoEditorInspectorOptions {
  project: Ref<VideoEditProject | undefined>;
  editorRoot: Ref<HTMLElement | null>;
  selection: ReturnType<typeof useVideoEditSelection>;
  playback: ReturnType<typeof useVideoProjectPlayback>;
}

export function useVideoEditorInspector(options: VideoEditorInspectorOptions) {
  const { project, editorRoot, selection, playback } = options;
  const visible = computed(
    () =>
      !!selection.selectedClipId.value ||
      !!selection.selectedTextId.value ||
      !!selection.selectedAudioId.value,
  );

  function selectText(textId: string) {
    const item = project.value?.textTrack.items.find((entry) => entry.id === textId);
    if (!item) return;
    playback.pause();
    selection.selectText(textId);
    if (playback.timelineMs.value < item.startMs || playback.timelineMs.value >= item.endMs)
      playback.seek(item.startMs);
  }

  function selectClip(clipId: string) {
    if (!project.value) return;
    playback.pause();
    selection.selectClip(clipId);
    playback.seek(clipTimelineStartMs(project.value, clipId));
  }

  function selectAudio(id: string) {
    const clip = project.value?.audioTrack.clips.find((candidate) => candidate.id === id);
    if (!clip) return;
    playback.pause();
    selection.selectAudio(id);
    playback.seek(clip.timelineStartMs);
  }
  async function close() {
    selection.clear();
    await nextTick();
    editorRoot.value?.querySelector<HTMLElement>("[data-video-timeline-seek]")?.focus();
  }

  return { visible, selectText, selectClip, selectAudio, close };
}
