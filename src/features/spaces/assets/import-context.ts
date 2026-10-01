import type { useSpaceStore } from "../session/space-store";
import type { SpaceImportContext } from "./import-contract";

export type SpaceImportStore = Pick<
  ReturnType<typeof useSpaceStore>,
  | "activeSpaceId"
  | "ensureActiveSpacePersisted"
  | "createPendingContentForSpace"
  | "savePendingNodeContentForSpace"
  | "finalizePendingContent"
  | "applyNodeContentForSpace"
  | "discardPendingContent"
>;

export function spaceImportContext(store: SpaceImportStore): SpaceImportContext {
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
