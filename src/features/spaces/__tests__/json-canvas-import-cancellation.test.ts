import { createPinia, setActivePinia } from "pinia";
import { beforeEach, expect, it, vi } from "vitest";

import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { importJsonCanvas } from "../canvas-exchange/json-canvas-import-operations";
import { createJsonCanvasImportActions } from "../session/actions/json-canvas-import-actions";
import { createSpaceState } from "../session/session-state";
import { openTestSpaceTab } from "./support/session-context";
type CanvasImport = typeof importJsonCanvas;
import { deferred } from "../../../shared/__tests__/support/deferred";
import type { loadSpaceSessionProjection } from "../session/space-projection";
type Projection = typeof loadSpaceSessionProjection;
type Receipt = Awaited<ReturnType<CanvasImport>>;
const mocks = vi.hoisted(() => ({
  import: vi.fn<CanvasImport>(),
  cancel: vi.fn<(mutationId: string) => Promise<void>>(),
  projection: vi.fn<Projection>(),
}));
vi.mock("../canvas-exchange/json-canvas-import-operations", () => ({
  importJsonCanvas: mocks.import,
  cancelJsonCanvasImport: mocks.cancel,
}));
vi.mock("../session/space-projection", () => ({ loadSpaceSessionProjection: mocks.projection }));

beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
  mocks.cancel.mockResolvedValue(undefined);
  mocks.projection.mockResolvedValue({
    nodes: [],
    scenes: [],
    relations: [],
    editSnapshot: { editRevision: 1, structure: [], layouts: [], relationStyles: [] },
  });
});

function startImport() {
  const state = createSpaceState();
  openTestSpaceTab(state, "space");
  return createJsonCanvasImportActions({
    state,
    ensureActiveSpacePersisted: async () => {},
    mutations: { commitPendingContent: vi.fn<() => void>(), applyNodeContentForSpace: vi.fn<() => void>() },
    placement: { requestReveal: vi.fn<() => void>() },
  }).importJsonCanvas({ sourcePath: "/canvas.canvas", baseRevision: 0 });
}

/** Starts an import that Rust has registered, so the running task already offers cancellation. */
async function startRegisteredImport() {
  const receipt = deferred<Receipt>();
  mocks.import.mockImplementation(
    (_input, onProgress: (progress: { stage: string; bytesCopied: number }) => void) => {
      onProgress({ stage: "preparing", bytesCopied: 0 });
      return receipt.promise;
    },
  );
  const operation = startImport();
  const tasks = useTaskStore();
  await vi.waitFor(() => expect(tasks.tasks[0]?.cancellable).toBe(true));
  return { receipt, operation, tasks };
}

it("waits for Rust cancellation and cleanup before settling the task", async () => {
  const { receipt, operation, tasks } = await startRegisteredImport();
  const rejection = operation.catch((error: unknown) => error);
  const cancellation = tasks.cancel(tasks.tasks[0]!.id);
  await vi.waitFor(() => expect(mocks.cancel).toHaveBeenCalled());
  expect(tasks.tasks[0]?.status).toBe("running");
  receipt.reject(
    new Error("cancelled", { cause: { code: "json_canvas_import_cancelled", message: "cancelled" } }),
  );
  expect(await rejection).toBeInstanceOf(Error);
  await cancellation;
  expect(tasks.tasks[0]?.status).toBe("cancelled");
  expect(mocks.projection).not.toHaveBeenCalled();
});

it("keeps a committed import successful when cancellation loses the race", async () => {
  const { receipt, operation, tasks } = await startRegisteredImport();
  const cancellation = tasks.cancel(tasks.tasks[0]!.id);
  receipt.resolve({ importedNodes: [], revision: 1 });
  await operation;
  await cancellation;
  expect(tasks.tasks[0]?.status).toBe("succeeded");
});

it("reports cleanup errors as failures even after cancellation was requested", async () => {
  const { receipt, operation, tasks } = await startRegisteredImport();
  const rejection = operation.catch((error: unknown) => error);
  const cancellation = tasks.cancel(tasks.tasks[0]!.id);
  receipt.reject(
    new Error("cleanup failed", { cause: { code: "operation_failed", message: "cleanup failed" } }),
  );
  expect(await rejection).toBeInstanceOf(Error);
  await cancellation;
  expect(tasks.tasks[0]?.status).toBe("failed");
});

it("does not offer cancellation before Rust registers the operation", async () => {
  const receipt = deferred<Receipt>();
  mocks.import.mockReturnValue(receipt.promise);
  const operation = startImport();
  const tasks = useTaskStore();
  await vi.waitFor(() => expect(mocks.import).toHaveBeenCalled());
  expect(tasks.tasks[0]?.cancellable).not.toBe(true);
  await tasks.cancel(tasks.tasks[0]!.id);
  expect(mocks.cancel).not.toHaveBeenCalled();
  receipt.resolve({ importedNodes: [], revision: 1 });
  await operation;
  expect(tasks.tasks[0]?.status).toBe("succeeded");
});
