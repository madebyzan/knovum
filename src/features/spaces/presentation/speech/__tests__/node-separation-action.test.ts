import { flushPromises, mount } from "@vue/test-utils";
import { expect, it, onTestFinished, vi } from "vitest";
import { defineComponent, h, shallowRef } from "vue";

import { i18n } from "../../../../../foundation/i18n/i18n";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { useSourceSeparationCandidate } from "../../../speech/use-source-separation-candidate";
import SpaceNodeSpeechActions from "../SpaceNodeSpeechActions.vue";
import { useNodeSeparation } from "../use-node-separation";

const mocks = vi.hoisted(() => ({
  store: { activeSpaceId: "space", nodes: [] as SpaceCanvasNode[] },
  separation: vi.fn<typeof useSourceSeparationCandidate>(),
}));
vi.mock("../../../session/space-store", () => ({ useSpaceStore: () => mocks.store }));
vi.mock("../../../speech/use-source-separation-candidate", () => ({
  useSourceSeparationCandidate: mocks.separation,
}));
const revision = `sha256:${"a".repeat(64)}`;
const node: SpaceCanvasNode = {
  id: "source",
  type: "audio",
  initialPosition: { x: 0, y: 0 },
  data: {
    kind: "audio",
    title: "Interview",
    assetPath: "/source.wav",
    fileName: "source.wav",
    sourceRevision: revision,
    mediaAnalysis: { schemaVersion: 1, sourceRevision: revision, durationMs: 30_000, waveform: [] },
  },
};
function render() {
  mocks.store.nodes = [node];
  type Separation = ReturnType<typeof useSourceSeparationCandidate>;
  const separation: Separation = {
    candidate: shallowRef(undefined),
    activeRunId: shallowRef(undefined),
    busy: shallowRef(false),
    error: shallowRef(undefined),
    restore: vi.fn<Separation["restore"]>().mockResolvedValue(),
    start: vi.fn<Separation["start"]>().mockResolvedValue(),
    cancel: vi.fn<Separation["cancel"]>().mockResolvedValue(),
    review: vi.fn<Separation["review"]>().mockResolvedValue(),
    reset: vi.fn<Separation["reset"]>(),
  };
  mocks.separation.mockReturnValue(separation);
  const Operation = defineComponent({
    setup() {
      const { invoke } = useNodeSeparation(shallowRef(node));
      return () => h(SpaceNodeSpeechActions, { node, onSeparate: invoke });
    },
  });
  const wrapper = mount(Operation, {
    global: { plugins: [i18n], stubs: { ToolbarButton: { template: "<slot />" } } },
  });
  onTestFinished(() => wrapper.unmount());
  return { wrapper, separation };
}
it("starts source separation with one toolbar click and opens no intermediate editor", async () => {
  const { wrapper, separation } = render();
  await flushPromises();
  await wrapper.get('[data-operation="separate"]').trigger("click");
  await flushPromises();
  expect(separation.start).toHaveBeenCalledWith("space", {
    assetPath: "/source.wav",
    sourceRevision: revision,
    durationMs: 30_000,
  });
  expect(wrapper.getComponent(SpaceNodeSpeechActions).emitted("begin")).toBeUndefined();
  expect(wrapper.find('[role="dialog"]').exists()).toBe(false);
});
it("uses the same operation to cancel the currently running separation", async () => {
  const { wrapper, separation } = render();
  (separation.activeRunId as ReturnType<typeof shallowRef<string | undefined>>).value = "running";
  await wrapper.get('[data-operation="separate"]').trigger("click");
  await flushPromises();
  expect(separation.cancel).toHaveBeenCalledOnce();
  expect(separation.start).not.toHaveBeenCalled();
});
