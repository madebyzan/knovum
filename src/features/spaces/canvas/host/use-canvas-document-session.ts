import { shallowRef } from "vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { emptySpaceEditorDocument, type SpaceEditorDocument } from "../contract/canvas-document";
import { registerSpaceSessionParticipant } from "../editor-session";
import {
  acquireCanvasPersistenceLease,
  registerCanvasMutationTarget,
  registerCanvasTransactionTarget,
} from "../persistence/canvas-persistence-coordinator";
import { canvasStateRepository } from "../persistence/canvas-state-repository";
import { CanvasSnapshotSaveQueue } from "../persistence/snapshot-save-queue";
import { CanvasDocumentWorkspace } from "./canvas-document-workspace";
interface Options {
  nodes(): readonly SpaceCanvasNode[];
  reportError(error: unknown): void;
  spaceId(): string;
}
export function useCanvasDocumentSession(options: Options) {
  const document = shallowRef<SpaceEditorDocument>();
  const surfaceKey = shallowRef(0);
  let workspace: CanvasDocumentWorkspace | undefined;
  let lease: ReturnType<typeof acquireCanvasPersistenceLease> | undefined;
  let detachTargets: (() => void)[] = [];
  let generation = 0;
  const detachSession = registerSpaceSessionParticipant({
    phase: "persistence",
    commit: async () => {
      await workspace?.flush();
    },
  });
  function commitTransaction<T extends { document: SpaceEditorDocument; revision: number }>(
    commit: (baseRevision: number) => Promise<T>,
  ) {
    if (!workspace) return Promise.reject(new Error("The Space editor is not ready"));
    return workspace.commit(commit);
  }
  async function start() {
    const expected = ++generation,
      spaceId = options.spaceId();
    const old = workspace;
    detachTargets.forEach((detach) => detach());
    detachTargets = [];
    lease?.release(release(old));
    workspace = undefined;
    document.value = undefined;
    lease = acquireCanvasPersistenceLease(spaceId);
    await lease.wait();
    if (generation !== expected) return;
    const stored = await canvasStateRepository.load(spaceId);
    if (generation !== expected) return;
    const initial = stored?.document ?? emptySpaceEditorDocument();
    const queue = new CanvasSnapshotSaveQueue({
      spaceId,
      initialState: stored,
      persistence: canvasStateRepository,
      reportError: options.reportError,
    });
    const next: CanvasDocumentWorkspace = new CanvasDocumentWorkspace(initial, queue, {
      isActive: (): boolean => generation === expected && workspace === next,
      publish: (value) => {
        document.value = value;
      },
    });
    workspace = next;
    document.value = initial;
    detachTargets = [
      registerCanvasTransactionTarget(spaceId, (commit) =>
        next.commit((revision) => commit(next.document, revision)),
      ),
      registerCanvasMutationTarget(spaceId, async (mutation) => {
        await next.commit(async (baseRevision) => {
          const updated = mutation(next.document);
          return {
            document: updated,
            revision: await canvasStateRepository.save({ spaceId, baseRevision, document: updated }),
          };
        });
      }),
    ];
    surfaceKey.value += 1;
  }
  async function release(previous: CanvasDocumentWorkspace | undefined) {
    try {
      await previous?.flush();
    } catch (error) {
      options.reportError(error);
    }
  }
  function dispose() {
    generation += 1;
    detachTargets.forEach((detach) => detach());
    detachTargets = [];
    lease?.release(release(workspace));
    lease = undefined;
    workspace = undefined;
    detachSession();
  }
  return {
    document,
    surfaceKey,
    start,
    dispose,
    commitTransaction,
    documentChanged: (next: SpaceEditorDocument) => workspace?.change(next),
    synchronizeNodes: (_nodes: readonly SpaceCanvasNode[]) => undefined,
  };
}
