import { describe, expect, it } from "vitest";

import { createSpaceNotePayload } from "../../document-editing/tiptap-content";
import type { SpaceEditSnapshot } from "../../model/structure/schema/edit-snapshot";
import { SECTION_HEADER_HEIGHT } from "../../model/structure/section-metrics";
import { projectMinimapScene } from "../minimap/minimap-scene";
import {
  computeMinimapTransform,
  computeMinimapViewport,
  MINIMAP_DIMENSIONS,
  minimapPointToWorld,
  viewportCenteredAtWorld,
} from "../minimap/minimap-transform";
import { createSpaceNode } from "../node";

describe("minimap scene projection and transforms", () => {
  it("projects visible nodes, collapsed section heights, selection, focus, and relation colors", () => {
    const noteA = createSpaceNode(createSpaceNotePayload("Alpha"), { x: 0, y: 0 }, "note-a");
    const noteB = createSpaceNode(createSpaceNotePayload("Beta"), { x: 600, y: 400 }, "note-b");
    const noteHidden = {
      ...createSpaceNode(createSpaceNotePayload("Hidden"), { x: 5000, y: 5000 }, "note-hidden"),
      hidden: true,
    };
    const snapshot: SpaceEditSnapshot = {
      editRevision: 1,
      structure: [
        { id: "sec-1", kind: "section", parentId: null, order: 0, locked: false, title: "Collapsed" },
        { id: "c-a", kind: "content", nodeId: "note-a", parentId: "sec-1", order: 0, locked: false },
        { id: "c-b", kind: "content", nodeId: "note-b", parentId: null, order: 1, locked: false },
        { id: "c-hidden", kind: "content", nodeId: "note-hidden", parentId: null, order: 2, locked: false },
      ],
      layouts: [
        {
          structureNodeId: "sec-1",
          kind: "section",
          position: { x: 0, y: 0 },
          size: { width: 400, height: 500 },
          collapsed: true,
          autoLayout: false,
        },
        {
          structureNodeId: "c-a",
          kind: "content",
          transform: { x: 20, y: 60, rotation: 0, scale: 1 },
          size: { width: 200, height: 140 },
          sizingMode: "content",
        },
        {
          structureNodeId: "c-b",
          kind: "content",
          transform: { x: 600, y: 400, rotation: 15, scale: 1 },
          size: { width: 260, height: 180 },
          sizingMode: "frame",
          appearance: { fill: "blue" },
        },
        {
          structureNodeId: "c-hidden",
          kind: "content",
          transform: { x: 5000, y: 5000, rotation: 0, scale: 1 },
          size: { width: 200, height: 100 },
          sizingMode: "content",
        },
      ],
      relationStyles: [{ relationId: "rel-1", color: "purple" }],
    };

    const scene = projectMinimapScene({
      snapshot,
      nodes: [noteA, noteB, noteHidden],
      relations: [
        {
          id: "rel-1",
          spaceId: "inbox",
          sourceId: "note-a",
          targetId: "note-b",
          kind: "related",
          rationale: "related notes",
          status: "active",
          actorKind: "human",
          createdAt: 0,
          updatedAt: 0,
        },
      ],
      selectedNodeIds: new Set(["note-b"]),
      selectedStructureNodeIds: new Set(["sec-1"]),
      focusNodeIds: new Set(["note-b"]),
    });

    expect(scene.sections).toHaveLength(1);
    expect(scene.sections[0]?.collapsed).toBe(true);
    expect(scene.sections[0]?.bounds.bottom).toBe(SECTION_HEADER_HEIGHT);
    expect(scene.sections[0]?.selected).toBe(true);

    // note-a is inside a collapsed section and note-hidden is hidden, so only note-b is visible.
    expect(scene.nodes.map((node) => node.id)).toEqual(["note-b"]);
    expect(scene.nodes[0]?.selected).toBe(true);
    expect(scene.nodes[0]?.focused).toBe(true);
    expect(scene.nodes[0]?.fillPreset).toBe("blue");

    expect(scene.relations).toHaveLength(1);
    expect(scene.relations[0]?.color).toBe("purple");
    expect(scene.contentBounds?.left).toBe(0);
    expect(scene.contentBounds?.top).toBe(0);
  });

  it("computes stable world-to-minimap transforms and round-trips coordinates", () => {
    const transform = computeMinimapTransform({ left: 0, top: 0, right: 3000, bottom: 1500 });
    const worldCenter = { x: 1500, y: 750 };
    const mapCenter = {
      x: worldCenter.x * transform.scale + transform.offsetX,
      y: worldCenter.y * transform.scale + transform.offsetY,
    };
    expect(mapCenter.x).toBeCloseTo(MINIMAP_DIMENSIONS.width / 2, 5);
    expect(mapCenter.y).toBeCloseTo(MINIMAP_DIMENSIONS.height / 2, 5);

    const probe = { x: 42, y: 77 };
    const roundTripped = minimapPointToWorld(probe, transform);
    expect(roundTripped.x * transform.scale + transform.offsetX).toBeCloseTo(probe.x, 5);
    expect(roundTripped.y * transform.scale + transform.offsetY).toBeCloseTo(probe.y, 5);
  });

  it("projects viewport rect and provides an offscreen directional indicator when panned away", () => {
    const transform = computeMinimapTransform({ left: 0, top: 0, right: 2000, bottom: 1200 });
    const hostSize = { width: 1000, height: 700 };

    const centered = viewportCenteredAtWorld({ x: 1000, y: 600 }, 1, hostSize);
    const inView = computeMinimapViewport(centered, hostSize, transform);
    expect(inView.intersectsMap).toBe(true);
    expect(inView.offscreenIndicator).toBeUndefined();

    const farAway = viewportCenteredAtWorld({ x: 50000, y: 600 }, 1, hostSize);
    const outOfView = computeMinimapViewport(farAway, hostSize, transform);
    expect(outOfView.intersectsMap).toBe(false);
    expect(outOfView.offscreenIndicator?.x).toBe(MINIMAP_DIMENSIONS.width - 7);
    expect(outOfView.offscreenIndicator?.angleDeg).toBeCloseTo(0, 1);
  });
});
