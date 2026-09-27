import { describe, expect, it } from "vitest";

import { describeSpaceNode } from "../model/nodes/node-description";
import { spaceNodePayloadSchema } from "../model/nodes/schema/node-variants";
import { spaceNodePreviewText } from "../presentation/canvas/nodes/space-canvas-node-content";

const context = { mode: "preview", viewport: { level: "compact", previewItemLimit: 2 } } as const;

function payload(fields: unknown) {
  return spaceNodePayloadSchema.parse(fields);
}

describe("node description and preview", () => {
  it("preserves explicit empty text and uses website fallback in order", () => {
    const website = {
      title: "Website",
      kind: "website",
      url: "https://example.com",
      contentPreview: "Preview",
    };
    expect(describeSpaceNode(payload(website))).toBe("Preview");
    expect(describeSpaceNode(payload({ ...website, description: "" }))).toBe("");
    expect(describeSpaceNode(payload({ ...website, contentPreview: undefined }))).toBe("https://example.com");
    expect(spaceNodePreviewText(payload(website), context)).toBe("Preview");
  });

  it("describes portal destination and preserves object summary fallback", () => {
    const portal = { title: "Portal", kind: "portal", targetSpaceId: "space" };
    expect(describeSpaceNode(payload(portal))).toBe("Linked space");
    expect(describeSpaceNode(payload({ ...portal, targetSceneId: "scene" }))).toBe("Linked scene");
    const object = {
      title: "Object",
      kind: "object",
      objectTypeId: "custom",
      status: "draft",
      properties: {},
    };
    expect(describeSpaceNode(payload(object))).toBe("custom");
    expect(describeSpaceNode(payload({ ...object, summary: "" }))).toBe("");
  });

  it("keeps subtitle description independent of viewport preview limits", () => {
    const subtitle = payload({
      title: "Captions",
      kind: "subtitle",
      sourceVideoNodeId: "video",
      sourceVideoRevision: "revision",
      language: "en",
      durationMs: 4000,
      cues: ["One", "Two", "Three", "Four"].map((text, index) => ({
        startMs: index * 1000,
        endMs: (index + 1) * 1000,
        text,
      })),
    });
    expect(describeSpaceNode(subtitle)).toBe("One\nTwo\nThree");
    expect(spaceNodePreviewText(subtitle, context)).toBe("One\nTwo");
  });
});
