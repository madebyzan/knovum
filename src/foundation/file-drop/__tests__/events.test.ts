import { beforeEach, describe, expect, it, vi } from "vitest";

import { listenForFileDrops } from "../events";

type DragPayload =
  | { type: "enter"; paths: string[]; position: { x: number; y: number } }
  | { type: "over"; position: { x: number; y: number } }
  | { type: "drop"; paths: string[]; position: { x: number; y: number } }
  | { type: "leave" };

const mocks = vi.hoisted(() => ({
  emit: undefined as ((event: { payload: DragPayload }) => void) | undefined,
}));

vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => ({
    onDragDropEvent: async (handler: (event: { payload: DragPayload }) => void) => {
      mocks.emit = handler;
      return () => {
        mocks.emit = undefined;
      };
    },
  }),
}));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => ({}) }));
vi.mock("@tauri-apps/plugin-os", () => ({ platform: () => "macos" }));
vi.mock("@tauri-apps/plugin-log", () => ({ error: vi.fn<() => Promise<void>>() }));

beforeEach(() => {
  mocks.emit = undefined;
});

describe("listenForFileDrops", () => {
  it("ignores internal WebView drags that carry no OS file paths", async () => {
    const hover = vi.fn<(...args: unknown[]) => void>();
    const drop = vi.fn<(...args: unknown[]) => void>();
    const dispose = await listenForFileDrops({ hover, drop });

    mocks.emit?.({ payload: { type: "enter", paths: [], position: { x: 10, y: 20 } } });
    mocks.emit?.({ payload: { type: "over", position: { x: 15, y: 25 } } });
    mocks.emit?.({ payload: { type: "drop", paths: [], position: { x: 15, y: 25 } } });

    expect(hover).not.toHaveBeenCalledWith(true, expect.anything());
    expect(drop).not.toHaveBeenCalled();

    mocks.emit?.({ payload: { type: "enter", paths: ["/tmp/photo.png"], position: { x: 30, y: 40 } } });
    mocks.emit?.({ payload: { type: "over", position: { x: 35, y: 45 } } });
    expect(hover).toHaveBeenCalledWith(true, { x: 30, y: 40 });
    expect(hover).toHaveBeenCalledWith(true, { x: 35, y: 45 });

    dispose();
  });
});
