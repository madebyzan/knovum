import { beforeEach, expect, it, vi } from "vitest";

import { deferred } from "../../../../shared/__tests__/support/deferred";
import { canvasViewStateOperations } from "../persistence/canvas-view-state";

const ipc = vi.hoisted(() => ({ invoke: vi.fn<(command: string, args: unknown) => Promise<unknown>>() }));
vi.mock("../../../../foundation/ipc/invoke-command", () => ({ invokeCommand: ipc.invoke }));

const LOAD = "plugin:spaces|load_canvas_view_state";
const SAVE = "plugin:spaces|save_canvas_view_state";

beforeEach(() => ipc.invoke.mockReset());

it("reads a Space's camera from storage once, then from what this window knows", async () => {
  ipc.invoke.mockResolvedValueOnce({ x: 1, y: 2, zoom: 1 });

  expect(await canvasViewStateOperations.load("read-once")).toEqual({ x: 1, y: 2, zoom: 1 });
  expect(await canvasViewStateOperations.load("read-once")).toEqual({ x: 1, y: 2, zoom: 1 });
  expect(ipc.invoke).toHaveBeenCalledExactlyOnceWith(LOAD, { spaceId: "read-once" });
});

it("returns a camera it was asked to store before that write lands", async () => {
  const write = deferred();
  ipc.invoke.mockReturnValueOnce(write.promise);

  const saved = canvasViewStateOperations.save("written", { x: 5, y: 6, zoom: 2 });

  expect(await canvasViewStateOperations.load("written")).toEqual({ x: 5, y: 6, zoom: 2 });
  expect(ipc.invoke).toHaveBeenCalledExactlyOnceWith(SAVE, {
    request: { spaceId: "written", view: { x: 5, y: 6, zoom: 2 } },
  });
  write.resolve();
  await saved;
});

it("writes in the order made, and a failed write holds back none after it", async () => {
  const first = deferred();
  ipc.invoke.mockReturnValueOnce(first.promise).mockResolvedValueOnce(undefined);

  const failing = canvasViewStateOperations.save("ordered", { x: 1, y: 1, zoom: 1 });
  const next = canvasViewStateOperations.save("ordered", { x: 2, y: 2, zoom: 1 });
  await Promise.resolve();
  expect(ipc.invoke).toHaveBeenCalledOnce();
  first.reject(new Error("disk full"));

  await expect(failing).rejects.toThrow("disk full");
  await next;
  expect(ipc.invoke).toHaveBeenLastCalledWith(SAVE, {
    request: { spaceId: "ordered", view: { x: 2, y: 2, zoom: 1 } },
  });
  await canvasViewStateOperations.settled();
});
