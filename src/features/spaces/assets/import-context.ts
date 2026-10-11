import { useSpaceStore } from "../session/space-store";
import type { SpaceImportContext } from "./import-contract";

export function spaceImportContext(): SpaceImportContext {
  const store = useSpaceStore();

  return {
    ensureImportTarget: async (spaceId) => {
      if (store.activeSpaceId !== spaceId) return;
      await store.ensureActiveSpacePersisted();
    },
    createPendingContentForSpace: store.createPendingContentForSpace,
    savePendingNodeContentForSpace: store.savePendingNodeContentForSpace,
    finalizePendingContent: store.finalizePendingContent,
    applyNodeContentForSpace: store.applyNodeContentForSpace,
    discardPendingContent: store.discardPendingContent,
  };
}
