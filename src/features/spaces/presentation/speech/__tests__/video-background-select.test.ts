import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { shallowRef } from "vue";

import { queryClient as resourceClient } from "../../../../../foundation/query/query-client";
import type { RunObservation } from "../../../contracts/generated/RunObservation";
import type { SpeechSourceSeparationPublication } from "../../../contracts/generated/SpeechSourceSeparationPublication";
import type { VideoDubbingOutputConfig } from "../../../contracts/generated/VideoDubbingOutputConfig";
import { observeResourceRun } from "../../../runs/resource-run-observation";
import type { observeRun } from "../../../runs/run-observation";
import type { listSourceSeparations } from "../../../speech/source-separation-ipc";
import VideoBackgroundSelect from "../VideoBackgroundSelect.vue";

const mocks = vi.hoisted(() => ({
  list: vi.fn<typeof listSourceSeparations>(),
  observe: vi.fn<typeof observeRun>(),
}));
vi.mock("../../../runs/run-observation", () => ({ observeRun: mocks.observe }));
vi.mock("../../../session/space-store", () => ({
  useSpaceStore: () => ({ activeSpaceId: "another-space" }),
}));
vi.mock("../../../../../foundation/tasks/task-store", () => ({
  useTaskStore: () => ({ restoreRunning: vi.fn<() => void>(), setOpen: vi.fn<() => void>() }),
}));
vi.mock("../../../../../foundation/i18n/i18n", () => ({
  i18n: { global: { t: (key: string) => key } },
}));
vi.mock("../../../speech/source-separation-ipc", () => ({ listSourceSeparations: mocks.list }));
vi.mock("../../../speech/use-speech-media-sources", () => ({
  useSpeechMediaSources: () => ({ backgrounds: shallowRef([]) }),
}));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

const video = { assetPath: "/video.mp4", sourceRevision: "sha256:source", durationMs: 1_000 };
const published: SpeechSourceSeparationPublication = {
  separationId: "published",
  sourceNodeId: "source",
  runId: "published",
  spaceId: "space",
  source: video,
  background: { ...video, assetPath: "/background.wav" },
  vocals: { ...video, assetPath: "/vocals.wav" },
  modelSpecId: "spleeter",
};
const config: VideoDubbingOutputConfig = {
  format: "mp4",
  video,
  background: null,
  backgroundSeparation: null,
  voiceGainDb: 0,
  backgroundGainDb: -12,
  translation: null,
  preserveOriginalOutsideDialogue: false,
  sourceDialogueIntervals: [],
  retainOriginalTrack: true,
};
function render(
  overrides: Partial<VideoDubbingOutputConfig> = {},
  queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } }),
) {
  const wrapper = mount(VideoBackgroundSelect, {
    props: { spaceId: "space", config: { ...config, ...overrides }, disabled: false },
    global: {
      plugins: [[VueQueryPlugin, { queryClient }]],
      stubs: {
        UiSelect: {
          props: ["options", "modelValue"],
          emits: ["update:modelValue"],
          template:
            '<select :value="modelValue" @change="$emit(\'update:modelValue\', $event.target.value)"><option v-for="option in options" :key="option.value" :value="option.value">{{ option.label }}</option></select>',
        },
      },
    },
  });
  onTestFinished(() => {
    wrapper.unmount();
    queryClient.clear();
  });
  return wrapper;
}
beforeEach(() => {
  vi.clearAllMocks();
  mocks.list.mockResolvedValue([published]);
});

it("selects a published accompaniment without launching separation inside dubbing", async () => {
  const wrapper = render();
  await flushPromises();
  expect(mocks.list).toHaveBeenCalledWith("space", video);
  await wrapper.get("select").setValue("separation:published");
  expect(wrapper.emitted("separated")).toEqual([[published]]);
});

it("retains a saved background while catalog retrieval fails and offers retry", async () => {
  mocks.list.mockRejectedValue(new Error("Unable to list saved accompaniments"));
  const wrapper = render({
    background: published.background,
    backgroundSeparation: {
      separationId: "published",
      source: video,
      modelSpecId: "spleeter",
    },
  });
  await flushPromises();
  expect(wrapper.get("select").element.value).toBe("separation:published");
  expect(wrapper.find('[role="alert"]').exists()).toBe(true);
  mocks.list.mockResolvedValue([published]);
  await wrapper.get("button").trigger("click");
  await flushPromises();
  expect(wrapper.find('[role="alert"]').exists()).toBe(false);
  await wrapper.get("select").setValue("mute");
  expect(wrapper.emitted("background")).toEqual([[null]]);
});

it("refreshes published accompaniments after a background run finishes in another selected space", async () => {
  mocks.list.mockResolvedValue([]);
  const wrapper = render({}, resourceClient);
  await flushPromises();
  expect(wrapper.find('option[value="separation:published"]').exists()).toBe(false);
  const run: RunObservation = {
    id: "separation-refresh",
    scope: { kind: "space", spaceId: "space" },
    parentRunId: null,
    retryOfRunId: null,
    forkOfRunId: null,
    kind: "workflow",
    state: "running",
    stateVersion: 1,
    waitReason: null,
    inputTokens: 0,
    outputTokens: 0,
    totalTokens: 0,
    stepAttempts: 1,
    expandedSteps: 1,
    outputBytes: 0,
    createdAt: 1,
    updatedAt: 1,
    completedAt: null,
  };
  observeResourceRun(run, "Separation");
  mocks.list.mockResolvedValue([published]);
  mocks.observe.mock.calls[0]![1].finished();
  await flushPromises();
  expect(wrapper.find('option[value="separation:published"]').exists()).toBe(true);
  await wrapper.get("select").setValue("separation:published");
  expect(wrapper.emitted("separated")).toEqual([[published]]);
});
