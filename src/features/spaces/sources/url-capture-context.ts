import { useSpaceStore } from "../session/space-store";
import type { UrlCaptureContext } from "./capture-url";

export function urlCaptureContext(): UrlCaptureContext {
  const store = useSpaceStore();

  return {
    getActiveSpaceId: () => store.activeSpaceId,
    get nodes() {
      return store.nodes;
    },
    createPendingContentForSpace: store.createPendingContentForSpace,
    savePendingNodeContentForSpace: store.savePendingNodeContentForSpace,
    finalizePendingContent: store.finalizePendingContent,
    discardPendingContent: store.discardPendingContent,
    addContent: store.addContent,
    saveNodeContentForSpace: store.saveNodeContentForSpace,
  };
}
