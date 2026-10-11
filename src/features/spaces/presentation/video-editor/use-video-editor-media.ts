import { shallowRef, watch, type Ref } from "vue";

import type { SpaceVideoPayload } from "../../model/nodes/node-payload";
import type { VideoEditProject } from "../../video-editing/project";
import type { useVideoEditHistory } from "../../video-editing/use-video-edit-history";
import type { useVideoProjectPlayback } from "./use-video-project-playback";

interface VideoEditorMediaOptions {
  data: Ref<SpaceVideoPayload | undefined>;
  project: Ref<VideoEditProject | undefined>;
  video: Ref<HTMLVideoElement | null>;
  history: ReturnType<typeof useVideoEditHistory>;
  playback: ReturnType<typeof useVideoProjectPlayback>;
  initializeFromMetadata(event: Event): void;
}

export function useVideoEditorMedia(options: VideoEditorMediaOptions) {
  const sourceDurationMs = shallowRef(0);

  watch(
    () => options.data.value?.sourceRevision,
    () => {
      sourceDurationMs.value = 0;
    },
  );

  function onMetadataReady(event: Event) {
    options.initializeFromMetadata(event);
    const element = event.currentTarget as HTMLVideoElement;
    sourceDurationMs.value = Number.isFinite(element.duration) ? Math.round(element.duration * 1_000) : 0;
    if (options.project.value) {
      options.history.reset(options.data.value?.videoEditProject ?? options.project.value);
      options.playback.onSourceReady();
    }
  }

  function onPreviewReady() {
    if (options.video.value && Number.isFinite(options.video.value.duration))
      sourceDurationMs.value = Math.round(options.video.value.duration * 1_000);
    options.playback.onSourceReady();
  }

  return { sourceDurationMs, onMetadataReady, onPreviewReady };
}
