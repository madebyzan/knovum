import { expect, it } from "vitest";
import { effectScope, shallowRef } from "vue";

import type { ProjectedSection } from "../canvas/document/structure-scene";
import type { CanvasScene } from "../canvas/konva/scene/canvas-scene";
import { NODE_CONTENT_MIN_ZOOM } from "../canvas/konva/scene/content-legibility";
import { projectCanvasScene } from "../canvas/konva/scene/scene-projection";
import { useSceneVisibility } from "../canvas/konva/scene/use-scene-visibility";
import { createCanvasCamera } from "../canvas/konva/viewport/canvas-camera";
import { createSpaceNode } from "../canvas/node";
import { createSpaceTextPayload } from "../document-editing/tiptap-content";
import type { SpaceRelation } from "../model/relations/relation";

function section(sectionId: string, top: number, collapsed = false): ProjectedSection {
  return {
    section: { kind: "section", id: sectionId, title: sectionId, parentId: null, order: 0, locked: false },
    layout: {
      kind: "section",
      structureNodeId: sectionId,
      position: { x: 0, y: top },
      size: { width: 200, height: 1000 },
      collapsed,
    },
    bounds: { left: 0, right: 200, top, bottom: top + 1000 },
    contentCount: 0,
    locked: false,
    temporarilyExpanded: false,
  };
}

const EMPTY_SCENE: CanvasScene = {
  nodes: [],
  nodesById: new Map(),
  relations: [],
  relationKeyByRelationId: new Map(),
  sectionBackdrops: [],
};

/** Runs the visibility of `scene` in its own effect scope, with nothing selected unless overridden. */
function withVisibility(
  overrides: Partial<Parameters<typeof useSceneVisibility>[0]>,
  check: (visibility: ReturnType<typeof useSceneVisibility>) => void,
) {
  const scope = effectScope();
  const visibility = scope.run(() =>
    useSceneVisibility({
      activeNodeId: shallowRef<string>(),
      scene: shallowRef(EMPTY_SCENE),
      nodeIds: shallowRef(new Set<string>()),
      relationIds: shallowRef(new Set<string>()),
      revealedRelationIds: shallowRef(new Set<string>()),
      sections: shallowRef([]),
      sectionIds: shallowRef(new Set<string>()),
      camera: createCanvasCamera({ x: 0, y: 0, zoom: 1 }),
      size: { width: 500, height: 400 },
      ...overrides,
    }),
  )!;
  try {
    check(visibility);
  } finally {
    scope.stop();
  }
}

/** Text nodes laid out left to right, `spacing` apart. */
function sceneOf(nodeIds: readonly string[], relations: readonly SpaceRelation[] = [], spacing = 150) {
  return projectCanvasScene({
    nodes: nodeIds.map((id) => createSpaceNode(createSpaceTextPayload(id), { x: 0, y: 0 }, id)),
    relations,
    document: {
      version: 1,
      viewport: { x: 0, y: 0, zoom: 1 },
      nodes: nodeIds.map((nodeId, index) => ({
        nodeId,
        frame: { x: index * spacing, y: 0, width: 100, height: 100, rotation: 0 },
        sizingMode: "frame",
        zIndex: index,
      })),
    },
  });
}

function relation(
  id: string,
  sourceId: string,
  targetId: string,
  kind: SpaceRelation["kind"],
): SpaceRelation {
  return {
    id,
    spaceId: "space",
    sourceId,
    targetId,
    kind,
    rationale: "",
    status: "active",
    actorKind: "system",
    createdAt: 0,
    updatedAt: 0,
  };
}

it("culls collapsed section bodies while retaining selected and temporarily expanded sections", () => {
  const expanded = { ...section("expanded", 2000, true), temporarilyExpanded: true };
  const sectionIds = shallowRef<ReadonlySet<string>>(new Set(["selected"]));
  const sections = shallowRef([
    section("visible", 50),
    section("collapsed", -500, true),
    section("selected", 1000),
    expanded,
  ]);
  withVisibility({ sections, sectionIds }, (visibility) => {
    expect(visibility.visibleSections.value.map(({ section: value }) => value.id)).toEqual([
      "visible",
      "selected",
      "expanded",
    ]);
    sectionIds.value = new Set();
    expect(visibility.visibleSections.value.map(({ section: value }) => value.id)).toEqual([
      "visible",
      "expanded",
    ]);
  });
});

it("keeps offscreen selection geometry without mounting its content and retains the active editor", () => {
  const activeNodeId = shallowRef<string>();
  const scene = sceneOf(["onscreen", "offscreen"], [], 2000);
  const nodeIds = shallowRef<ReadonlySet<string>>(new Set(["offscreen"]));
  withVisibility({ activeNodeId, nodeIds, scene: shallowRef(scene) }, (visibility) => {
    expect([...visibility.visibleNodeIds.value]).toEqual(["onscreen", "offscreen"]);
    expect([...visibility.visibleContentNodeIds.value]).toEqual(["onscreen"]);
    activeNodeId.value = "offscreen";
    expect([...visibility.visibleContentNodeIds.value]).toEqual(["onscreen", "offscreen"]);
    activeNodeId.value = undefined;
    expect([...visibility.visibleContentNodeIds.value]).toEqual(["onscreen"]);
  });
});

it("draws lineage only while the selection explains it or it is selected itself", () => {
  const scene = sceneOf(
    ["source", "output", "note"],
    [
      relation("lineage", "output", "source", "derives-from"),
      relation("meaning", "note", "source", "supports"),
    ],
  );
  const revealedRelationIds = shallowRef<ReadonlySet<string>>(new Set());
  const relationIds = shallowRef<ReadonlySet<string>>(new Set());
  withVisibility({ scene: shallowRef(scene), revealedRelationIds, relationIds }, (visibility) => {
    expect([...visibility.visibleRelationIds.value]).toEqual(["relation:meaning"]);

    revealedRelationIds.value = new Set(["lineage"]);
    expect(visibility.visibleRelationIds.value).toEqual(new Set(["relation:lineage", "relation:meaning"]));

    revealedRelationIds.value = new Set();
    relationIds.value = new Set(["lineage"]);
    expect(visibility.visibleRelationIds.value).toEqual(new Set(["relation:lineage", "relation:meaning"]));
    expect([...visibility.selectedRelationIds.value]).toEqual(["relation:lineage"]);
  });
});

it("mounts content only where it is legible, keeping the node being edited", () => {
  const activeNodeId = shallowRef<string>();
  const scene = shallowRef(sceneOf(["first", "second"]));
  const camera = createCanvasCamera({ x: 0, y: 0, zoom: NODE_CONTENT_MIN_ZOOM / 2 });
  withVisibility({ activeNodeId, camera, scene }, (visibility) => {
    expect([...visibility.visibleNodeIds.value]).toEqual(["first", "second"]);
    expect([...visibility.visibleContentNodeIds.value]).toEqual([]);
    activeNodeId.value = "second";
    expect([...visibility.visibleContentNodeIds.value]).toEqual(["second"]);
  });
  withVisibility(
    { scene, camera: createCanvasCamera({ x: 0, y: 0, zoom: NODE_CONTENT_MIN_ZOOM }) },
    (visibility) => {
      expect([...visibility.visibleContentNodeIds.value]).toEqual(["first", "second"]);
    },
  );
});
