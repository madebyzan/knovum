import { createPinia, setActivePinia } from "pinia";
import { beforeEach, expect, it, vi } from "vitest";

import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { importJsonCanvas } from "../canvas-exchange/json-canvas-import-operations";
import { createJsonCanvasImportActions } from "../session/actions/json-canvas-import-actions";
import { createSpaceState } from "../session/session-state";
type CanvasImport = typeof importJsonCanvas;
import type { loadSpaceSessionProjection } from "../session/space-projection";
type Projection = typeof loadSpaceSessionProjection;
type Receipt = Awaited<ReturnType<CanvasImport>>;
const native = vi.hoisted(() => ({
  import: vi.fn<CanvasImport>(),
  cancel: vi.fn<(mutationId: string) => Promise<void>>(),
  projection: vi.fn<Projection>(),
}));
vi.mock("../canvas-exchange/json-canvas-import-operations", () => ({
  importJsonCanvas: native.import,
  cancelJsonCanvasImport: native.cancel,
}));
vi.mock("../session/space-projection", () => ({ loadSpaceSessionProjection: native.projection }));

function pending<T>() {
  let fulfill!: (value: T) => void;
  let fail!: (reason: unknown) => void;
  const promise = new Promise<T>((resolve, reject) => {
    fulfill = resolve;
    fail = reject;
  });
  return { promise, resolve: fulfill, reject: fail };
}

beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
  native.cancel.mockResolvedValue(undefined);
  native.projection.mockResolvedValue({
    nodes: [],
    scenes: [],
    relations: [],
    editSnapshot: { editRevision: 1, structure: [], layouts: [], relationStyles: [] },
  });
});

function startImport() {
  const state = createSpaceState();
  state.activeSpaceId.value = "space";
  return createJsonCanvasImportActions({
    state,
    ensureActiveSpacePersisted: async () => {},
    mutations: { commitPendingContent: vi.fn<() => void>(), applyNodeContentForSpace: vi.fn<() => void>() },
    placement: { requestReveal: vi.fn<() => void>() },
  }).importJsonCanvas({ sourcePath: "/canvas.canvas", baseRevision: 0 });
}

it("waits for native cancellation and cleanup before settling the task", async () => {
  const receipt = pending<Receipt>();
  native.import.mockImplementation(
    (_input, onProgress: (progress: { stage: string; bytesCopied: number }) => void) => {
      onProgress({ stage: "preparing", bytesCopied: 0 });
      return receipt.promise;
    },
  );
  const operation = startImport();
  const tasks = useTaskStore();
  await vi.waitFor(() => expect(tasks.tasks[0]?.cancellable).toBe(true));
  const rejection = operation.catch((error: unknown) => error);
  const cancellation = tasks.cancel(tasks.tasks[0]!.id);
  await vi.waitFor(() => expect(native.cancel).toHaveBeenCalled());
  expect(tasks.tasks[0]?.status).toBe("running");
  receipt.reject(
    new Error("cancelled", { cause: { code: "json_canvas_import_cancelled", message: "cancelled" } }),
  );
  expect(await rejection).toBeInstanceOf(Error);
  await cancellation;
  expect(tasks.tasks[0]?.status).toBe("cancelled");
  expect(native.projection).not.toHaveBeenCalled();
});

it("keeps a committed import successful when cancellation loses the race", async () => {
  const receipt = pending<Receipt>();
  native.import.mockImplementation(
    (_input, onProgress: (progress: { stage: string; bytesCopied: number }) => void) => {
      onProgress({ stage: "preparing", bytesCopied: 0 });
      return receipt.promise;
    },
  );
  const operation = startImport();
  const tasks = useTaskStore();
  await vi.waitFor(() => expect(tasks.tasks[0]?.cancellable).toBe(true));
  const cancellation = tasks.cancel(tasks.tasks[0]!.id);
  receipt.resolve({ importedNodes: [], revision: 1 });
  await operation;
  await cancellation;
  expect(tasks.tasks[0]?.status).toBe("succeeded");
});

it("reports cleanup errors as failures even after cancellation was requested", async () => {
  const receipt = pending<Receipt>();
  native.import.mockImplementation(
    (_input, onProgress: (progress: { stage: string; bytesCopied: number }) => void) => {
      onProgress({ stage: "preparing", bytesCopied: 0 });
      return receipt.promise;
    },
  );
  const operation = startImport();
  const rejection = operation.catch((error: unknown) => error);
  const tasks = useTaskStore();
  await vi.waitFor(() => expect(tasks.tasks[0]?.cancellable).toBe(true));
  const cancellation = tasks.cancel(tasks.tasks[0]!.id);
  receipt.reject(
    new Error("cleanup failed", { cause: { code: "operation_failed", message: "cleanup failed" } }),
  );
  expect(await rejection).toBeInstanceOf(Error);
  await cancellation;
  expect(tasks.tasks[0]?.status).toBe("failed");
});

it("does not offer cancellation before Rust registers the operation", async () => {
  const receipt = pending<Receipt>();
  native.import.mockReturnValue(receipt.promise);
  const operation = startImport();
  const tasks = useTaskStore();
  await vi.waitFor(() => expect(native.import).toHaveBeenCalled());
  expect(tasks.tasks[0]?.cancellable).not.toBe(true);
  await tasks.cancel(tasks.tasks[0]!.id);
  expect(native.cancel).not.toHaveBeenCalled();
  receipt.resolve({ importedNodes: [], revision: 1 });
  await operation;
  expect(tasks.tasks[0]?.status).toBe("succeeded");
});
