import { describe, expect, it, vi } from "vitest";

import {
  createNodeContentHeightBatch,
  type FittedContentHeight,
} from "../canvas/konva/node-content-height-batch";
import { createSpaceNode } from "../canvas/node";
import type { CanvasNodeLayoutV1 } from "../canvas/schema/document";
import { createSpaceTextPayload } from "../editor/tiptap-content";

const node = createSpaceNode(createSpaceTextPayload("Text"), { x: 0, y: 0 }, "text");
const layout: CanvasNodeLayoutV1 = {
  nodeId: "text",
  frame: { x: 0, y: 0, width: 240, height: 220, rotation: 0 },
  sizingMode: "frame",
  zIndex: 0,
};

describe("node content height batch", () => {
  it("commits the latest valid content measurement", async () => {
    const commit = vi.fn<(heights: ReadonlyMap<string, FittedContentHeight>) => void>();
    const batch = createNodeContentHeightBatch(
      (id) => (id === "text" ? node : undefined),
      (id) => (id === "text" ? layout : undefined),
      commit,
    );

    batch.fit("text", {
      contentRevision: null,
      height: 96,
      sizingMode: "frame",
      typographyScale: 1,
      width: 240,
    });
    batch.fit("text", {
      contentRevision: null,
      height: 120,
      sizingMode: "frame",
      typographyScale: 1,
      width: 240,
    });
    await Promise.resolve();

    expect(commit).toHaveBeenCalledOnce();
    expect(commit.mock.calls[0]?.[0].get("text")?.height).toBe(120);
  });

  it("defers work while a direct resize owns the frame", async () => {
    const commit = vi.fn<(heights: ReadonlyMap<string, FittedContentHeight>) => void>();
    const batch = createNodeContentHeightBatch(
      () => node,
      () => layout,
      commit,
    );
    batch.setPaused(true);
    batch.fit("text", {
      contentRevision: null,
      height: 96,
      sizingMode: "frame",
      typographyScale: 1,
      width: 240,
    });
    await Promise.resolve();
    expect(commit).not.toHaveBeenCalled();

    batch.setPaused(false);
    await Promise.resolve();
    expect(commit.mock.calls[0]?.[0].get("text")?.height).toBe(96);
  });

  it("drops measurements captured for a stale width", async () => {
    const commit = vi.fn<(heights: ReadonlyMap<string, FittedContentHeight>) => void>();
    const batch = createNodeContentHeightBatch(
      () => node,
      () => layout,
      commit,
    );
    batch.fit("text", {
      contentRevision: null,
      height: 96,
      sizingMode: "frame",
      typographyScale: 1,
      width: 200,
    });
    await Promise.resolve();
    expect(commit).not.toHaveBeenCalled();
  });
  it("does not commit a repeated measurement with unchanged source and height", async () => {
    const commit = vi.fn<(heights: ReadonlyMap<string, FittedContentHeight>) => void>();
    const batch = createNodeContentHeightBatch(
      () => node,
      () => layout,
      commit,
    );
    batch.fit("text", {
      contentRevision: null,
      height: 220,
      sizingMode: "frame",
      typographyScale: 1,
      width: 240,
    });
    await Promise.resolve();
    expect(commit).not.toHaveBeenCalled();
  });
});
