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
      project.value = saved ? structuredClone(saved) : undefined;
      metadataError.value = undefined;
    },
    { immediate: true },
  );

  function initializeFromMetadata(event: Event) {
    if (project.value || !data.value) return;
    const element = event.currentTarget as HTMLVideoElement;
    if (!Number.isFinite(element.duration) || element.duration <= 0) {
      if (proxyPath.value) metadataError.value = i18n.global.t("spaces.media.durationUnavailable");
      return;
    }
    if (
      !element.videoWidth ||
      !element.videoHeight ||
      element.videoWidth > 16_384 ||
      element.videoHeight > 16_384 ||
      element.videoWidth > element.videoHeight * 10 ||
      element.videoHeight > element.videoWidth * 10
    ) {
      metadataError.value = i18n.global.t("spaces.media.videoMetadataUnavailable");
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
