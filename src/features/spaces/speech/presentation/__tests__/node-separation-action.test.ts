import { flushPromises, mount } from "@vue/test-utils";
import { ToolbarRoot } from "reka-ui";
import { expect, it, onTestFinished, vi } from "vitest";
import { defineComponent, h, shallowRef } from "vue";

import { i18n } from "../../../../../foundation/i18n/instance";
import type { RunObservation } from "../../../contracts/generated/RunObservation";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { useSourceSeparation } from "../../use-source-separation";
import type { NodeSpeechOperation } from "../node-speech-operations";
import SpaceNodeSeparationBar from "../SpaceNodeSeparationBar.vue";
import SpaceNodeSpeechActions from "../SpaceNodeSpeechActions.vue";
import SpaceNodeSpeechControls from "../SpaceNodeSpeechControls.vue";
import { useNodeSeparation } from "../use-node-separation";

const mocks = vi.hoisted(() => ({
  store: { activeSpaceId: "space", nodes: [] as SpaceCanvasNode[] },
  separation: vi.fn<typeof useSourceSeparation>(),
}));
vi.mock("../../../session/space-store", () => ({ useSpaceStore: () => mocks.store }));
vi.mock("../../../../../foundation/ai/ai-readiness-admission", () => ({
  admitAiRequirements: () => Promise.resolve(true),
}));
vi.mock("../../use-source-separation", () => ({
  useSourceSeparation: mocks.separation,
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
function render(nodes = [node]) {
  mocks.store.nodes = nodes;
  type Separation = ReturnType<typeof useSourceSeparation>;
  const state = {
    activeRunId: shallowRef<string>(),
    runState: shallowRef<RunObservation["state"]>(),
    isPreparing: shallowRef(false),
    isCancelling: shallowRef(false),
    isBusy: shallowRef(false),
    error: shallowRef<string>(),
  };
  const separation: Separation = {
    ...state,
    restore: vi.fn<Separation["restore"]>().mockResolvedValue(),
    start: vi.fn<Separation["start"]>().mockResolvedValue(),
    cancel: vi.fn<Separation["cancel"]>().mockResolvedValue(),
    reset: vi.fn<Separation["reset"]>(),
  };
  mocks.separation.mockReturnValue(separation);
  const Operation = defineComponent({
    setup() {
      const { unavailableReason, startSeparation } = useNodeSeparation(shallowRef(node));
      const operation = shallowRef<NodeSpeechOperation>();
      return () => [
        h(ToolbarRoot, () =>
          h(SpaceNodeSpeechActions, {
            node,
            separating: Boolean(separation.activeRunId.value),
            separationBusy: separation.isBusy.value,
            separationUnavailableReason: unavailableReason.value,
            onSeparate: () => {
              operation.value = "separate";
              if (!separation.activeRunId.value) void startSeparation();
            },
          }),
        ),
        h(
          SpaceNodeSpeechControls,
          {
            node,
            operation: operation.value,
            "onUpdate:operation": (next: NodeSpeechOperation | undefined) => {
              operation.value = next;
            },
          },
          {
            separation: ({ close }: { close: () => void }) =>
              h(SpaceNodeSeparationBar, { separation, onClose: close }),
          },
        ),
      ];
    },
  });
  const wrapper = mount(Operation, {
    global: { plugins: [i18n] },
  });
  onTestFinished(() => {
    wrapper.unmount();
    document.body.innerHTML = "";
  });
  return { wrapper, separation, state };
}
it("starts separation immediately and exposes its task in a nonmodal contextual bar", async () => {
  const { wrapper, separation } = render();
  await flushPromises();
  await wrapper.get('[data-operation="separate"]').trigger("click");
  await flushPromises();
  expect(separation.start).toHaveBeenCalledWith("space", "source", {
    assetPath: "/source.wav",
    sourceRevision: revision,
    durationMs: 30_000,
  });
  expect(wrapper.getComponent(SpaceNodeSpeechActions).emitted("begin")).toBeUndefined();
  expect(wrapper.find('[role="dialog"]').exists()).toBe(false);
});
it("opens a running separation without cancelling it and uses an explicit cancel action", async () => {
  const { wrapper, separation, state } = render();
  state.activeRunId.value = "running";
  await wrapper.get('[data-operation="separate"]').trigger("click");
  await flushPromises();
  expect(separation.cancel).not.toHaveBeenCalled();
  expect(separation.start).not.toHaveBeenCalled();
  await wrapper.get('[data-testid="cancel-separation"]').trigger("click");
  expect(separation.cancel).toHaveBeenCalledOnce();
  state.isCancelling.value = true;
  await flushPromises();
  expect(wrapper.get('[data-testid="cancel-separation"]').attributes("disabled")).toBeDefined();
});

it("closes task controls without cancelling the background task", async () => {
  const { wrapper, separation, state } = render();
  state.activeRunId.value = "running";
  await wrapper.get('[data-operation="separate"]').trigger("click");
  await flushPromises();
  await wrapper.get('[data-testid="node-speech-controls"]').trigger("keydown", { key: "Escape" });
  expect(wrapper.find('[data-testid="node-separation-controls"]').exists()).toBe(false);
  expect(separation.cancel).not.toHaveBeenCalled();
});

it("reflects automatic publication without another confirmation or repeated failure message", async () => {
  const { wrapper, state } = render();
  state.activeRunId.value = "running";
  await wrapper.get('[data-operation="separate"]').trigger("click");
  await flushPromises();
  state.activeRunId.value = undefined;
  state.runState.value = "succeeded";
  await flushPromises();
  expect(wrapper.get('[role="status"]').text()).toBe(i18n.global.t("spaces.speech.separation.completed"));
  expect(wrapper.find('[data-testid="retry-separation"]').exists()).toBe(false);
  state.runState.value = "failed";
  state.error.value = i18n.global.t("spaces.runs.states.failed");
  await flushPromises();
  expect(wrapper.findAll('[role="alert"]')).toHaveLength(1);
});

it("explains an unavailable source instead of leaving a silent disabled action", async () => {
  const { wrapper } = render([]);
  await flushPromises();
  const action = wrapper.get('[data-operation="separate"]');
  expect(action.attributes("disabled")).toBeDefined();
  expect(action.attributes("title")).toBe(i18n.global.t("spaces.speech.separation.sourceUnavailable"));
});
