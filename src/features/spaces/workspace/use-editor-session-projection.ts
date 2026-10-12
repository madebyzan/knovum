import { watch } from "vue";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { i18n } from "../../../foundation/i18n/instance";
import { useDocumentSessionStore } from "../document-editing/document-session-store";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceState } from "../session/session-state";
import { useVideoSessionStore } from "../video-editing/video-session-store";
import type { SpaceEditorTab } from "./workbench-tab";

export function useEditorSessionProjection(state: SpaceState, closeTab: (tabId: string) => Promise<void>) {
  const documents = useDocumentSessionStore();
  const videos = useVideoSessionStore();
  function synchronize(tab: SpaceEditorTab, node: SpaceCanvasNode | undefined) {
    const session = tab.kind === "document" ? documents.sessions.get(tab.id) : videos.sessions.get(tab.id);
    if (!session) throw new Error("Editor session is unavailable");
    const previousError = session.error.value;
    if (tab.kind === "document") documents.synchronize(tab.id, node);
    else videos.synchronize(tab.id, node);
    const dirty =
      tab.kind === "document"
        ? documents.sessions.get(tab.id)!.dirty.value
        : videos.sessions.get(tab.id)!.history.dirty.value;
    const error = session.error.value;
    if (error && error !== previousError) toast.error(i18n.global.t(`spaces.editor.tabs.${error}`));
    if (error === "missing" && !dirty)
      void closeTab(tab.id).catch((caught: unknown) => toast.error(getErrorMessage(caught)));
  }
  const savingStates = () =>
    state.editorTabs.value.map((tab) =>
      tab.kind === "document"
        ? documents.sessions.get(tab.id)!.saving.value
        : videos.sessions.get(tab.id)!.saving.value,
    );
  watch([state.nodes, state.activeSpaceId, savingStates], ([nodes, spaceId]) => {
    for (const tab of state.editorTabs.value) {
      if (tab.spaceId === spaceId)
        synchronize(
          tab,
          nodes.find(({ id }) => id === tab.nodeId),
        );
    }
  });
}
