import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, expect, it, onTestFinished, vi } from "vitest";
import { shallowRef } from "vue";
import { createMemoryHistory, createRouter } from "vue-router";

import { i18n } from "../../../../../foundation/i18n/instance";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import SpaceNodeSpeechEditor from "../SpaceNodeSpeechEditor.vue";

const loading = shallowRef(true);
const initializationReady = shallowRef(true);
const errors = shallowRef<string[]>([]);
const selectedProject = shallowRef();
const videos = shallowRef<unknown[]>([]);
const addProject = vi.fn<(input: unknown) => Promise<void>>();
vi.mock("../../../speech/use-speech-media-sources", () => ({ useSpeechMediaSources: () => ({ videos }) }));
vi.mock("../../../speech/use-speech-workspace", () => ({
  useSpeechWorkspace: () => ({
    voices: shallowRef([]),
    projects: shallowRef([]),
    selectedProject,
    loading,
    initializationReady,
    errors,
    creatingProject: shallowRef(false),
    savingRevision: shallowRef(false),
    hasActiveExecution: shallowRef(false),
    generationSubmitting: shallowRef(false),
    generationCancelling: shallowRef(false),
    previewListening: shallowRef(false),
    addProject,
    refresh: vi.fn<() => Promise<void>>(),
    openProject: vi.fn<() => Promise<void>>(),
    saveRevision: vi.fn<() => Promise<void>>(),
    settleRevisionSave: vi.fn<() => Promise<void>>(),
  }),
}));
afterEach(() => {
  loading.value = true;
  initializationReady.value = true;
  selectedProject.value = undefined;
  errors.value = [];
  videos.value = [];
  vi.clearAllMocks();
});
const node: SpaceCanvasNode = {
  id: "source",
  type: "markdown",
  initialPosition: { x: 0, y: 0 },
  data: { kind: "markdown", title: "Notes", text: "# Agenda\n\nDiscuss the plan." },
};

function render(projectType: "short_dubbing" | "audiobook" | "video_dubbing" = "short_dubbing") {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: "/", component: { template: "<div />" } },
      { path: "/settings", component: { template: "<div />" } },
    ],
  });
  const wrapper = mount(SpaceNodeSpeechEditor, {
    props: { node, projectType },
    global: {
      plugins: [i18n, router],
      stubs: { SpeechProjectEditor: true, VoiceCandidateForm: true, SpaceUnsavedChangesDialog: true },
    },
  });
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}

it.each(["short_dubbing", "audiobook"] as const)(
  "opens a source-bound %s draft without a project setup form",
  async (type) => {
    const wrapper = render(type);
    expect(addProject).not.toHaveBeenCalled();
    loading.value = false;
    await flushPromises();
    expect(addProject).toHaveBeenCalledExactlyOnceWith(
      expect.objectContaining({ title: "Notes", projectType: type }),
    );
    expect(wrapper.find("form").exists()).toBe(false);
    expect(wrapper.find('[aria-label="More"]').exists()).toBe(false);
    expect(wrapper.text()).toContain(i18n.global.t("spaces.speech.project.noVoices"));
    expect(wrapper.get('[data-testid="manage-voices"]').attributes("href")).toBe("/settings?pane=voices");
    await wrapper.get('[data-testid="add-reference-voice"]').trigger("click");
    expect(wrapper.findComponent({ name: "VoiceCandidateForm" }).exists()).toBe(true);
  },
);

it("does not create another draft when one already exists", async () => {
  render();
  selectedProject.value = { project: { id: "existing", activeRevision: 1 }, revision: { outputConfig: {} } };
  loading.value = false;
  await flushPromises();
  expect(addProject).not.toHaveBeenCalled();
});

it("waits for video readiness and initializes exactly once", async () => {
  const wrapper = render("video_dubbing");
  loading.value = false;
  await flushPromises();
  expect(addProject).not.toHaveBeenCalled();
  expect(wrapper.text()).toContain(i18n.global.t("spaces.speech.project.sourceNotReady"));
  videos.value = [
    { id: "source", source: { assetPath: "/video.mp4", sourceRevision: "sha256:source", durationMs: 1000 } },
  ];
  await flushPromises();
  expect(addProject).toHaveBeenCalledTimes(1);
  loading.value = true;
  loading.value = false;
  await flushPromises();
  expect(addProject).toHaveBeenCalledTimes(1);
});

it("does not mistake a failed project load for an empty workspace", async () => {
  render();
  initializationReady.value = false;
  errors.value = ["Could not load saved drafts"];
  loading.value = false;
  await flushPromises();
  expect(addProject).not.toHaveBeenCalled();
});

it("closes initialized content without a decision but protects actual incomplete edits", async () => {
  const wrapper = render();
  selectedProject.value = { project: { id: "existing", activeRevision: 1 }, revision: { outputConfig: {} } };
  loading.value = false;
  await flushPromises();
  const editor = wrapper.getComponent({ name: "SpeechProjectEditor" });
  editor.vm.$emit("draftState", false);
  await wrapper.vm.$nextTick();
  expect(await wrapper.vm.requestClose()).toBe(true);
  const dialog = wrapper.getComponent({ name: "SpaceUnsavedChangesDialog" });
  expect(dialog.props("open")).toBe(false);
  editor.vm.$emit("draftState", true);
  await wrapper.vm.$nextTick();
  const closing = wrapper.vm.requestClose();
  await flushPromises();
  expect(dialog.props("open")).toBe(true);
  expect(dialog.props("showSave")).toBe(false);
  expect(dialog.props("title")).toBe(i18n.global.t("spaces.speech.project.incompleteTitle"));
  dialog.vm.$emit("cancel");
  expect(await closing).toBe(false);
});
