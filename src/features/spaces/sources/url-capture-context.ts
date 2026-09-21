import type { useSpaceStore } from "../session/space-store";
import type { UrlCaptureContext } from "./capture-url";

export function urlCaptureContext(store: ReturnType<typeof useSpaceStore>): UrlCaptureContext {
  return {
    getActiveSpaceId: () => store.activeSpaceId,
    get nodes() {
      return store.nodes;
    },
    beginUrlCapture: store.beginUrlCapture,
    updateUrlCapture: store.updateUrlCapture,
    finishUrlCapture: store.finishUrlCapture,
    addContent: store.addContent,
    saveNodeDataForSpace: store.saveNodeDataForSpace,
  };
}
