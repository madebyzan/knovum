import * as log from "@tauri-apps/plugin-log";
import { shallowRef, watch, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { z } from "zod";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import type { PrepareVideoEditProxyRequest } from "../contracts/generated/PrepareVideoEditProxyRequest";

const proxyPathSchema = z.string().min(1);

export function useVideoEditProxy(
  nodeId: Ref<string | undefined>,
  sourceRevision: Ref<string | undefined>,
  spaceId: Ref<string | undefined>,
) {
  const { t } = useI18n();
  const path = shallowRef<string>();
  const error = shallowRef<string>();
  const attempt = shallowRef(0);

  watch(
    [nodeId, sourceRevision, spaceId, attempt],
    async ([sourceNodeId, revision, currentSpaceId], _previous, onCleanup) => {
      path.value = undefined;
      error.value = undefined;
      if (!sourceNodeId || !revision || !currentSpaceId) {
        error.value = t("spaces.media.videoEditor.preview.sourceUnavailable");
        return;
      }
      const jobId = crypto.randomUUID();
      let active = true;
      let pending = true;
      onCleanup(() => {
        active = false;
        if (pending) {
          void invokeCommand<void>("plugin:spaces|cancel_video_edit_proxy", { jobId }).catch((cause) =>
            log.error(`Failed to cancel video editing proxy preparation: ${getErrorMessage(cause)}`),
          );
        }
      });
      try {
        const prepared = await invokeCommand<string>("plugin:spaces|prepare_video_edit_proxy", {
          request: {
            id: jobId,
            spaceId: currentSpaceId,
            sourceNodeId,
            sourceRevision: revision,
          } satisfies PrepareVideoEditProxyRequest,
        });
        const proxyPath = proxyPathSchema.parse(prepared);
        if (active) path.value = proxyPath;
      } catch (caught) {
        if (active)
          error.value = `${t("spaces.media.videoEditor.preview.proxyFailed")}: ${getErrorMessage(caught)}`;
      } finally {
        pending = false;
      }
    },
    { immediate: true },
  );

  return { path, error, retry: () => (attempt.value += 1) };
}
