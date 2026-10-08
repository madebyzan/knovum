import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { shallowRef } from "vue";

import type { SpeechSourceSeparationCandidate } from "../../../contracts/generated/SpeechSourceSeparationCandidate";
import type { VideoDubbingOutputConfig } from "../../../contracts/generated/VideoDubbingOutputConfig";
import type { listAcceptedSourceSeparations } from "../../../speech/source-separation-ipc";
import VideoBackgroundSelect from "../VideoBackgroundSelect.vue";

const mocks = vi.hoisted(() => ({ list: vi.fn<typeof listAcceptedSourceSeparations>() }));
vi.mock("../../../speech/source-separation-ipc", () => ({ listAcceptedSourceSeparations: mocks.list }));
vi.mock("../../../speech/use-speech-media-sources", () => ({
  useSpeechMediaSources: () => ({ backgrounds: shallowRef([]) }),
}));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

const video = { assetPath: "/video.mp4", sourceRevision: "sha256:source", durationMs: 1_000 };
const candidate: SpeechSourceSeparationCandidate = {
  separationId: "accepted",
  runId: "accepted",
  spaceId: "space",
  source: video,
  background: { ...video, assetPath: "/background.wav" },
  vocals: { ...video, assetPath: "/vocals.wav" },
  modelSpecId: "spleeter",
  reviewStatus: "accepted",
  reviewWarning: "review",
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
function render(overrides: Partial<VideoDubbingOutputConfig> = {}) {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
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
  mocks.list.mockResolvedValue([candidate]);
});

it("selects an already reviewed accompaniment without launching separation inside dubbing", async () => {
  const wrapper = render();
  await flushPromises();
  expect(mocks.list).toHaveBeenCalledWith("space", video);
  await wrapper.get("select").setValue("separation:accepted");
  expect(wrapper.emitted("separated")).toEqual([[candidate]]);
});

it("retains a saved background while catalog retrieval fails and offers retry", async () => {
  mocks.list.mockRejectedValue(new Error("Unable to list saved accompaniments"));
  const wrapper = render({
    background: candidate.background,
    backgroundSeparation: {
      separationId: "accepted",
      source: video,
      modelSpecId: "spleeter",
    },
  });
  await flushPromises();
  expect(wrapper.get("select").element.value).toBe("separation:accepted");
  expect(wrapper.find('[role="alert"]').exists()).toBe(true);
  mocks.list.mockResolvedValue([candidate]);
  await wrapper.get("button").trigger("click");
  await flushPromises();
  expect(wrapper.find('[role="alert"]').exists()).toBe(false);
  await wrapper.get("select").setValue("mute");
  expect(wrapper.emitted("background")).toEqual([[null]]);
});
