import { describe, expect, it } from "vitest";

import { canvasNodeInteraction } from "../canvas-node-interaction";
import { projectSelectionTransformPolicy } from "../../selection/selection-transform-policy";
import { createSpaceNotePayload, createSpaceTextPayload } from "../../../../editor/tiptap-content";
import type { SpaceCanvasNode } from "../../../../model/nodes/canvas-node";

describe("canvas node interaction policy", () => {
  it("distinguishes media controls on selection from embedded editing on activation", () => {
    expect(canvasNodeInteraction(canvasNode("audio", audioData())).activationMode).toBe("select-to-interact");
    expect(canvasNodeInteraction(canvasNode("note", createSpaceNotePayload("Note"))).activationMode).toBe(
      "activate-to-interact",
    );
    expect(canvasNodeInteraction(canvasNode("text", createSpaceTextPayload("Text"))).activationMode).toBe(
      "activate-to-interact",
    );
    expect(
      canvasNodeInteraction(
        canvasNode("website", {
          kind: "website",
          title: "Example",
          url: "https://example.com",
        }),
      ).activationMode,
    ).toBe("activate-to-interact");
    expect(
      canvasNodeInteraction(
        canvasNode("file", {
          kind: "file",
          title: "archive.zip",
          entries: [{ path: "file.txt", kind: "file" }],
        }),
      ).activationMode,
    ).toBe("select-to-interact");
    expect(canvasNodeInteraction(canvasNode("file", { kind: "file", title: "doc.pdf" })).activationMode).toBe(
      "primary",
    );
    expect(
      canvasNodeInteraction(canvasNode("section", { kind: "section", title: "Section" })).activationMode,
    ).toBe("primary");
  });

  it("keeps section and navigation nodes out of connector creation", () => {
    for (const node of [
      canvasNode("section", { kind: "section", title: "Section" }),
      canvasNode("portal", { kind: "portal", title: "Portal", targetSpaceId: "space" }),
      canvasNode("credential", { kind: "credential", title: "Credential", credentialKind: "api-key" }),
    ]) {
      expect(canvasNodeInteraction(node).connectable).toBe(false);
    }
  });

  it("keeps text relations independent from its specialized resize policy", () => {
    expect(canvasNodeInteraction(canvasNode("text", createSpaceTextPayload("Text"))).connectable).toBe(true);
  });

  it("projects fixed and collapsed nodes as move-only selections", () => {
    const fixed = canvasNode("audio", {
      kind: "audio",
      title: "Audio",
      assetPath: "/audio.wav",
      fileName: "audio.wav",
    });
    const collapsed = canvasNode("section", { kind: "section", title: "Section", collapsed: true });

    expect(canvasNodeInteraction(fixed).transform.resizeMode).toBe("fixed");
    expect(canvasNodeInteraction(collapsed).transform.resizeMode).toBe("fixed");
  });

  it("uses the selected node's semantic bounds and aspect policy", () => {
    const image = canvasNode("image", {
      kind: "image",
      title: "Image",
      assetPath: "/image.png",
      fileName: "image.png",
      intrinsicWidth: 1200,
      intrinsicHeight: 800,
    });
    const interaction = canvasNodeInteraction(image);
    const policy = projectSelectionTransformPolicy(
      [{ layout: layout("image"), node: image, interaction, paintOrder: 0 }],
      new Set(["image"]),
    );

    expect(policy).toMatchObject({
      nodeIds: ["image"],
      resizeMode: "proportional",
      minWidth: 200,
      minHeight: 164,
      maxWidth: Number.POSITIVE_INFINITY,
      maxHeight: Number.POSITIVE_INFINITY,
    });
  });

  it("gives text width handles and one proportional scale handle", () => {
    const text = canvasNode("text", createSpaceTextPayload("Text"));
    expect(canvasNodeInteraction(text).transform.resizeMode).toBe("width-and-proportional");
  });

  it("does not expose ambiguous resize handles for a mixed multi-selection", () => {
    const left = canvasNode("left", createSpaceNotePayload("Left"));
    const right = canvasNode("right", createSpaceNotePayload("Right"));
    const items = [left, right].map((node, paintOrder) => ({
      layout: layout(node.id),
      node,
      interaction: canvasNodeInteraction(node),
      paintOrder,
    }));

    expect(projectSelectionTransformPolicy(items, new Set(["left", "right"]))).toMatchObject({
      nodeIds: ["left", "right"],
      resizeMode: "fixed",
    });
  });
});

function canvasNode(id: string, data: SpaceCanvasNode["data"]): SpaceCanvasNode {
  return { id, type: data.kind, data, initialPosition: { x: 0, y: 0 } };
}

function audioData() {
  return { kind: "audio", title: "Audio", assetPath: "/audio.wav", fileName: "audio.wav" } as const;
}

function layout(nodeId: string) {
  return {
    nodeId,
    frame: { x: 0, y: 0, width: 300, height: 220, rotation: 0 },
    sizeAuthority: "frame",
    zIndex: 0,
  };
}
