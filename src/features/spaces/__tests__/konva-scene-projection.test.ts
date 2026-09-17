import { describe, expect, it } from "vitest";

import type { CanvasDocumentV1 } from "../canvas/contract/canvas-document";
import { projectCanvasScene } from "../canvas/konva/scene/scene-projection";
import { createSpaceNotePayload } from "../editor/tiptap-content";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

const frame = (x: number, y: number, width: number, height: number) => ({
  x,
  y,
  width,
  height,
  rotation: 0,
});

describe("Konva scene projection", () => {
  it("always paints section backgrounds behind their members", () => {
    const section: SpaceCanvasNode = {
      id: "section",
      type: "section",
      data: { kind: "section", title: "Group" },
      initialPosition: { x: 0, y: 0 },
    };
    const member: SpaceCanvasNode = {
      id: "member",
      type: "note",
      data: { ...createSpaceNotePayload("Member", ""), sectionId: "section" },
      initialPosition: { x: 40, y: 80 },
    };
    const document: CanvasDocumentV1 = {
      version: 1,
      viewport: { x: 0, y: 0, zoom: 1 },
      nodes: [
        { nodeId: "member", frame: frame(40, 80, 300, 220), sizeAuthority: "frame", zIndex: 0 },
        { nodeId: "section", frame: frame(0, 0, 640, 420), sizeAuthority: "frame", zIndex: 1 },
      ],
    };

    const scene = projectCanvasScene(document, [section, member], []);

    expect(scene.sections.map(({ node }) => node.id)).toEqual(["section"]);
    expect(scene.contentNodes.map(({ node }) => node.id)).toEqual(["member"]);
    expect(scene.nodes.map(({ node, paintOrder }) => [node.id, paintOrder])).toEqual([
      ["section", 0],
      ["member", 1],
    ]);
    expect(scene.sections[0]?.members.map(({ node }) => node.id)).toEqual(["member"]);
  });

  it("projects collapsed sections without mutating stored geometry or rendering members", () => {
    const section: SpaceCanvasNode = {
      id: "section",
      type: "section",
      data: { kind: "section", title: "Group", collapsed: true },
      initialPosition: { x: 0, y: 0 },
    };
    const member: SpaceCanvasNode = {
      id: "member",
      type: "note",
      data: { ...createSpaceNotePayload("Member", ""), sectionId: "section" },
      initialPosition: { x: 40, y: 80 },
    };
    const document: CanvasDocumentV1 = {
      version: 1,
      viewport: { x: 0, y: 0, zoom: 1 },
      nodes: [
        { nodeId: "section", frame: frame(0, 0, 640, 420), sizeAuthority: "frame", zIndex: 0 },
        { nodeId: "member", frame: frame(40, 80, 300, 220), sizeAuthority: "frame", zIndex: 1 },
      ],
    };

    const scene = projectCanvasScene(document, [section, member], []);

    expect(scene.sections[0]?.layout.frame).toMatchObject({ width: 360, height: 36 });
    expect(scene.sections[0]?.members).toEqual([]);
    expect(scene.relations).toEqual([]);
    expect(document.nodes[0]?.frame).toMatchObject({ width: 640, height: 420 });
  });

  it("projects one identity for each semantic relation", () => {
    const left: SpaceCanvasNode = {
      id: "left",
      type: "note",
      data: createSpaceNotePayload("Left", ""),
      initialPosition: { x: 0, y: 0 },
    };
    const right: SpaceCanvasNode = {
      id: "right",
      type: "note",
      data: createSpaceNotePayload("Right", ""),
      initialPosition: { x: 400, y: 0 },
    };
    const document: CanvasDocumentV1 = {
      version: 1,
      viewport: { x: 0, y: 0, zoom: 1 },
      nodes: [
        { nodeId: "left", frame: frame(0, 0, 300, 220), sizeAuthority: "frame", zIndex: 0 },
        { nodeId: "right", frame: frame(400, 0, 300, 220), sizeAuthority: "frame", zIndex: 1 },
      ],
    };
    const relation = {
      id: "same-id",
      spaceId: "space",
      sourceId: "left",
      targetId: "right",
      kind: "related" as const,
      rationale: "",
      status: "active" as const,
      actorKind: "human" as const,
      createdAt: 1,
      updatedAt: 1,
    };

    expect(projectCanvasScene(document, [left, right], [relation]).relations.map(({ key }) => key)).toEqual([
      "relation:same-id",
    ]);
  });

  it("projects parallel relations onto stable distinct curve lanes", () => {
    const left: SpaceCanvasNode = {
      id: "left",
      type: "note",
      data: createSpaceNotePayload("Left", ""),
      initialPosition: { x: 0, y: 0 },
    };
    const right: SpaceCanvasNode = {
      id: "right",
      type: "note",
      data: createSpaceNotePayload("Right", ""),
      initialPosition: { x: 400, y: 0 },
    };
    const document: CanvasDocumentV1 = {
      version: 1,
      viewport: { x: 0, y: 0, zoom: 1 },
      nodes: [
        { nodeId: "left", frame: frame(0, 0, 300, 220), sizeAuthority: "frame", zIndex: 0 },
        { nodeId: "right", frame: frame(400, 0, 300, 220), sizeAuthority: "frame", zIndex: 1 },
      ],
    };
    const relation = {
      spaceId: "space",
      sourceId: "left",
      targetId: "right",
      rationale: "",
      status: "active" as const,
      actorKind: "human" as const,
      createdAt: 1,
      updatedAt: 1,
    };
    const projected = projectCanvasScene(
      document,
      [left, right],
      [
        { ...relation, id: "a", kind: "supports" },
        { ...relation, id: "b", kind: "depends-on" },
      ],
    ).relations;

    expect(projected[0]?.midpoint.y).not.toBe(projected[1]?.midpoint.y);
    expect(projected[0]?.points.slice(0, 2)).toEqual(projected[1]?.points.slice(0, 2));
    expect(projected[0]?.points.slice(-2)).toEqual(projected[1]?.points.slice(-2));
  });

  it("projects external relations from collapsed members onto their section boundary", () => {
    const section: SpaceCanvasNode = {
      id: "section",
      type: "section",
      data: { kind: "section", title: "Group", collapsed: true },
      initialPosition: { x: 0, y: 0 },
    };
    const member: SpaceCanvasNode = {
      id: "member",
      type: "note",
      data: { ...createSpaceNotePayload("Member", ""), sectionId: "section" },
      initialPosition: { x: 40, y: 80 },
    };
    const outside: SpaceCanvasNode = {
      id: "outside",
      type: "note",
      data: createSpaceNotePayload("Outside", ""),
      initialPosition: { x: 500, y: 0 },
    };
    const document: CanvasDocumentV1 = {
      version: 1,
      viewport: { x: 0, y: 0, zoom: 1 },
      nodes: [
        { nodeId: "section", frame: frame(0, 0, 640, 420), sizeAuthority: "frame", zIndex: 0 },
        { nodeId: "member", frame: frame(40, 80, 300, 220), sizeAuthority: "frame", zIndex: 1 },
        { nodeId: "outside", frame: frame(500, 0, 300, 220), sizeAuthority: "frame", zIndex: 2 },
      ],
    };
    const relation = {
      id: "external",
      spaceId: "space",
      sourceId: "member",
      targetId: "outside",
      kind: "supports" as const,
      rationale: "",
      status: "active" as const,
      actorKind: "human" as const,
      createdAt: 1,
      updatedAt: 1,
    };

    const [projected] = projectCanvasScene(document, [section, member, outside], [relation]).relations;
    expect(projected?.points.slice(0, 2)).toEqual([360, 18]);
  });

  it("keeps hidden layout data out of the rendered scene", () => {
    const hidden: SpaceCanvasNode = {
      id: "hidden",
      type: "note",
      data: createSpaceNotePayload("Hidden", ""),
      initialPosition: { x: 0, y: 0 },
      hidden: true,
    };
    const document: CanvasDocumentV1 = {
      version: 1,
      viewport: { x: 0, y: 0, zoom: 1 },
      nodes: [{ nodeId: "hidden", frame: frame(0, 0, 300, 220), sizeAuthority: "frame", zIndex: 0 }],
    };

    expect(projectCanvasScene(document, [hidden], []).nodes).toEqual([]);
    expect(document.nodes).toHaveLength(1);
  });
});
