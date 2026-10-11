import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { applySpaceNodeContent } from "../canvas/node";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceVideoPayload } from "../model/nodes/node-payload";
import { useSpaceStore } from "../session/space-store";
import type { VideoSession } from "./video-session-store";

export function useVideoProjectSave(session: VideoSession) {
  const store = useSpaceStore();
  const { t } = useI18n();
  let pending: Promise<boolean> | undefined;
  function save(): Promise<boolean> {
    if (pending) return pending;
    const node = session.node.value;
    const snapshot = session.project.value;
    if (!snapshot || session.error.value) return Promise.resolve(false);
    session.saving.value = true;
    pending = store
      .saveNodeContentForSpace(session.tab.spaceId, {
        id: node.id,
        expectedContent: node.data,
        data: { ...node.data, videoEditProject: snapshot },
      })
      .then((payload) => {
        session.node.value = applySpaceNodeContent(node, payload) as SpaceCanvasNode<SpaceVideoPayload>;
        session.history.markSaved(snapshot);
        toast.success(t("spaces.media.videoEditor.project.saved"));
        return true;
      })
      .catch((error: unknown) => {
        toast.error(getErrorMessage(error));
        return false;
      })
      .finally(() => {
        session.saving.value = false;
        pending = undefined;
      });
    return pending;
  }
  return save;
}
