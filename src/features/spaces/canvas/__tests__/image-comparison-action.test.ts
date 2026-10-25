import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { ToolbarRoot } from "reka-ui";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { defineComponent, h } from "vue";

import type { prepareDerivationComparison } from "../../knowledge-derivations/comparison/comparison-commands";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import SpaceCanvasImageComparisonAction from "../presentation/images/SpaceCanvasImageComparisonAction.vue";

const mocks = vi.hoisted(() => ({
  comparable: vi.fn<(nodeId: string) => Promise<boolean>>(),
  prepare: vi.fn<typeof prepareDerivationComparison>(),
}));
vi.mock("../../knowledge-derivations/comparison/comparison-commands", () => ({
  isDerivationComparable: mocks.comparable,
  prepareDerivationComparison: mocks.prepare,
}));
vi.mock("@tauri-apps/api/core", () => ({ convertFileSrc: (path: string) => `asset://localhost${path}` }));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

function image(id: string, sourceRevision: string): SpaceCanvasNode<SpaceImagePayload> {
  return {
    id,
    type: "image",
    initialPosition: { x: 0, y: 0 },
    data: { kind: "image", title: id, fileName: `${id}.png`, assetPath: `/${id}.png`, sourceRevision },
  };
}
const output = image("output", "asset:output");

const Viewer = defineComponent({
  name: "SpaceImageComparisonViewer",
  props: ["open", "before", "after", "error"],
  setup: () => () => h("div"),
});

function render() {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const wrapper = mount(
    defineComponent({
      setup: () => () => h(ToolbarRoot, () => h(SpaceCanvasImageComparisonAction, { node: output })),
    }),
    {
      global: {
        plugins: [[VueQueryPlugin, { queryClient }]],
        stubs: {
          SpaceImageComparisonViewer: Viewer,
          UiTooltip: { props: ["text", "disabled", "side"], template: "<slot />" },
        },
      },
    },
  );
  onTestFinished(() => {
    wrapper.unmount();
    queryClient.clear();
  });
  return wrapper;
}

beforeEach(() => {
  vi.clearAllMocks();
  mocks.prepare.mockResolvedValue({ path: "/cache/input.png", revision: "rendered1:input" });
});

it("offers nothing on an image whose derivation does not lay over its source", async () => {
  mocks.comparable.mockResolvedValue(false);
  const wrapper = render();
  await flushPromises();
  expect(mocks.comparable).toHaveBeenCalledWith("output");
  expect(wrapper.find("button").exists()).toBe(false);
});

it("prepares the input only once the comparison is opened", async () => {
  mocks.comparable.mockResolvedValue(true);
  const wrapper = render();
  await flushPromises();
  expect(mocks.prepare).not.toHaveBeenCalled();

  await wrapper.get("button").trigger("click");
  await flushPromises();

  expect(mocks.prepare).toHaveBeenCalledWith("output");
  expect(wrapper.getComponent(Viewer).props()).toEqual({
    open: true,
    error: null,
    before: "asset://localhost/cache/input.png?v=rendered1:input",
    after: "asset://localhost/output.png?v=asset:output",
  });
});
