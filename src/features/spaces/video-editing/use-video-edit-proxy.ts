import { shallowRef, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { prepareVideoEditProxy } from "./prepare-video-edit-proxy";

export function useVideoEditProxy(
  nodeId: Ref<string | undefined>,
  sourceRevision: Ref<string | undefined>,
  spaceId: Ref<string | undefined>,
  visible: Ref<boolean>,
) {
  const { t } = useI18n();
  const path = shallowRef<string>();
  const error = shallowRef<string>();
  const attempt = shallowRef(0);
  let preparedIdentity: string | undefined;

  watch(
    [nodeId, sourceRevision, spaceId, attempt, visible],
    async ([sourceNodeId, revision, currentSpaceId, _attempt, visibleNow], _previous, onCleanup) => {
      if (!visibleNow) return;
      const identity = `${currentSpaceId}\0${sourceNodeId}\0${revision}`;
      if (preparedIdentity === identity && path.value) return;
      path.value = undefined;
      error.value = undefined;
      if (!sourceNodeId || !revision || !currentSpaceId) {
        error.value = t("spaces.media.videoEditor.preview.sourceUnavailable");
        return;
      }
      const controller = new AbortController();
      onCleanup(() => controller.abort());
      try {
        const proxyPath = await prepareVideoEditProxy(
          {
            spaceId: currentSpaceId,
            sourceNodeId,
            sourceRevision: revision,
          },
          controller.signal,
        );
        if (!controller.signal.aborted) {
          path.value = proxyPath;
          preparedIdentity = identity;
        }
      } catch (caught) {
        if (!controller.signal.aborted)
          error.value = `${t("spaces.media.videoEditor.preview.proxyFailed")}: ${getErrorMessage(caught)}`;
      }
    },
    { immediate: true },
  );

  return {
    path,
    error,
    retry: () => {
      preparedIdentity = undefined;
      attempt.value += 1;
    },
  };
}
