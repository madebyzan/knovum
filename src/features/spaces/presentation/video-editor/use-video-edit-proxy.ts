import { invoke } from "@tauri-apps/api/core";
import { shallowRef, watch, type Ref } from "vue";

import { i18n } from "../../../../foundation/i18n/i18n";

export function useVideoEditProxy(
  nodeId: Ref<string | undefined>,
  sourceRevision: Ref<string | undefined>,
  spaceId: Ref<string | undefined>,
) {
  const path = shallowRef<string>();
  const error = shallowRef<string>();
  const attempt = shallowRef(0);

  watch(
    [nodeId, sourceRevision, spaceId, attempt],
    async ([sourceNodeId, revision, currentSpaceId], _previous, onCleanup) => {
      path.value = undefined;
      error.value = undefined;
      if (!sourceNodeId || !revision || !currentSpaceId) {
        error.value = i18n.global.t("spaces.media.sourceUnavailable");
        return;
      }
      const jobId = crypto.randomUUID();
      let active = true;
      let pending = true;
      onCleanup(() => {
        active = false;
        if (pending) {
          void invoke("plugin:spaces|cancel_video_edit_proxy", { jobId }).catch(() => undefined);
        }
      });
      try {
        const prepared = await invoke<string>("plugin:spaces|prepare_video_edit_proxy", {
          request: {
            id: jobId,
            spaceId: currentSpaceId,
            sourceNodeId,
            sourceRevision: revision,
          },
        });
        if (active) path.value = prepared;
      } catch (caught) {
        if (active) error.value = `${i18n.global.t("spaces.media.proxyFailed")}: ${String(caught)}`;
      } finally {
        pending = false;
      }
    },
    { immediate: true },
  );

  return { path, error, retry: () => (attempt.value += 1) };
}
