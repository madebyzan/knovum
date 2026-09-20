import type { SpaceEditorDocument } from "../contract/canvas-document";
import { canvasStateRepository } from "./canvas-state-repository";

interface CanvasPersistenceLease {
  wait(): Promise<void>;
  release(flush: Promise<unknown>): void;
}

const spaceTails = new Map<string, Promise<void>>();
const activeMutationTargets = new Map<string, CanvasMutationTarget>();

export type CanvasDocumentMutation = (document: SpaceEditorDocument) => SpaceEditorDocument;
type CanvasMutationTarget = (mutation: CanvasDocumentMutation) => Promise<void>;

export function registerCanvasMutationTarget(spaceId: string, target: CanvasMutationTarget) {
  if (activeMutationTargets.has(spaceId)) {
    throw new Error(`Canvas mutation target is already registered for Space ${spaceId}`);
  }
  activeMutationTargets.set(spaceId, target);
  return () => {
    if (activeMutationTargets.get(spaceId) === target) activeMutationTargets.delete(spaceId);
  };
}

export async function mutateSpaceCanvasDocument(spaceId: string, mutation: CanvasDocumentMutation) {
  const activeTarget = activeMutationTargets.get(spaceId);
  if (activeTarget) return activeTarget(mutation);

  const lease = acquireCanvasPersistenceLease(spaceId);
  await lease.wait();
  const operation = persistMutation(spaceId, mutation);
  lease.release(operation);
  await operation;
}

export function acquireCanvasPersistenceLease(spaceId: string): CanvasPersistenceLease {
  const predecessor = spaceTails.get(spaceId) ?? Promise.resolve();
  let finish: (() => void) | undefined;
  const released = new Promise<void>((resolve) => {
    finish = resolve;
  });
  const tail = predecessor.then(() => released);
  spaceTails.set(spaceId, tail);
  let active = true;

  return {
    wait: () => predecessor,
    release(flush) {
      if (!active) return;
      active = false;
      void flush.finally(() => {
        finish?.();
        if (spaceTails.get(spaceId) === tail) spaceTails.delete(spaceId);
      });
    },
  };
}

async function persistMutation(spaceId: string, mutation: CanvasDocumentMutation) {
  const state = await canvasStateRepository.load(spaceId);
  if (!state) throw new Error(`Canvas state does not exist for Space ${spaceId}`);
  const document = mutation(state.document);
  if (document === state.document) return;
  await canvasStateRepository.save({ spaceId, baseRevision: state.revision, document });
}
