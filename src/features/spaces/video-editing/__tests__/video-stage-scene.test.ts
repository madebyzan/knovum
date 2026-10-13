import { describe, expect, it } from "vitest";

import { createVideoEditProject, defaultTextStyle, type VideoTextOverlay } from "../project";
import { textAnchorX, textBoxPadding, textConfig } from "../stage/scene-text";
import { transitionStyle } from "../stage/scene-transition";

const output = createVideoEditProject({
  nodeId: "source",
  sourceRevision: "revision",
  durationMs: 4_000,
  sourceWidth: 1280,
  sourceHeight: 720,
}).output;

function overlay(style: Partial<VideoTextOverlay["style"]> = {}): VideoTextOverlay {
  return {
    id: "text",
    startMs: 0,
    endMs: 1_000,
    text: "Title",
    x: 0.5,
    y: 0.8,
    style: { ...defaultTextStyle("Inter"), ...style },
  };
}

describe("stage transitions", () => {
  it("dissolves by raising the incoming layer's opacity only", () => {
    const style = transitionStyle({ kind: "dissolve", progress: 0.25 }, output);
    expect(style.from).toMatchObject({ opacity: 1, dx: 0, dy: 0 });
    expect(style.to.opacity).toBe(0.25);
    expect(style.backdrop).toBeUndefined();
  });

  it("passes a fade through its colour with both layers dark at the midpoint", () => {
    const style = transitionStyle({ kind: "fadeBlack", progress: 0.5 }, output);
    expect(style.backdrop).toBe("#000000");
    expect(style.from.opacity).toBe(0);
    expect(style.to.opacity).toBe(0);
    expect(transitionStyle({ kind: "fadeWhite", progress: 0 }, output)).toMatchObject({
      backdrop: "#FFFFFF",
      from: { opacity: 1 },
    });
  });

  it("wipes by clipping the incoming layer to the revealed band", () => {
    expect(transitionStyle({ kind: "wipeRight", progress: 0.25 }, output).to.clip).toEqual({
      x: 0,
      y: 0,
      width: 320,
      height: 720,
    });
    expect(transitionStyle({ kind: "wipeUp", progress: 0.5 }, output).to.clip).toEqual({
      x: 0,
      y: 360,
      width: 1280,
      height: 360,
    });
  });

  it("slides both layers together so the cut travels as one move", () => {
    const style = transitionStyle({ kind: "slideLeft", progress: 0.25 }, output);
    expect(style.from.dx).toBe(-320);
    expect(style.to.dx).toBe(960);
    expect(style.from.dy).toBe(0);
  });

  it("clamps progress to the span of the transition", () => {
    expect(transitionStyle({ kind: "dissolve", progress: 2 }, output).to.opacity).toBe(1);
    expect(transitionStyle({ kind: "dissolve", progress: -1 }, output).to.opacity).toBe(0);
  });
});

describe("stage text", () => {
  it("keeps sizes 720p-relative and positions the anchor in output pixels", () => {
    const config = textConfig(overlay({ size: 48 }), output);
    expect(config).toMatchObject({ fontSize: 48, x: 640, y: 576, align: "center" });
    expect(config.fontStyle).toBe("600");
    expect(config.fontFamily).toBe('"Inter", sans-serif');
  });

  it("maps the outline, shadow and box decorations the export writes", () => {
    const outline = textConfig(overlay({ decoration: "outline", decorationStrength: 3 }), output);
    expect(outline).toMatchObject({ stroke: "#000000", strokeWidth: 6, shadowOpacity: 0 });
    const shadow = textConfig(overlay({ decoration: "shadow", decorationStrength: 2 }), output);
    expect(shadow).toMatchObject({ strokeWidth: 0, shadowOpacity: 1 });
    expect(shadow.shadowOffset).toEqual({ x: 2, y: 2 });
    expect(textBoxPadding(overlay({ decoration: "box", decorationStrength: 4 }), output)).toBe(4);
    expect(textBoxPadding(overlay({ decoration: "outline" }), output)).toBe(0);
  });

  it("anchors the text box the way the alignment does", () => {
    expect(textAnchorX("left", 200)).toBe(0);
    expect(textAnchorX("center", 200)).toBe(100);
    expect(textAnchorX("right", 200)).toBe(200);
  });
});
