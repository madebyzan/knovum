import { shallowRef } from "vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { CanvasDocumentV1 } from "../contract/canvas-document";
import { createCanvasDocument, reconcileCanvasDocument } from "../document/canvas-document-sync";
import { registerSpaceSessionParticipant } from "../editor-session";
import {
  acquireCanvasPersistenceLease,
  registerCanvasMutationTarget,
} from "../persistence/canvas-persistence-coordinator";
import { canvasStateRepository } from "../persistence/canvas-state-repository";
import { CanvasSnapshotSaveQueue } from "../persistence/snapshot-save-queue";
import { CanvasDocumentWorkspace } from "./canvas-document-workspace";

interface CanvasDocumentSessionOptions {
  nodes(): readonly SpaceCanvasNode[];
  reportError(error: unknown): void;
  spaceId(): string;
}

export function useCanvasDocumentSession(options: CanvasDocumentSessionOptions) {
  const document = shallowRef<CanvasDocumentV1>();
  const surfaceKey = shallowRef(0);
  let workspace: CanvasDocumentWorkspace | undefined;
  let persistenceLease: ReturnType<typeof acquireCanvasPersistenceLease> | undefined;
  let unregisterMutationTarget: (() => void) | undefined;
  let generation = 0;
  const detachSessionCommit = registerSpaceSessionParticipant({
    phase: "persistence",
    commit: async () => {
      try { await workspace?.flush(); }
      catch (error) { options.reportError(error); throw error; }
    },
  });

  function commitTransaction<T extends { document: CanvasDocumentV1; revision: number }>(
    commit: (baseRevision: number) => Promise<T>,
  ): Promise<T> {
    if (!workspace) return Promise.reject(new Error("Canvas persistence is not ready"));
    return workspace.commit(commit);
  }

  async function start() {
    const expectedGeneration = ++generation;
    const spaceId = options.spaceId();
    const previousWorkspace = workspace;
    const previousLease = persistenceLease;
    unregisterMutationTarget?.();
    unregisterMutationTarget = undefined;
    workspace = undefined;
    document.value = undefined;
    previousLease?.release(release(previousWorkspace));
    const lease = acquireCanvasPersistenceLease(spaceId);
    persistenceLease = lease;
    await lease.wait();
    if (!current()) return;
    const stored = await canvasStateRepository.load(spaceId);
    if (!current()) return;
    const initial = stored
      ? reconcileCanvasDocument(stored.document, options.nodes())
      : createCanvasDocument(options.nodes());
    const queue = new CanvasSnapshotSaveQueue({
      spaceId,
      persistence: canvasStateRepository,
      initialState: stored,
      reportError: options.reportError,
    });
    const next = new CanvasDocumentWorkspace(initial, queue, {
      isActive: () => current() && workspace === next,
      publish: (value) => { document.value = value; },
    });
    workspace = next;
    unregisterMutationTarget = registerCanvasMutationTarget(spaceId, async (mutation) => {
      if (!current() || workspace !== next) {
        throw new Error("Canvas mutation target is no longer active");
      }
      next.change(mutation(next.document));
    });
    document.value = initial;
    surfaceKey.value += 1;
    if (!stored || initial !== stored.document) queue.schedule(() => initial);

    function current() {
      return expectedGeneration === generation && options.spaceId() === spaceId;
    }
  }

  function synchronizeNodes(nodes: readonly SpaceCanvasNode[]) {
    if (!workspace) return;
    const next = reconcileCanvasDocument(workspace.document, nodes);
    if (next !== workspace.document) workspace.change(next);
  }

  function documentChanged(next: CanvasDocumentV1) {
    workspace?.change(next);
  }

  async function release(previous: CanvasDocumentWorkspace | undefined) {
    try { await previous?.flush(); }
    catch (error) { options.reportError(error); }
  }

  function dispose() {
    generation += 1;
    unregisterMutationTarget?.();
    unregisterMutationTarget = undefined;
    persistenceLease?.release(release(workspace));
    persistenceLease = undefined;
    workspace = undefined;
    detachSessionCommit();
  }

  return { commitTransaction, dispose, document, documentChanged, start, surfaceKey, synchronizeNodes };
}
