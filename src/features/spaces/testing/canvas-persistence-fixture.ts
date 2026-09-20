import type { CanvasPersistencePort, StoredCanvasState } from "../canvas/contract/canvas-persistence";

// Model both aggregate existence and compare-and-swap; unconditional successful saves hide this bug.
export function createCanvasPersistenceFixture() {
  const spaces = new Set<string>();
  const states = new Map<string, StoredCanvasState>();
  const persistence: CanvasPersistencePort = {
    async load(spaceId) {
      return states.get(spaceId) ?? null;
    },
    async save({ spaceId, baseRevision, document }) {
      if (!spaces.has(spaceId)) throw new Error(`Cannot save canvas state for missing Space ${spaceId}`);
      const revision = states.get(spaceId)?.revision ?? 0;
      if (baseRevision !== revision) throw new Error("Canvas state revision conflict");
      states.set(spaceId, { revision: revision + 1, document });
      return revision + 1;
    },
  };
  return { spaces, states, persistence };
}
