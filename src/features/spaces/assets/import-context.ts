import type { useSpaceStore } from "../session/space-store";
import type { SpaceImportContext } from "./import-session";

export function spaceImportContext(store: ReturnType<typeof useSpaceStore>): SpaceImportContext {
  return {
    ensureImportTarget: async (spaceId) => {
      if (store.activeSpaceId !== spaceId) return;
      await store.ensureActiveSpacePersisted();
    },
    createPendingContentForSpace: store.createPendingContentForSpace,
    savePendingNodeDataForSpace: store.savePendingNodeDataForSpace,
    finalizePendingContent: store.finalizePendingContent,
    applyNodeDataForSpace: store.applyNodeDataForSpace,
    discardPendingContent: store.discardPendingContent,
  };
}
