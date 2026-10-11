import { describe, expect, it, onTestFinished, vi } from "vitest";
import { effectScope, shallowRef } from "vue";

import { projectStructureCanvasDocument } from "../canvas/document/structure-canvas-document";
import { projectStructureScene } from "../canvas/document/structure-scene-projection";
import { useSectionDropPreview } from "../canvas/konva/use-section-drop-preview";
import { createSpaceNode } from "../canvas/node";
import { createSpaceNotePayload } from "../document-editing/tiptap-content";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import {
  sectionDropMoveCommand,
  sectionDropPreview,
  sectionDropReparentCommand,
} from "../structure/section-drop-reparent";

describe("Structure Section projection", () => {
  it("keeps folded descendants in the adapter document while hiding only the scene projection", () => {
    const snapshot = collapsedSectionSnapshot();
    const node = createSpaceNode(createSpaceNotePayload("Evidence"), { x: 0, y: 0 }, "node");

    expect(projectStructureCanvasDocument(snapshot).nodes.map(({ nodeId }) => nodeId)).toEqual(["node"]);
    expect(projectStructureScene({ snapshot, nodes: [node], relations: [] }).content[0]?.visible).toBe(false);
    expect(
      projectStructureScene({
        snapshot,
        nodes: [node],
        relations: [],
        temporarilyExpandedSectionIds: new Set(["section"]),
      }).content[0]?.visible,
    ).toBe(true);
  });

  it("accepts the visible collapsed header immediately and expands the committed destination", () => {
    const snapshot = dropSnapshot();
    const delta = { x: -350, y: -100 };
    const drop = { snapshot, rootIds: ["free"], delta };

    expect(sectionDropPreview(drop)).toEqual({
      sectionId: "section",
      kind: "enter",
    });
    expect(sectionDropMoveCommand(drop)).toMatchObject({
      kind: "structure.move-to-container",
      parentId: "section",
      expandSectionId: "section",
    });
    expect(
      sectionDropReparentCommand(drop, {
        allowCollapsedSectionId: "section",
      }),
    ).toEqual({ kind: "structure.reparent", rootIds: ["free"], parentId: "section", order: 0 });
  });

  it("uses the moving selection center so entry and exit agree at a Section edge", () => {
    const snapshot = dropSnapshot();
    const sectionLayout = snapshot.layouts[0]!;
    if (sectionLayout.kind !== "section") throw new Error("Expected Section");
    sectionLayout.collapsed = false;
    const drop = { snapshot, rootIds: ["free"], delta: { x: -150, y: 0 } };
    expect(sectionDropReparentCommand(drop)?.parentId).toBe("section");
    const memberSnapshot = dropSnapshot();
    memberSnapshot.structure[1]!.parentId = "section";
    memberSnapshot.layouts[0] = sectionLayout;
    expect(sectionDropReparentCommand({ ...drop, snapshot: memberSnapshot })).toBeUndefined();
  });

  it("does not offer hidden body space as a target for a collapsed Section", () => {
    expect(
      sectionDropPreview({
        snapshot: dropSnapshot(),
        rootIds: ["free"],
        delta: { x: -350, y: 0 },
      }),
    ).toBeUndefined();
  });

  it("shows entry immediately, expands after dwell, and commits the same destination", () => {
    vi.useFakeTimers();
    const scope = effectScope();
    onTestFinished(() => {
      scope.stop();
      vi.useRealTimers();
    });
    const snapshot = dropSnapshot();
    const expanded = shallowRef<string>();
    const preview = scope.run(() =>
      useSectionDropPreview({
        snapshot: () => snapshot,
        temporarilyExpandedSectionId: expanded,
      }),
    )!;
    preview.update(["free"], { x: -350, y: -100 });
    expect(preview.state.value).toEqual({ sectionId: "section", kind: "enter" });
    expect(expanded.value).toBeUndefined();
    vi.advanceTimersByTime(600);
    expect(expanded.value).toBe("section");
    const delta = { x: -350, y: 0 };
    preview.update(["free"], delta);
    expect(preview.state.value?.kind).toBe("enter");
    expect(sectionDropMoveCommand({ snapshot, rootIds: ["free"], delta }, expanded.value)).toMatchObject({
      kind: "structure.move-to-container",
      parentId: "section",
      expandSectionId: "section",
    });
    preview.clear();
    expect(preview.state.value).toBeUndefined();
    expect(expanded.value).toBeUndefined();
  });

  it("previews leaving a Section and releases membership without altering the world move", () => {
    const snapshot = collapsedSectionSnapshot();
    const layout = snapshot.layouts[0]!;
    if (layout.kind !== "section") throw new Error("Expected Section");
    layout.collapsed = false;
    const drop = { snapshot, rootIds: ["content"], delta: { x: 400, y: 0 } };
    expect(sectionDropPreview(drop)).toEqual({ sectionId: "section", kind: "leave" });
    expect(sectionDropMoveCommand(drop)).toMatchObject({
      kind: "structure.move-to-container",
      rootIds: ["content"],
      parentId: null,
      delta: drop.delta,
    });
  });
});

function collapsedSectionSnapshot(): SpaceEditSnapshot {
  return {
    editRevision: 1,
    relationStyles: [],
    structure: [
      { id: "section", kind: "section", parentId: null, order: 0, locked: false, title: "Sources" },
      { id: "content", kind: "content", parentId: "section", order: 0, locked: false, nodeId: "node" },
    ],
    layouts: [
      {
        structureNodeId: "section",
        kind: "section",
        position: { x: 0, y: 0 },
        size: { width: 400, height: 300 },
        collapsed: true,
      },
      {
        structureNodeId: "content",
        kind: "content",
        transform: { x: 40, y: 70, rotation: 0, scale: 1 },
        size: { width: 100, height: 80 },
        sizingMode: "frame",
      },
    ],
  };
}

function dropSnapshot(): SpaceEditSnapshot {
  const snapshot = collapsedSectionSnapshot();
  return {
    ...snapshot,
    structure: [
      snapshot.structure[0]!,
      { id: "free", kind: "content", parentId: null, order: 1, locked: false, nodeId: "free-node" },
    ],
    layouts: [
      snapshot.layouts[0]!,
      {
        structureNodeId: "free",
        kind: "content",
        transform: { x: 500, y: 100, rotation: 0, scale: 1 },
        size: { width: 80, height: 60 },
        sizingMode: "frame",
      },
    ],
  };
}
