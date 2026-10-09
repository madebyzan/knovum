import { expect, it } from "vitest";
import { effectScope, shallowRef } from "vue";

import type { ProjectedSection } from "../canvas/document/structure-scene";
import type { CanvasScene } from "../canvas/konva/scene/canvas-scene";
import { useSceneVisibility } from "../canvas/konva/scene/use-scene-visibility";

function section(id: string, top: number, collapsed = false): ProjectedSection {
  return {
    section: { kind: "section", id, title: id, parentId: null, order: 0, locked: false },
    layout: {
      kind: "section",
      structureNodeId: id,
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

it("culls collapsed section bodies while retaining selected and temporarily expanded sections", () => {
  const scope = effectScope();
  const expanded = { ...section("expanded", 2000, true), temporarilyExpanded: true };
  const selected = shallowRef<ReadonlySet<string>>(new Set(["selected"]));
  const visibility = scope.run(() =>
    useSceneVisibility({
      scene: shallowRef<CanvasScene>({
        nodes: [],
        nodesById: new Map(),
        relations: [],
        sectionBackdrops: [],
      }),
      nodeIds: shallowRef(new Set<string>()),
      relationIds: shallowRef(new Set<string>()),
      sections: shallowRef([
        section("visible", 50),
        section("collapsed", -500, true),
        section("selected", 1000),
        expanded,
      ]),
      sectionIds: selected,
      viewport: shallowRef({ x: 0, y: 0, zoom: 1 }),
      size: { width: 500, height: 400 },
    }),
  )!;
  try {
    expect(visibility.visibleSections.value.map(({ section: value }) => value.id)).toEqual([
      "visible",
      "selected",
      "expanded",
    ]);
    selected.value = new Set();
    expect(visibility.visibleSections.value.map(({ section: value }) => value.id)).toEqual([
      "visible",
      "expanded",
    ]);
  } finally {
    scope.stop();
  }
});
