import type {
  CanvasGestureCommitRequest,
  CanvasPersistenceMutationRequest,
} from "./contract/canvas-editor";
import { commitCanvasGesture } from "./persistence/canvas-gesture-repository";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";
import { omit } from "../organization/selection-mutations";
import type { SpaceState } from "../session/session-state";

export function createCanvasGestureActions(state: SpaceState) {
  async function commitGesture(request: CanvasGestureCommitRequest) {
    const spaceId = requireSpaceId(state.activeSpaceId.value);
    const assignments = new Map(request.assignments.map(({ id, sectionId }) => [id, sectionId]));
    const changes = state.nodes.value.flatMap((node) => {
      if (!assignments.has(node.id) || node.data.kind === "section") return [];
      const sectionId = assignments.get(node.id);
      const before = validateSpaceNodePayload(node.data);
      const after = sectionId ? { ...before, sectionId } : omit(before, "sectionId");
      return before.sectionId === after.sectionId ? [] : [{ id: node.id, before, after }];
    });
    const result = await persist(request, changes, "after");
    return {
      ...result,
      document: request.document,
      history: {
        undo: (input: CanvasPersistenceMutationRequest) => persist(input, changes, "before"),
        redo: (input: CanvasPersistenceMutationRequest) => persist(input, changes, "after"),
      },
    };

    async function persist(
      input: CanvasPersistenceMutationRequest,
      entries: typeof changes,
      side: "before" | "after",
    ) {
      const persisted = await commitCanvasGesture(
        input,
        spaceId,
        entries.map(({ id, [side]: data }) => ({ id, data })),
      );
      if (state.activeSpaceId.value !== spaceId) throw new Error("Canvas gesture target changed");
      const payloads = new Map(entries.map(({ id, [side]: data }) => [id, data]));
      state.nodes.value = state.nodes.value.map((node) => {
        const data = payloads.get(node.id);
        return data ? { ...node, data } : node;
      });
      return persisted;
    }
  }

  return { commitGesture };
}

function requireSpaceId(spaceId: string | undefined) {
  if (!spaceId) throw new Error("A Space must be active to commit a canvas gesture");
  return spaceId;
}
