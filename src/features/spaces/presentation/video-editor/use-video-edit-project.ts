import { shallowRef, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import type { SpaceVideoPayload } from "../../model/nodes/node-payload";
import { createVideoEditProject, type VideoEditProject } from "../../video-editing/project";

export function useVideoEditProject(
  project: Ref<VideoEditProject | undefined>,
  data: Ref<SpaceVideoPayload | undefined>,
  proxyPath: Ref<string | undefined>,
) {
  const { t } = useI18n();
  const metadataError = shallowRef<string>();

  watch(
    () => data.value?.sourceRevision,
    () => (metadataError.value = undefined),
  );

  function initializeFromMetadata(event: Event) {
    if (project.value || !data.value) return;
    const element = event.currentTarget as HTMLVideoElement;
    if (!Number.isFinite(element.duration) || element.duration <= 0) {
      if (proxyPath.value) metadataError.value = t("spaces.media.videoEditor.preview.durationUnavailable");
      return;
    }
    if (!validVideoDimensions(element)) {
      metadataError.value = t("spaces.media.videoEditor.preview.metadataUnavailable");
      return;
    }
    project.value = createVideoEditProject({
      sourceRevision: data.value.sourceRevision ?? data.value.assetPath,
      durationMs: Math.max(1, Math.round(element.duration * 1000)),
      sourceWidth: element.videoWidth,
      sourceHeight: element.videoHeight,
    });
  }

  return { project, metadataError, initializeFromMetadata };
}

function validVideoDimensions(element: HTMLVideoElement) {
  const { videoWidth: width, videoHeight: height } = element;
  return (
    width > 0 &&
    height > 0 &&
    width <= 16_384 &&
    height <= 16_384 &&
    width <= height * 10 &&
    height <= width * 10
  );
}
