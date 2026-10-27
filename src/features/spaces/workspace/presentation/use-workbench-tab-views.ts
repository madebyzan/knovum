import { computed } from "vue";
import { useI18n } from "vue-i18n";

import { useDocumentSessionStore } from "../../document-editing/document-session-store";
import { useSpaceStore } from "../../session/space-store";
import { useVideoSessionStore } from "../../video-editing/video-session-store";
import { isSpaceEditorTab, type SpaceEditorTab, type SpaceWorkbenchTab } from "../workbench-tab";

export interface WorkbenchTabView {
  readonly id: string;
  readonly kind: SpaceWorkbenchTab["kind"];
  readonly spaceId: string;
  readonly title: string;
  readonly active: boolean;
  readonly canClose: boolean;
}

/**
 * Projects the workbench tab list into the titles and affordances the strip renders. A tab carries no
 * save state: every editor stores its work by itself and reports that in its own header.
 */
export function useWorkbenchTabViews() {
  const store = useSpaceStore();
  const documents = useDocumentSessionStore();
  const videos = useVideoSessionStore();
  const { t } = useI18n();

  function editorTitle(tab: SpaceEditorTab) {
    const document = documents.sessions.get(tab.id);
    if (tab.kind === "document") return document?.state.title.value || document?.node.value.data.title || "";
    return videos.sessions.get(tab.id)?.node.value.data.title ?? "";
  }

  function spaceTitle(spaceId: string) {
    const draft = store.draftSpace;
    if (draft?.id === spaceId) return draft.title;
    const space = store.spaces.find((candidate) => candidate.id === spaceId);
    if (!space) throw new Error(`Workbench tab references an unavailable space: ${spaceId}`);
    return space.kind === "inbox" ? t("spaces.space.inbox") : space.title;
  }

  return computed<readonly WorkbenchTabView[]>(() =>
    store.tabs.map((tab) => ({
      id: tab.id,
      kind: tab.kind,
      spaceId: tab.spaceId,
      title: isSpaceEditorTab(tab) ? editorTitle(tab) : spaceTitle(tab.spaceId),
      active: store.activeTabId === tab.id,
      canClose: tab.id !== store.permanentTabId,
    })),
  );
}
