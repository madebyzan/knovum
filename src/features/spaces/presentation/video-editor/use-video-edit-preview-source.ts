import { shallowRef, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import type { SpaceVideoPayload } from "../../model/nodes/node-payload";

interface VideoEditPreviewSourceOptions {
  data: Ref<SpaceVideoPayload | undefined>;
  proxyPath: Ref<string | undefined>;
  playing: Ref<boolean>;
  metadataError: Ref<string | undefined>;
}

export function useVideoEditPreviewSource(options: VideoEditPreviewSourceOptions) {
  const { t } = useI18n();
  const { data, proxyPath, playing, metadataError } = options;
  const previewPath = shallowRef<string>();

  watch(
    [() => data.value?.assetPath, () => data.value?.sourceRevision],
    ([path]) => {
      previewPath.value = path;
      metadataError.value = undefined;
    },
    { immediate: true },
  );
  watch([proxyPath, playing], ([prepared, isPlaying]) => {
    if (prepared && !isPlaying) previewPath.value = prepared;
  });

  function onMetadataError() {
    if (!proxyPath.value) return;
    if (previewPath.value !== proxyPath.value) {
      previewPath.value = proxyPath.value;
      return;
    }
    metadataError.value = t("spaces.media.videoEditor.preview.unavailable");
  }

  return { previewPath, onMetadataError };
}
