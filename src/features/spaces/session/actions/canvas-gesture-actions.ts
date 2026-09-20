import type {
  CanvasGestureCommitRequest,
  CanvasPersistenceMutationRequest,
} from "../../canvas/contract/canvas-editor";
import { commitCanvasGesture } from "../../canvas/persistence/canvas-gesture-repository";
import type { SpaceState } from "../session-state";
export function createCanvasGestureActions(state: SpaceState) {
  async function commitGesture(request: CanvasGestureCommitRequest) {
    const spaceId = state.activeSpaceId.value;
    if (!spaceId) throw new Error("An active Space is required for editing");
    const persist = (input: CanvasPersistenceMutationRequest) => commitCanvasGesture(input, spaceId);
    const result = await persist(request);
    return { ...result, document: request.document, history: { undo: persist, redo: persist } };
  }
  return { commitGesture };
}
