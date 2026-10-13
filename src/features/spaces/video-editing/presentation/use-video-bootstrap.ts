import { shallowRef, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import type { VideoMaterialNode } from "../material";
import { createVideoEditProject, type VideoEditProject } from "../project";
import { readVideoMaterial } from "../read-material";

interface BootstrapOptions {
  node: Ref<VideoMaterialNode | undefined>;
  project: Ref<VideoEditProject | undefined>;
  obsolete: Ref<boolean>;
  active: Ref<boolean>;
  onCreated: (project: VideoEditProject) => void;
}

/**
 * Creates the project for a video that has never been edited by measuring the source itself, so the
 * output starts at the video's own resolution and cadence. A stored project from an older contract
 * is not migrated: the editor asks for an explicit reset.
 */
export function useVideoProjectBootstrap(options: BootstrapOptions) {
  const { t } = useI18n();
  const error = shallowRef<string>();
  const preparing = shallowRef(false);
  const attempt = shallowRef(0);

  async function build(node: VideoMaterialNode, signal: AbortSignal) {
    preparing.value = true;
    error.value = undefined;
    try {
      const { asset, frameRate } = await readVideoMaterial(node, signal);
      if (signal.aborted) return;
      if (asset.kind !== "video" || asset.durationMs < 100 || !asset.width || !asset.height) {
        error.value = t("spaces.media.videoEditor.preview.metadataUnavailable");
        return;
      }
      options.onCreated(
        createVideoEditProject({
          nodeId: node.id,
          sourceRevision: asset.sourceRevision,
          durationMs: asset.durationMs,
          sourceWidth: asset.width,
          sourceHeight: asset.height,
          frameRate,
        }),
      );
    } catch (caught) {
      if (!signal.aborted)
        error.value = `${t("spaces.media.videoEditor.preview.proxyFailed")}: ${getErrorMessage(caught)}`;
    } finally {
      preparing.value = false;
    }
  }

  watch(
    [options.node, options.project, options.obsolete, options.active, attempt],
    ([node, project, obsolete, isActive], _previous, onCleanup) => {
      if (!isActive || project || obsolete || !node) return;
      const controller = new AbortController();
      onCleanup(() => controller.abort());
      void build(node, controller.signal);
    },
    { immediate: true },
  );

  return {
    error,
    preparing,
    retry: () => {
      error.value = undefined;
      attempt.value += 1;
    },
    /** Explicit reset of an obsolete stored project. */
    reset() {
      options.obsolete.value = false;
      error.value = undefined;
      attempt.value += 1;
    },
  };
}
