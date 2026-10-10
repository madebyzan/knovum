import { flushPromises, mount } from "@vue/test-utils";
import { ToolbarButton, ToolbarRoot } from "reka-ui";
import { afterEach, expect, it, vi } from "vitest";
import { defineComponent, h, shallowRef, type PropType } from "vue";
import { createI18n } from "vue-i18n";

import { localeMessages } from "../../../../../locales/locale-registry";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { NodeSpeechOperation } from "../node-speech-operations";
import SpaceNodeSpeechActions from "../SpaceNodeSpeechActions.vue";
import SpaceNodeSpeechControls from "../SpaceNodeSpeechControls.vue";

vi.mock("../../../session/space-store", () => ({
  useSpaceStore: () => ({ activeSpaceId: "space", nodes: [] }),
}));
const SpeechOperation = defineComponent({
  props: { node: { type: Object as PropType<SpaceCanvasNode>, required: true } },
  setup(props) {
    const operation = shallowRef<NodeSpeechOperation>();
    return () => [
      h(SpaceNodeSpeechActions, {
        node: props.node,
        separationUnavailableReason: undefined,
        onBegin: (selected: NodeSpeechOperation) => {
          operation.value = selected;
        },
      }),
      h(SpaceNodeSpeechControls, {
        node: props.node,
        operation: operation.value,
        "onUpdate:operation": (selected: NodeSpeechOperation | undefined) => {
          operation.value = selected;
        },
      }),
    ];
  },
});

const i18n = createI18n({
  legacy: false,
  locale: "en",
  messages: localeMessages,
});
const global = {
  plugins: [i18n],
  stubs: {
    ToolbarButton: { template: "<slot />" },
    SpaceNodeSpeechEditor: {
      props: ["node", "projectType"],
      template: '<div data-testid="editor" :data-project-type="projectType">{{ node.id }}</div>',
    },
    SpaceVoiceCloneBar: {
      props: ["node"],
      template: '<div data-testid="clone">{{ node.id }}</div>',
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
  wrapper = mount(SpeechOperation, { props: { node: note }, global, attachTo: document.body });
  await wrapper.get('[data-testid="node-speech-action"]').trigger("click");
  await flushPromises();
  expect(document.querySelector('[data-testid="editor"]')?.textContent).toBe("first");
  expect(document.querySelector('[aria-modal="true"]')).toBeNull();
  expect(wrapper.find('[data-testid="node-speech-controls"] [data-testid="editor"]').exists()).toBe(true);
  expect(wrapper.find('button[aria-haspopup="dialog"]').exists()).toBe(false);
  expect(document.querySelector('[data-testid="node-speech-controls"] [role="toolbar"]')).toBeNull();
  await wrapper.setProps({ node: { ...note, data: { ...note.data, title: "Renamed" } } });
  await flushPromises();
  expect(document.querySelector('[data-testid="editor"]')?.textContent).toBe("first");
  await wrapper.setProps({ node: { ...note, id: "second" } });
  await flushPromises();
  expect(document.querySelector('[data-testid="node-speech-controls"]')).toBeNull();
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
  wrapper = mount(SpeechOperation, { props: { node: audio }, global, attachTo: document.body });
  expect(wrapper.get("button").attributes("aria-label")).toBe("Clone from reference");
  await wrapper.get("button").trigger("click");
  await flushPromises();
  expect(document.querySelector('[data-testid="clone"]')?.textContent).toBe("audio");
  expect(document.querySelector('[data-testid="editor"]')).toBeNull();
  await wrapper.setProps({ node: { ...audio, data: { ...audio.data, sourceRevision: "updated-source" } } });
  await flushPromises();
  expect(document.querySelector('[data-testid="node-speech-controls"]')).toBeNull();
});

it("provides separate dubbing and reference-voice operations on a video node", async () => {
  const video: SpaceCanvasNode = {
    id: "video",
    type: "video",
    initialPosition: { x: 0, y: 0 },
    data: { kind: "video", title: "Interview", assetPath: "/interview.mp4", fileName: "interview.mp4" },
  };
  wrapper = mount(SpeechOperation, { props: { node: video }, global, attachTo: document.body });
  expect(
    wrapper
      .findAll('[data-testid="node-speech-action"]')
      .map((action) => action.attributes("data-operation")),
  ).toEqual(["video_dubbing", "clone", "separate"]);
  await wrapper.get('[data-operation="clone"]').trigger("click");
  await flushPromises();
  expect(document.querySelector('[data-testid="clone"]')?.textContent).toBe("video");
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
            h(SpeechOperation, { node: note }),
            h(ToolbarButton, { "data-testid": "next" }, () => "Next"),
          ],
        },
      ),
  });
  wrapper = mount(Toolbar, {
    global: { ...global, stubs: { ...global.stubs, ToolbarButton: false } },
    attachTo: document.body,
  });
  await flushPromises();
  const action = wrapper.get<HTMLButtonElement>('[data-testid="node-speech-action"]');
  action.element.focus();
  await flushPromises();
  await action.trigger("keydown", { key: "ArrowRight" });
  await flushPromises();
  await vi.waitFor(() =>
    expect(document.activeElement).toBe(wrapper.findAll('[data-testid="node-speech-action"]')[1]!.element),
  );
});

it("provides separate text operations with one project type per editor", async () => {
  wrapper = mount(SpeechOperation, { props: { node: note }, global, attachTo: document.body });
  const actions = wrapper.findAll('[data-testid="node-speech-action"]');
  expect(actions).toHaveLength(2);
  await actions[1]!.trigger("click");
  await flushPromises();
  expect(document.querySelector('[data-testid="editor"]')?.getAttribute("data-project-type")).toBe(
    "audiobook",
  );
  await wrapper.setProps({ node: { ...note, id: "next" } });
  await flushPromises();
  await wrapper.get('[data-testid="node-speech-action"]').trigger("click");
  await flushPromises();
  expect(document.querySelector('[data-testid="editor"]')?.getAttribute("data-project-type")).toBe(
    "short_dubbing",
  );
});
