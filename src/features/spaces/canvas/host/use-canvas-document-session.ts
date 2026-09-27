import { useDebounceFn } from "@vueuse/core";
import { shallowRef } from "vue";

import type { SpaceEditSnapshot } from "../../model/structure/schema/edit-snapshot";
import { projectStructureCanvasDocument } from "../document/structure-canvas-document";
import { registerSpaceSessionParticipant } from "../editor-session";
import { canvasViewStateOperations } from "../persistence/canvas-view-state";
import type { CanvasDocumentV1 } from "../schema/document";

interface CanvasDocumentSessionOptions {
  snapshot(): SpaceEditSnapshot;
  reportError(error: unknown): void;
  spaceId(): string;
  persistence(): "draft" | "persisted";
  ensurePersisted(): Promise<void>;
}

/** Keeps only an ephemeral Konva adapter document. Structure/Layout and camera persist separately. */
export function useCanvasDocumentSession(options: CanvasDocumentSessionOptions) {
  const document = shallowRef<CanvasDocumentV1>();
  const surfaceKey = shallowRef(0);
  let pendingView:
    | { spaceId: string; viewport: CanvasDocumentV1["viewport"]; persisted: boolean }
    | undefined;
  let viewSaveTail = Promise.resolve();
  let generation = 0;
  const scheduleViewSave = useDebounceFn(() => void flushView().catch(options.reportError), 250);
  const detachSessionCommit = registerSpaceSessionParticipant({
    commit: flushView,
    phase: "persistence",
  });

  async function start() {
    await flushView();
    const currentGeneration = ++generation;
    const storedView =
      options.persistence() === "persisted" ? await canvasViewStateOperations.load(options.spaceId()) : null;
    if (currentGeneration !== generation) return;
    document.value = projectStructureCanvasDocument(options.snapshot(), storedView ?? undefined);
    surfaceKey.value += 1;
  }

  async function persist() {
    await options.ensurePersisted();
    synchronizeSnapshot();
  }

  function synchronizeSnapshot(snapshot = options.snapshot()) {
    if (!document.value) return;
    document.value = projectStructureCanvasDocument(snapshot, document.value.viewport);
  }

  function documentChanged(next: CanvasDocumentV1) {
    document.value = next;
  }

  function viewportChanged(viewport: CanvasDocumentV1["viewport"]) {
    pendingView = {
      spaceId: options.spaceId(),
      viewport,
      persisted: options.persistence() === "persisted",
    };
    void scheduleViewSave();
  }

  async function flushView() {
    scheduleViewSave.cancel();
    const pending = pendingView;
    pendingView = undefined;
    if (!pending) return viewSaveTail;
    if (!pending.persisted) return viewSaveTail;
    viewSaveTail = viewSaveTail.then(() => canvasViewStateOperations.save(pending.spaceId, pending.viewport));
    return viewSaveTail;
  }

  function dispose() {
    generation += 1;
    detachSessionCommit();
    void flushView().catch(options.reportError);
  }

  return {
    dispose,
    document,
    documentChanged,
    persist,
    start,
    surfaceKey,
    synchronizeSnapshot,
    viewportChanged,
  };
}
