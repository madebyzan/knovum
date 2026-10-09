import { beforeEach, describe, expect, it, vi } from "vitest";

import type { SpacePackageTransfer } from "../transfer";
import { useSpacePackageTransfer } from "../use-space-package-transfer";

const mocks = vi.hoisted(() => ({
  cancel: vi.fn<() => Promise<void>>(),
  cancelTask: undefined as (() => Promise<void>) | undefined,
  exportTransfer: vi.fn<() => SpacePackageTransfer>(),
}));

vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));
vi.mock("vue-sonner", () => ({
  toast: {
    error: vi.fn<(message: string) => void>(),
    success: vi.fn<(message: string) => void>(),
  },
}));
vi.mock("../../../../foundation/tasks/task-store", () => ({
  useTaskStore: () => ({
    cancelled: vi.fn<(taskId: string) => void>(),
    fail: vi.fn<(taskId: string, message: string) => void>(),
    progress: vi.fn<(taskId: string, percent: number, message: string) => void>(),
    start: (_title: string, _message: undefined, cancelTask: () => Promise<void>) => {
      mocks.cancelTask = cancelTask;
    },
    succeed: vi.fn<(taskId: string) => void>(),
  }),
}));
vi.mock("../../session/space-store", () => ({
  useSpaceStore: () => ({
    commitActiveSession: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
    spaces: [],
  }),
}));
vi.mock("../transfer", () => ({
  cancelSpacePackageTransfer: mocks.cancel,
  chooseSpacePackageExportDestination: vi
    .fn<() => Promise<string>>()
    .mockResolvedValue("/tmp/space.knovumspace"),
  chooseSpacePackageImportSource: vi.fn<() => Promise<undefined>>(),
  exportSpacePackage: mocks.exportTransfer,
  importSpacePackage: vi.fn<() => SpacePackageTransfer>(),
}));

function deferred<T>() {
  let resolvePromise!: (value: T) => void;
  let rejectPromise!: (error: unknown) => void;
  const promise = new Promise<T>((resolve, reject) => {
    resolvePromise = resolve;
    rejectPromise = reject;
  });
  return { promise, reject: rejectPromise, resolve: resolvePromise };
}

describe("Space package task cancellation", () => {
  beforeEach(() => {
    mocks.cancel.mockReset().mockResolvedValue(undefined);
    mocks.exportTransfer.mockReset();
    mocks.cancelTask = undefined;
  });

  it("does not invoke backend cancellation until the transfer is admitted", async () => {
    const admission = deferred<void>();
    const completion = deferred<void>();
    mocks.exportTransfer.mockReturnValue({ admitted: admission.promise, completion: completion.promise });

    const run = useSpacePackageTransfer().exportPackage({ id: "space", title: "Space" });
    await vi.waitFor(() => expect(mocks.cancelTask).toBeTypeOf("function"));
    const cancellation = mocks.cancelTask!();
    await Promise.resolve();
    expect(mocks.cancel).not.toHaveBeenCalled();

    admission.resolve(undefined);
    await vi.waitFor(() => expect(mocks.cancel).toHaveBeenCalledTimes(1));
    completion.reject({ code: "cancelled", message: "cancelled" });
    await cancellation;
    await run;
  });
});
