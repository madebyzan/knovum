import type Konva from "konva";
import { onTestFinished } from "vitest";
import { computed, effectScope, shallowRef, watch } from "vue";

import { projectStructureCanvasDocument } from "../../canvas/document/structure-canvas-document";
import { projectStructureScene } from "../../canvas/document/structure-scene-projection";
import { projectStructureGesture } from "../../canvas/konva/interaction/structure-gesture-preview";
import { useStructureGestureSession } from "../../canvas/konva/interaction/use-structure-gesture-session";
import { projectCanvasScene } from "../../canvas/konva/scene/scene-projection";
import { createSpaceNode } from "../../canvas/node";
import type { SpaceEditCommand } from "../../contracts/generated/SpaceEditCommand";
import { createSpaceNotePayload } from "../../document-editing/tiptap-content";
import type { SpaceRelation } from "../../model/relations/relation";
import type { SpaceEditSnapshot } from "../../model/structure/schema/edit-snapshot";
import { deferred } from "./canvas-fixtures";

export function structureGestureFixture() {
  const scope = effectScope();
  onTestFinished(() => scope.stop());
  return scope.run(() => {
    const snapshot = shallowRef(initialSnapshot());
    const preview = shallowRef<SpaceEditSnapshot>();
    const document = computed(() => projectStructureCanvasDocument(preview.value ?? snapshot.value));
    const nodes = ["note", "outside"].map((id) =>
      createSpaceNode(createSpaceNotePayload(id), { x: 0, y: 0 }, id),
    );
    const relations: SpaceRelation[] = [
      {
        id: "link",
        spaceId: "space",
        sourceId: "note",
        targetId: "outside",
        kind: "related",
        rationale: "",
        status: "active",
        actorKind: "human",
        createdAt: 0,
        updatedAt: 0,
      },
    ];
    const scene = computed(() =>
      projectCanvasScene({
        document: document.value,
        nodes,
        relations,
        structure: projectStructureScene({ snapshot: preview.value ?? snapshot.value, nodes, relations }),
      }),
    );
    const points = computed(() => scene.value.relations[0]!.points);
    const renderedPoints: (typeof points.value)[] = [];
    watch(points, (value) => renderedPoints.push(value), { flush: "post" });
    const rendered: (typeof document.value)[] = [];
    watch(document, (value) => rendered.push(value), { flush: "post" });
    const started = deferred<void>();
    const commit = deferred<Error | undefined>();
    const commands: SpaceEditCommand[] = [];
    const execute = async (command: SpaceEditCommand) => {
      commands.push(command);
      started.resolve();
      const error = await commit.promise;
      if (error) throw error;
      if (command.kind !== "structure.move") throw new Error("Expected a move command");
      snapshot.value = projectStructureGesture(snapshot.value, command);
    };
    let point = { x: 0, y: 0 };
    let pointer = { x: 0, y: 0 };
    const node = {
      getParent: () => ({ getRelativePointerPosition: () => pointer }),
      x: () => point.x,
      y: () => point.y,
      position: (next?: typeof point) => (next ? (point = next) : point),
      stopDrag: () => {
        void gestures.finish();
      },
    } as unknown as Konva.Node;
    const gestures = useStructureGestureSession({
      snapshot: () => snapshot.value,
      preview,
      selectedRootIds: () => ["section"],
      bounds: () =>
        new Map([
          ["section", { x: 0, y: 0, width: 400, height: 300 }],
          ["content", { x: 40, y: 60, width: 100, height: 80 }],
        ]),
      execute,
      previewDrop() {},
      expandedSectionId: () => undefined,
      zoom: () => 1,
    });
    return {
      setPointer: (next: typeof pointer) => (pointer = next),
      gestures,
      node,
      document,
      preview,
      rendered,
      started,
      commit,
      commands,
      points,
      renderedPoints,
    };
  })!;
}

function initialSnapshot(): SpaceEditSnapshot {
  return {
    editRevision: 1,
    relationStyles: [],
    structure: [
      { id: "section", kind: "section", parentId: null, order: 0, locked: false, title: "Sources" },
      { id: "content", kind: "content", parentId: "section", order: 0, locked: false, nodeId: "note" },
      { id: "outside", kind: "content", parentId: null, order: 1, locked: false, nodeId: "outside" },
    ],
    layouts: [
      {
        structureNodeId: "section",
        kind: "section",
        position: { x: 0, y: 0 },
        size: { width: 400, height: 300 },
        collapsed: false,
      },
      {
        structureNodeId: "content",
        kind: "content",
        transform: { x: 40, y: 60, rotation: 0, scale: 1 },
        size: { width: 100, height: 80 },
        sizingMode: "frame",
      },
      {
        structureNodeId: "outside",
        kind: "content",
        transform: { x: 700, y: 200, rotation: 0, scale: 1 },
        size: { width: 100, height: 80 },
        sizingMode: "frame",
      },
    ],
  };
}
