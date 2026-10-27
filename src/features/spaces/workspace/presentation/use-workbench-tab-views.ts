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
  readonly dirty: boolean;
}

/** Projects the workbench tab list into the titles, markers and affordances the strip renders. */
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

  /**
   * The strip marks only work that waits on the person: a video edit is kept or discarded by an
   * explicit save. A document saves itself, and its header already reports where that save is, so
   * marking its tab would repeat a transient state on a second surface.
   */
  function dirty(tab: SpaceWorkbenchTab) {
    if (!isSpaceEditorTab(tab) || tab.kind !== "video") return false;
    return Boolean(videos.sessions.get(tab.id)?.history.dirty.value);
  }

  return computed<readonly WorkbenchTabView[]>(() =>
    store.tabs.map((tab) => ({
      id: tab.id,
      kind: tab.kind,
      spaceId: tab.spaceId,
      title: isSpaceEditorTab(tab) ? editorTitle(tab) : spaceTitle(tab.spaceId),
      active: store.activeTabId === tab.id,
      canClose: tab.id !== store.permanentTabId,
      dirty: dirty(tab),
    })),
  );
}
