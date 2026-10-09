import { Channel } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { invokeCommand } from "../../../../foundation/ipc/invoke-command";
import type { SpacePackageProgress } from "../../contracts/generated/SpacePackageProgress";
import { cancelSpacePackageTransfer, exportSpacePackage } from "../transfer";

const bridge = vi.hoisted(() => ({
  invoke: vi.fn<typeof invokeCommand>(),
}));

vi.mock("../../../../foundation/ipc/invoke-command", () => ({ invokeCommand: bridge.invoke }));
vi.mock("@tauri-apps/api/core", () => ({
  Channel: class {
    onmessage: ((event: SpacePackageProgress) => void) | undefined;
  },
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn<() => Promise<null>>(),
  save: vi.fn<() => Promise<null>>(),
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

function admitted(channel: Channel<SpacePackageProgress>) {
  channel.onmessage({ stage: "admitted", processed: 0, total: 0, percent: 100 });
}

describe("Space package transfer admission", () => {
  beforeEach(() => bridge.invoke.mockReset());

  it("waits for backend admission before invoking cancellation", async () => {
    const exportCompletion = deferred<void>();
    bridge.invoke.mockImplementation((command) => {
      if (command === "plugin:spaces|export_space_package") return exportCompletion.promise;
      return Promise.resolve(undefined);
    });
    const transfer = exportSpacePackage(
      { space: { id: "space", title: "Space" } },
      "/tmp/space.knovumspace",
      "00000000-0000-4000-8000-000000000001",
    );
    const cancellation = transfer.admitted.then(() =>
      cancelSpacePackageTransfer("00000000-0000-4000-8000-000000000001"),
    );

    await Promise.resolve();
    expect(bridge.invoke).toHaveBeenCalledTimes(1);
    const args = bridge.invoke.mock.calls[0]?.[1] as { onProgress: Channel<SpacePackageProgress> };
    const channel = args.onProgress;
    expect(channel).toBeInstanceOf(Channel);
    admitted(channel as Channel<SpacePackageProgress>);
    await cancellation;
    expect(bridge.invoke).toHaveBeenLastCalledWith("plugin:spaces|cancel_space_package_transfer", {
      operationId: "00000000-0000-4000-8000-000000000001",
    });
    exportCompletion.resolve(undefined);
    await transfer.completion;
  });

  it("rejects failed admission without invoking cancellation", async () => {
    const failure = new Error("invalid operation ID");
    bridge.invoke.mockRejectedValueOnce(failure);
    const transfer = exportSpacePackage(
      { space: { id: "space", title: "Space" } },
      "/tmp/space.knovumspace",
      "invalid",
    );
    const cancellation = transfer.admitted.then(() => cancelSpacePackageTransfer("invalid"));

    await expect(transfer.completion).rejects.toBe(failure);
    await expect(cancellation).rejects.toBe(failure);
    expect(bridge.invoke).toHaveBeenCalledTimes(1);
  });
});
