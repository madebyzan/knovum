import { cloneDeep } from "es-toolkit";
import { shallowRef, watch, type Ref } from "vue";

import { i18n } from "../../../../foundation/i18n/i18n";
import type { SpaceVideoPayload } from "../../model/nodes/node-payload";
import { createVideoEditProject, type VideoEditProject } from "../../video-editing/project";

export function useVideoEditProject(
  nodeId: Ref<string | undefined>,
  data: Ref<SpaceVideoPayload | undefined>,
  proxyPath: Ref<string | undefined>,
) {
  const project = shallowRef<VideoEditProject>();
  const metadataError = shallowRef<string>();

  watch(
    [nodeId, () => data.value?.sourceRevision],
    () => {
      const saved = data.value?.videoEditProject;
      project.value = saved ? cloneDeep(saved) : undefined;
      metadataError.value = undefined;
    },
    { immediate: true },
  );

  function initializeFromMetadata(event: Event) {
    if (project.value || !data.value) return;
    const element = event.currentTarget as HTMLVideoElement;
    if (!Number.isFinite(element.duration) || element.duration <= 0) {
      if (proxyPath.value)
        metadataError.value = i18n.global.t("spaces.media.videoEditor.preview.durationUnavailable");
      return;
    }
    if (!validVideoDimensions(element)) {
      metadataError.value = i18n.global.t("spaces.media.videoEditor.preview.metadataUnavailable");
      return;
    }
    project.value = createVideoEditProject(
      data.value.sourceRevision ?? data.value.assetPath,
      Math.max(1, Math.round(element.duration * 1000)),
      element.videoWidth,
      element.videoHeight,
    );
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
