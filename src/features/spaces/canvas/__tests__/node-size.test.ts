import { describe, expect, it } from "vitest";

import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import { spaceNodeDimensions, spaceNodeResizePolicy, spaceNodeSizePolicy } from "../node-size";

function visual(kind: "image" | "video", width: number, height: number): SpaceNodePayload {
  return {
    kind,
    title: `${kind} asset`,
    assetPath: `/spaces/assets/${kind}.bin`,
    fileName: `${kind}.bin`,
    intrinsicWidth: width,
    intrinsicHeight: height,
  };
}

function rendered(width?: number, height?: number): SpaceNodePayload {
  const data: SpaceNodePayload = {
    kind: "file",
    title: "Quarterly Review",
    assetPath: "/spaces/assets/quarterly.docx",
    fileName: "quarterly.docx",
  };
  if (width !== undefined) data.intrinsicWidth = width;
  if (height !== undefined) data.intrinsicHeight = height;
  return data;
}

describe("spaces node size policy", () => {
  it.each([
    ["note", 192, 192],
    ["text", 220, 40],
    ["markdown", 260, 184],
    ["website", 288, 208],
    ["audio", 288, 80],
    ["file", 256, 192],
    ["code", 304, 224],
    ["credential", 220, 56],
    ["section", 432, 288],
    ["object", 256, 176],
    ["portal", 256, 88],
  ] as const)("gives %s a semantic desktop default", (kind, width, height) => {
    const policy = spaceNodeSizePolicy({ kind, title: kind } as SpaceNodePayload);
    expect({ width: policy.width, height: policy.height }).toEqual({ width, height });
  });

  it("fits standard and long-form image ratios without cropping or distortion", () => {
    expect(spaceNodeDimensions(visual("image", 1920, 1080))).toEqual({ width: 280, height: 206 });
    expect(spaceNodeDimensions(visual("image", 1080, 1920))).toEqual({ width: 158, height: 328 });
    expect(spaceNodeDimensions(visual("image", 1080, 3840))).toEqual({ width: 128, height: 503 });
  });

  it("reserves stable desktop chrome around video presentation geometry", () => {
    expect(spaceNodeDimensions(visual("video", 1920, 1080))).toEqual({ width: 304, height: 223 });
    expect(spaceNodeDimensions(visual("video", 1080, 1920))).toEqual({ width: 171, height: 356 });
    expect(spaceNodeSizePolicy(visual("video", 1920, 1080)).contentAspect?.fixedVerticalChrome).toBe(52);
  });

  it("sizes a rendered document card from the page's own ratio", () => {
    expect(spaceNodeDimensions(rendered(210, 297))).toEqual({ width: 192, height: 320 });
    expect(spaceNodeDimensions(rendered(792, 1024))).toEqual({ width: 210, height: 320 });
    expect(spaceNodeDimensions(rendered(1920, 1080))).toEqual({ width: 272, height: 201 });
  });

  it("keeps the page area inside the card true to the rendered ratio", () => {
    const dimensions = spaceNodeDimensions(rendered(792, 1024));
    expect(dimensions.width / (dimensions.height - 48)).toBeCloseTo(792 / 1024, 2);
  });

  it("uses every available system-preview ratio and leaves unrendered documents at the default", () => {
    expect(spaceNodeDimensions(rendered(1024, 1024))).toEqual({ width: 272, height: 320 });
    expect(spaceNodeDimensions(rendered())).toEqual({ width: 256, height: 192 });
  });

  it("uses the rendered compact geometry for collapsed sections", () => {
    expect(
      spaceNodeDimensions({
        kind: "section",
        title: "Collapsed section",
        collapsed: true,
      }),
    ).toEqual({ width: 288, height: 36 });
  });

  it("keeps compact utility nodes fixed", () => {
    const portal = {
      kind: "portal",
      title: "Portal",
      targetSpaceId: "s-1",
    } as SpaceNodePayload;
    expect(spaceNodeDimensions(portal)).toEqual({ width: 256, height: 88 });
  });

  it("keeps compact utility nodes fixed while visual cards remain resizable", () => {
    expect(spaceNodeResizePolicy(visual("image", 1200, 800))).toMatchObject({
      resizeMode: "proportional",
    });
    expect(spaceNodeResizePolicy(visual("video", 1200, 800))).toMatchObject({
      resizeMode: "proportional",
    });
    const audio = {
      kind: "audio",
      title: "Audio",
      assetPath: "/audio.wav",
      fileName: "audio.wav",
    } as SpaceNodePayload;
    expect(spaceNodeResizePolicy(audio).resizeMode).toBe("fixed");
  });

  it("does not impose a presentation cap on resizable nodes", () => {
    for (const data of [
      { kind: "website", title: "Website", url: "https://example.com/" },
      { kind: "note", title: "Note" },
      visual("image", 1920, 1080),
      visual("video", 1920, 1080),
    ] as SpaceNodePayload[]) {
      expect(spaceNodeResizePolicy(data)).toMatchObject({
        maxWidth: Number.POSITIVE_INFINITY,
        maxHeight: Number.POSITIVE_INFINITY,
      });
    }
  });

  it("keeps document and section geometry independently resizable", () => {
    for (const data of [
      { kind: "note", title: "Note" },
      { kind: "markdown", title: "Document" },
      { kind: "subtitle", title: "Transcript" },
      { kind: "section", title: "Section" },
    ] as SpaceNodePayload[]) {
      expect(spaceNodeResizePolicy(data)).toMatchObject({ resizeMode: "free" });
    }
    expect(spaceNodeResizePolicy({ kind: "text", title: "Text" } as SpaceNodePayload)).toMatchObject({
      resizeMode: "width-and-proportional",
    });
  });

  it("derives initial visual dimensions only from intrinsic content", () => {
    expect(spaceNodeDimensions(visual("image", 1920, 1080))).toEqual({ width: 280, height: 206 });
    expect(spaceNodeDimensions(visual("video", 1920, 1080))).toEqual({ width: 304, height: 223 });
  });
});
