import { describe, expect, it, vi } from "vitest";

import { revealCanvasLayouts } from "../canvas-editor-navigation";

describe("canvas editor navigation", () => {
  it("pans an off-screen node into view without changing zoom", () => {
    const setViewport = vi.fn<(viewport: { x: number; y: number; zoom: number }) => void>();
    revealCanvasLayouts(navigationContext(1.25, setViewport), ["node"], "ensure-visible");

    expect(setViewport).toHaveBeenCalledWith(expect.objectContaining({ zoom: 1.25 }));
  });

  it("only changes scale for an explicit focus request", () => {
    const setViewport = vi.fn<(viewport: { x: number; y: number; zoom: number }) => void>();
    revealCanvasLayouts(navigationContext(0.75, setViewport), ["node"], "focus");

    expect(setViewport).toHaveBeenCalledWith(expect.objectContaining({ zoom: 2 }));
  });
});

function navigationContext(
  zoom: number,
  setViewport: (viewport: { x: number; y: number; zoom: number }) => void,
) {
  return {
    layouts: () => [layout("node", 900, 700)],
    selectedNodeIds: () => new Set<string>(),
    setSelection: vi.fn<(ids: readonly string[]) => void>(),
    setViewport,
    size: () => ({ width: 800, height: 600 }),
    viewport: () => ({ x: 0, y: 0, zoom }),
  };
}

function layout(nodeId: string, x: number, y: number) {
  return {
    nodeId,
    frame: { x, y, width: 300, height: 220, rotation: 0 },
    sizeAuthority: "frame",
    zIndex: 0,
  };
}
