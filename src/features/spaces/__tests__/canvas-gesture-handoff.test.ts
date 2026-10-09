import type Konva from "konva";
import { describe, expect, it, onTestFinished } from "vitest";
import { computed, effectScope, nextTick, shallowRef, watch } from "vue";

import type { CanvasHostEvents } from "../canvas/contracts/canvas-editor";
import { projectStructureCanvasDocument } from "../canvas/document/structure-canvas-document";
import { projectStructureScene } from "../canvas/document/structure-scene-projection";
import { CanvasDocumentModel } from "../canvas/konva/document/canvas-document-model";
import { createCanvasGestureSession } from "../canvas/konva/interaction/canvas-gesture-session";
import type { CanvasGestureUpdate } from "../canvas/konva/interaction/canvas-gesture-update";
import { createCanvasScenePreviewIndex } from "../canvas/konva/scene/scene-preview";
import { projectCanvasScene } from "../canvas/konva/scene/scene-projection";
import { useCanvasScenePreview } from "../canvas/konva/scene/use-canvas-scene-preview";
import { createSpaceNode } from "../canvas/node";
import type { SpaceEditCommand } from "../contracts/generated/SpaceEditCommand";
import { createSpaceNotePayload } from "../editor/tiptap-content";
import type { SpaceRelation } from "../model/relations/relation";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import { deferred } from "./support/canvas-fixtures";

describe("canvas gesture handoff", () => {
  it.each(["left", "right"])(
    "keeps the relation at the drop position while %s is being committed",
    async (id) => {
      const value = fixture(id);
      value.gestures.controller.beginDrag(id);
      value.gestures.beginDirectManipulation("drag");
      value.node.position({ x: 100, y: 180 });
      value.gestures.controller.moveDrag(id);
      await nextTick();
      const droppedPoints = value.points.value;
      value.rendered.length = 0;

      const finished = value.gestures.finish({ kind: "drag", nodeId: id });
      await value.started.promise;
      await nextTick();
      expect(value.preview.layouts.has(id)).toBe(true);
      expect(value.points.value).toEqual(droppedPoints);

      value.commit.resolve(undefined);
      await finished;
      await nextTick();
      expect(value.preview.layouts.size).toBe(0);
      expect(value.points.value).toEqual(droppedPoints);
      expect(value.rendered.length).toBeGreaterThan(0);
      for (const points of value.rendered) expect(points).toEqual(droppedPoints);
      expect(value.gestures.busy.value).toBe(false);
      expect(value.gestures.directManipulation.value).toBeUndefined();
    },
  );

  it("restores the node and relation together when the edit fails", async () => {
    const value = fixture("left");
    const originalPoints = value.points.value;
    value.gestures.controller.beginDrag("left");
    value.gestures.beginDirectManipulation("drag");
    value.node.position({ x: 100, y: 180 });
    value.gestures.controller.moveDrag("left");
    const finished = value.gestures.finish({ kind: "drag", nodeId: "left" });
    const error = new Error("edit rejected");
    await value.started.promise;
    value.commit.resolve(error);
    await expect(finished).rejects.toThrow(error);
    await nextTick();

    expect(value.preview.layouts.size).toBe(0);
    expect(value.node.position()).toEqual({ x: 0, y: 0 });
    expect(value.points.value).toEqual(originalPoints);
  });
});

function fixture(primaryId: string) {
  const scope = effectScope();
  onTestFinished(() => scope.stop());
  return scope.run(() => {
    const snapshot = shallowRef(initialSnapshot());
    const nodes = ["left", "right"].map((id) =>
      createSpaceNode(createSpaceNotePayload(id), { x: 0, y: 0 }, id),
    );
    const relations: SpaceRelation[] = [
      {
        id: "link",
        spaceId: "space",
        sourceId: "left",
        targetId: "right",
        kind: "related",
        rationale: "",
        status: "active",
        actorKind: "human",
        createdAt: 0,
        updatedAt: 0,
      },
    ];
    const document = new CanvasDocumentModel(projectStructureCanvasDocument(snapshot.value), {
      documentChanged() {},
      viewportChanged() {},
    });
    const scene = computed(() =>
      projectCanvasScene({
        document: document.sceneDocumentState.value,
        nodes,
        relations,
        structure: projectStructureScene({ snapshot: snapshot.value, nodes, relations }),
      }),
    );
    const layouts = shallowRef(new Map<string, CanvasGestureUpdate>());
    const preview = useCanvasScenePreview(
      computed(() => createCanvasScenePreviewIndex(scene.value)),
      layouts,
    );
    const points = computed(() => (preview.relations.get("link") ?? scene.value.relations[0]!).points);
    const rendered: (typeof points.value)[] = [];
    watch(points, (value) => rendered.push(value), { flush: "post" });
    const started = deferred<void>();
    const commit = deferred<Error | undefined>();
    const execute = async (command: SpaceEditCommand) => {
      started.resolve();
      const error = await commit.promise;
      if (error) throw error;
      if (command.kind !== "structure.move") throw new Error("Expected move command");
      snapshot.value = {
        ...snapshot.value,
        layouts: snapshot.value.layouts.map((layout) =>
          layout.kind === "content" && command.rootIds.includes(layout.structureNodeId)
            ? {
                ...layout,
                transform: {
                  ...layout.transform,
                  x: layout.transform.x + command.delta.x,
                  y: layout.transform.y + command.delta.y,
                },
              }
            : layout,
        ),
      };
    };
    const frame = document.document.nodes.find(({ nodeId }) => nodeId === primaryId)!.frame;
    let point = { x: frame.x, y: frame.y };
    const node = {
      getAttr: () => primaryId,
      x: () => point.x,
      y: () => point.y,
      position: (next?: typeof point) => (next ? (point = next) : point),
      rotation() {},
      scale() {},
    } as unknown as Konva.Node;
    const gestures = createCanvasGestureSession({
      document,
      events: { onExecuteSpaceEdit: execute, onMutationPendingChange() {} } as unknown as CanvasHostEvents,
      layouts: () => document.document.nodes,
      nodes: () => nodes,
      previewFrames: (updates) => {
        layouts.value = new Map(updates.map((update) => [update.nodeId, update]));
      },
      lockedNodeIds: () => new Set(),
      selectedNodeIds: () => new Set([primaryId]),
      snapshot: () => snapshot.value,
      temporarilyExpandedSectionId: () => undefined,
      stage: () => ({ find: () => [node] }) as unknown as Konva.Stage,
      zoom: () => 1,
    });
    return { gestures, node, points, preview, rendered, started, commit };
  })!;
}

function initialSnapshot(): SpaceEditSnapshot {
  return {
    editRevision: 1,
    relationStyles: [],
    structure: ["left", "right"].map((nodeId, order) => ({
      id: nodeId,
      kind: "content",
      parentId: null,
      order,
      locked: false,
      nodeId,
    })),
    layouts: ["left", "right"].map((structureNodeId, index) => ({
      structureNodeId,
      kind: "content",
      transform: { x: index * 500, y: 0, rotation: 0, scale: 1 },
      size: { width: 240, height: 120 },
      sizingMode: "frame",
    })),
  };
}
