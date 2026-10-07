import { flushPromises, mount } from "@vue/test-utils";
import { ToolbarButton, ToolbarRoot } from "reka-ui";
import { afterEach, expect, it, vi } from "vitest";
import { defineComponent, h } from "vue";
import { createI18n } from "vue-i18n";

import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import SpaceNodeSpeechAction from "../SpaceNodeSpeechAction.vue";

const i18n = createI18n({
  legacy: false,
  locale: "en",
  messages: {
    en: {
      common: { close: "Close" },
      spaces: { speech: { nodeReadAloud: "Read aloud", cloneVoice: "Clone voice" } },
    },
  },
});
const global = {
  plugins: [i18n],
  stubs: {
    SpaceNodeSpeechEditor: { props: ["node"], template: '<div data-testid="editor">{{ node.id }}</div>' },
    VoiceCandidateForm: {
      props: ["sourceNode"],
      template: '<div data-testid="clone">{{ sourceNode.id }}</div>',
    },
  },
};
const note: SpaceCanvasNode = {
  id: "first",
  type: "note",
  initialPosition: { x: 0, y: 0 },
  data: { kind: "note", title: "First", plainText: "Read this", tiptapContent: { type: "doc" } },
};
let wrapper: ReturnType<typeof mount>;
afterEach(() => {
  wrapper.unmount();
  document.body.innerHTML = "";
});

it("opens the selected node editor and closes it when the selected node changes", async () => {
  wrapper = mount(SpaceNodeSpeechAction, { props: { node: note }, global, attachTo: document.body });
  await wrapper.get('[data-testid="node-speech-action"]').trigger("click");
  await flushPromises();
  expect(document.querySelector('[data-testid="editor"]')?.textContent).toBe("first");
  await wrapper.setProps({ node: { ...note, id: "second" } });
  await flushPromises();
  expect(document.querySelector('[role="dialog"]')).toBeNull();
  await wrapper.get('[data-testid="node-speech-action"]').trigger("click");
  await flushPromises();
  expect(document.querySelector('[data-testid="editor"]')?.textContent).toBe("second");
});

it("uses the selected audio as the cloning source", async () => {
  const audio: SpaceCanvasNode = {
    id: "audio",
    type: "audio",
    initialPosition: { x: 0, y: 0 },
    data: { kind: "audio", title: "Recording", assetPath: "/recording.wav", fileName: "recording.wav" },
  };
  wrapper = mount(SpaceNodeSpeechAction, { props: { node: audio }, global, attachTo: document.body });
  expect(wrapper.get("button").attributes("aria-label")).toBe("Clone voice");
  await wrapper.get("button").trigger("click");
  await flushPromises();
  expect(document.querySelector('[data-testid="clone"]')?.textContent).toBe("audio");
  expect(document.querySelector('[data-testid="editor"]')).toBeNull();
});

it("participates in the node toolbar keyboard navigation", async () => {
  const Toolbar = defineComponent({
    setup: () => () =>
      h(
        ToolbarRoot,
        {},
        {
          default: () => [
            h(SpaceNodeSpeechAction, { node: note, compact: true }),
            h(ToolbarButton, { "data-testid": "next" }, () => "Next"),
          ],
        },
      ),
  });
  wrapper = mount(Toolbar, { global, attachTo: document.body });
  await flushPromises();
  const action = wrapper.get<HTMLButtonElement>('[data-testid="node-speech-action"]');
  action.element.focus();
  await flushPromises();
  await action.trigger("keydown", { key: "ArrowRight" });
  await flushPromises();
  await vi.waitFor(() => expect(document.activeElement).toBe(wrapper.get('[data-testid="next"]').element));
});
