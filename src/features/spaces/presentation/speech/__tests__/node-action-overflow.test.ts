import { flushPromises, mount } from "@vue/test-utils";
import { ToolbarButton, ToolbarRoot } from "reka-ui";
import { expect, it, onTestFinished, vi } from "vitest";
import { defineComponent, h, shallowRef } from "vue";
import { createI18n } from "vue-i18n";

import { localeMessages } from "../../../../../locales/locale-registry";
import UiDropdownMenu from "../../../../../shared/ui/UiDropdownMenu.vue";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { NodeSpeechOperation } from "../node-speech-operations";
import SpaceNodeSpeechActions from "../SpaceNodeSpeechActions.vue";
import SpaceNodeSpeechControls from "../SpaceNodeSpeechControls.vue";

vi.mock("../../../session/space-store", () => ({
  useSpaceStore: () => ({ activeSpaceId: "space", nodes: [] }),
}));
const i18n = createI18n({ legacy: false, locale: "en", messages: localeMessages });
const node: SpaceCanvasNode = {
  id: "note",
  type: "note",
  initialPosition: { x: 0, y: 0 },
  data: { kind: "note", title: "Source", plainText: "Read this", tiptapContent: { type: "doc" } },
};

it("keeps the node operation editor open after its overflow menu closes", async () => {
  const OperationBar = defineComponent({
    setup() {
      const operation = shallowRef<NodeSpeechOperation>();
      return () => [
        h(
          ToolbarRoot,
          {},
          {
            default: () =>
              h(
                UiDropdownMenu,
                {},
                {
                  trigger: () => h(ToolbarButton, { "data-testid": "more" }, () => "More"),
                  default: () =>
                    h(SpaceNodeSpeechActions, {
                      node,
                      separationUnavailableReason: undefined,
                      menuItem: true,
                      onBegin: (selected: NodeSpeechOperation) => {
                        operation.value = selected;
                      },
                    }),
                },
              ),
          },
        ),
        h(SpaceNodeSpeechControls, {
          node,
          operation: operation.value,
          "onUpdate:operation": (selected: NodeSpeechOperation | undefined) => {
            operation.value = selected;
          },
        }),
      ];
    },
  });
  const wrapper = mount(OperationBar, {
    attachTo: document.body,
    global: {
      plugins: [createI18n({ legacy: false, locale: "en", messages: localeMessages })],
      stubs: {
        SpaceNodeSpeechEditor: {
          props: ["node", "projectType"],
          template: '<div data-testid="editor" :data-operation="projectType">{{ node.id }}</div>',
        },
      },
    },
  });
  onTestFinished(() => {
    wrapper.unmount();
    document.body.innerHTML = "";
  });
  await wrapper.get('[data-testid="more"]').trigger("keydown", { key: "ArrowDown" });
  await flushPromises();
  const action = document.querySelector<HTMLDivElement>('[role="menuitem"][data-operation="audiobook"]');
  expect(action).not.toBeNull();
  action!.click();
  await flushPromises();
  await vi.waitFor(() => {
    expect(document.querySelector('[role="menu"]')).toBeNull();
    expect(document.querySelector('[data-testid="editor"]')?.textContent).toBe("note");
    expect(document.querySelector('[data-testid="editor"]')?.getAttribute("data-operation")).toBe(
      "audiobook",
    );
  });
});

it("closes cloning with Escape while keeping the canvas nonmodal", async () => {
  const audio: SpaceCanvasNode = {
    id: "audio",
    type: "audio",
    initialPosition: { x: 0, y: 0 },
    data: { kind: "audio", title: "Recording", assetPath: "/voice.wav", fileName: "voice.wav" },
  };
  const wrapper = mount(SpaceNodeSpeechControls, {
    props: { node: audio, operation: "clone" },
    global: { plugins: [i18n], stubs: { SpaceVoiceCloneBar: { template: "<button>Clone</button>" } } },
    attachTo: document.body,
  });
  onTestFinished(() => wrapper.unmount());
  await flushPromises();
  expect(document.querySelector('[role="dialog"]')).toBeNull();
  const panel = wrapper.get('[data-testid="node-speech-controls"]');
  await panel.trigger("keydown", { key: "Escape" });
  await flushPromises();
  expect(wrapper.emitted("update:operation")?.at(-1)).toEqual([undefined]);
});

it("waits for the project leave decision before closing the canvas panel", async () => {
  const requestClose = vi.fn<() => Promise<boolean>>().mockResolvedValue(false);
  const Editor = defineComponent({
    props: ["node", "projectType"],
    setup(_props, { expose }) {
      expose({ requestClose });
      return () => h("div", "Editor");
    },
  });
  const wrapper = mount(SpaceNodeSpeechControls, {
    props: { node, operation: "short_dubbing" },
    attachTo: document.body,
    global: { plugins: [i18n], stubs: { SpaceNodeSpeechEditor: Editor } },
  });
  onTestFinished(() => wrapper.unmount());
  await flushPromises();
  wrapper.getComponent(Editor).vm.$emit("close");
  await flushPromises();
  expect(wrapper.emitted("update:operation")).toBeUndefined();
  requestClose.mockResolvedValue(true);
  wrapper.getComponent(Editor).vm.$emit("close");
  await flushPromises();
  expect(wrapper.emitted("update:operation")?.at(-1)).toEqual([undefined]);
});
