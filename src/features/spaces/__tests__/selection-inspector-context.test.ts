import { shallowMount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { expect, it, onTestFinished, vi } from "vitest";

import { i18n } from "../../../foundation/i18n/instance";
import { getCanvasEditorSession, updateCanvasSelectionProjection } from "../canvas/editor-session";
import { createSpaceNode } from "../canvas/node";
import { createSpaceNotePayload } from "../document-editing/tiptap-content";
import SpaceRelationInspector from "../inspection/presentation/SpaceRelationInspector.vue";
import SpaceSelectionInspector from "../inspection/presentation/SpaceSelectionInspector.vue";
import SpaceKnowledgeHealth from "../knowledge/presentation/SpaceKnowledgeHealth.vue";
import { useSpaceStore } from "../session/space-store";

vi.mock("../enrichment/use-enrichment-job-observation", () => ({ useEnrichmentJobObservation: () => {} }));

it("inspects relation and mixed selections instead of showing Space health", async () => {
  const pinia = createPinia();
  setActivePinia(pinia);
  const store = useSpaceStore();
  const previous = getCanvasEditorSession().selection.value;
  onTestFinished(() => updateCanvasSelectionProjection(previous));
  store.nodes = ["a", "b"].map((id) => createSpaceNode(createSpaceNotePayload(id), { x: 0, y: 0 }, id));
  store.relations = [
    {
      id: "relation",
      spaceId: "inbox",
      sourceId: "a",
      targetId: "b",
      kind: "supports",
      rationale: "Evidence",
      status: "active",
      actorKind: "human",
      createdAt: 1,
      updatedAt: 1,
    },
  ];
  const wrapper = shallowMount(SpaceSelectionInspector, { global: { plugins: [pinia, i18n] } });
  onTestFinished(() => wrapper.unmount());
  expect(wrapper.findComponent(SpaceKnowledgeHealth).exists()).toBe(true);
  updateCanvasSelectionProjection({
    nodeIds: [],
    structureNodeIds: [],
    relationIds: ["relation"],
    nodePlacements: [],
  });
  await wrapper.vm.$nextTick();
  expect(wrapper.findComponent(SpaceKnowledgeHealth).exists()).toBe(false);
  expect(wrapper.findComponent(SpaceRelationInspector).props("relations")).toEqual(store.relations);
  updateCanvasSelectionProjection({ ...getCanvasEditorSession().selection.value, nodeIds: ["a"] });
  await wrapper.vm.$nextTick();
  expect(wrapper.findComponent(SpaceRelationInspector).props("objectTitles")).toEqual(["a"]);
});
