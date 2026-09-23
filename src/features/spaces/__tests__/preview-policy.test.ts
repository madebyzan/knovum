import { describe, expect, it } from "vitest";

import {
  canvasViewportDetailProfile,
  richPreviewZoomThreshold,
  shouldRenderRichCanvasPreview,
} from "../canvas/contract/canvas-detail-profile";

describe("spaces preview policy", () => {
  it("raises the rich-preview threshold as the canvas grows", () => {
    expect(richPreviewZoomThreshold(100)).toBe(78);
    expect(richPreviewZoomThreshold(400)).toBe(88);
    expect(richPreviewZoomThreshold(800)).toBe(100);
    expect(richPreviewZoomThreshold(1_500)).toBe(115);
  });

  it("keeps large zoomed-out canvases lightweight", () => {
    expect(shouldRenderRichCanvasPreview(99, 900)).toBe(false);
    expect(shouldRenderRichCanvasPreview(100, 900)).toBe(true);
  });

  it("changes information density instead of merely scaling full cards", () => {
    expect(canvasViewportDetailProfile(30, 100)).toEqual({ level: "compact", previewItemLimit: 2 });
    expect(canvasViewportDetailProfile(55, 100)).toEqual({ level: "summary", previewItemLimit: 4 });
    expect(canvasViewportDetailProfile(78, 100)).toEqual({ level: "full", previewItemLimit: 10 });
    expect(canvasViewportDetailProfile(75, 900)).toEqual({ level: "summary", previewItemLimit: 4 });
  });
});
