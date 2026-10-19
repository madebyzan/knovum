import { flushPromises, mount, type VueWrapper } from "@vue/test-utils";
import { beforeEach, expect, it, vi } from "vitest";
import { reactive, ref } from "vue";

import type { TranslateVideoDubbingRequest } from "../../../contracts/generated/TranslateVideoDubbingRequest";
import VideoTranslationControls from "../VideoTranslationControls.vue";

const calls = vi.hoisted(() => ({
  start: vi.fn<(request: TranslateVideoDubbingRequest) => Promise<void>>(),
  restore: vi.fn<() => Promise<void>>(),
  cancel: vi.fn<() => Promise<void>>(),
  ensureLoaded: vi.fn<() => Promise<void>>(),
}));

vi.mock("../../use-video-translation-run", () => ({
  useVideoTranslationRun: () => ({
    activeRunId: ref<string>(),
    submitting: ref(false),
    cancelling: ref(false),
    error: ref<string>(),
    restore: calls.restore,
    start: calls.start,
    cancel: calls.cancel,
  }),
}));
vi.mock("../../../../../foundation/ai/ai-settings-store", () => ({ useAiSettingsStore: () => store }));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

beforeEach(() => vi.clearAllMocks());

/** The translate action, which the select triggers around it would otherwise hide. */
function translateButton(wrapper: VueWrapper) {
  return wrapper
    .findAll("button")
    .find((button) => button.text().includes("spaces.speech.translation.translate"))!;
}

const store = reactive({
  settings: ref({
    defaults: { translation: { type: "model", connectionId: "connection", modelId: "model" } },
    connections: [
      {
        id: "connection",
        name: "Connection",
        provider: "openai",
        baseUrl: "https://api.openai.com/v1",
        models: [{ id: "model", name: "Model", model: "qwen", capabilities: ["text-generation"] }],
      },
    ],
    desktopAgents: [],
  }),
  ready: ref(true),
  ensureLoaded: calls.ensureLoaded,
});

/**
 * A Run translates a saved revision, so the script on screen is committed first and the Run is
 * created for whatever revision that save produced. Skipping either half is what used to make
 * translation fail with a stale revision.
 */
it("commits the script before translating and uses the revision the save produced", async () => {
  let wrapper!: VueWrapper;
  const save = vi.fn<() => Promise<boolean>>(async () => {
    await wrapper.setProps({ expectedRevision: 2 });
    return true;
  });
  wrapper = mount(VideoTranslationControls, {
    props: { projectId: "project", expectedRevision: 1, lineCount: 2, save, disabled: false },
  });
  await flushPromises();

  await translateButton(wrapper).trigger("click");
  await flushPromises();

  expect(save).toHaveBeenCalledOnce();
  expect(calls.start).toHaveBeenCalledOnce();
  expect(calls.start.mock.calls[0]?.[0]).toMatchObject({
    projectId: "project",
    expectedRevision: 2,
    targetLanguage: "en",
  });
  wrapper.unmount();
});

it("does not translate when the script cannot be saved", async () => {
  const wrapper = mount(VideoTranslationControls, {
    props: {
      projectId: "project",
      expectedRevision: 1,
      lineCount: 2,
      save: () => Promise.resolve(false),
      disabled: false,
    },
  });
  await flushPromises();

  await translateButton(wrapper).trigger("click");
  await flushPromises();

  expect(calls.start).not.toHaveBeenCalled();
  wrapper.unmount();
});
