import type { useSpaceStore } from "../session/space-store";
import type { UrlCaptureContext } from "./capture-url";

type UrlCaptureStore = Pick<
  ReturnType<typeof useSpaceStore>,
  | "activeSpaceId"
  | "nodes"
  | "beginUrlCapture"
  | "updateUrlCapture"
  | "finishUrlCapture"
  | "addContent"
  | "saveNodeContentForSpace"
>;

export function urlCaptureContext(store: UrlCaptureStore): UrlCaptureContext {
  return {
    getActiveSpaceId: () => store.activeSpaceId,
    get nodes() {
      return store.nodes;
    },
    beginUrlCapture: store.beginUrlCapture,
    updateUrlCapture: store.updateUrlCapture,
    finishUrlCapture: store.finishUrlCapture,
    addContent: store.addContent,
    saveNodeContentForSpace: store.saveNodeContentForSpace,
  };
}
