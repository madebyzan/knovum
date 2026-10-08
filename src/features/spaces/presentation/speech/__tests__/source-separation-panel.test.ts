import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { shallowRef } from "vue";

import type { exportSpaceAssetFile } from "../../../assets/export";
import type { SpeechSourceSeparationCandidate } from "../../../contracts/generated/SpeechSourceSeparationCandidate";
import type { useSpaceStore } from "../../../session/space-store";
import type { useSourceSeparationCandidate } from "../../../speech/use-source-separation-candidate";
import VideoSourceSeparationPanel from "../VideoSourceSeparationPanel.vue";

const mocks = vi.hoisted(() => ({
  activeSpaceId: "space",
  addContent: vi.fn<ReturnType<typeof useSpaceStore>["addContent"]>(),
  useCandidate: vi.fn<typeof useSourceSeparationCandidate>(),
  exportFile: vi.fn<typeof exportSpaceAssetFile>(),
}));
vi.mock("../../../session/space-store", () => ({ useSpaceStore: () => mocks }));
vi.mock("../../../speech/use-source-separation-candidate", () => ({
  useSourceSeparationCandidate: mocks.useCandidate,
}));
vi.mock("../../../assets/export", () => ({ exportSpaceAssetFile: mocks.exportFile }));
vi.mock("../../../../../foundation/runtime/resource-query", () => ({
  useLocalModelsQuery: () => ({
    data: shallowRef([
      {
        id: "spleeter2Stems",
        name: "Spleeter · 2 stems / FP16",
        capability: "speechSeparation",
        selected: true,
        verified: false,
        totalBytes: 39_000_000,
      },
    ]),
    error: shallowRef(null),
  }),
}));
vi.mock("@tauri-apps/api/core", () => ({ convertFileSrc: (path: string) => path }));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));
const source = { assetPath: "/source.wav", sourceRevision: "sha256:source", durationMs: 1_000 };
const candidate: SpeechSourceSeparationCandidate = {
  separationId: "run",
  runId: "run",
  spaceId: "space",
  source,
  background: { ...source, assetPath: "/managed/accompaniment.wav" },
  vocals: { ...source, assetPath: "/managed/vocals.wav" },
  modelSpecId: "spleeter",
  reviewStatus: "accepted",
  reviewWarning: "review",
};
function render(reviewStatus: SpeechSourceSeparationCandidate["reviewStatus"] = "accepted") {
  type Separation = ReturnType<typeof useSourceSeparationCandidate>;
  mocks.useCandidate.mockReturnValue({
    candidate: shallowRef({ ...candidate, reviewStatus }),
    activeRunId: shallowRef(undefined),
    busy: shallowRef(false),
    error: shallowRef(undefined),
    restore: vi.fn<Separation["restore"]>().mockResolvedValue(undefined),
    start: vi.fn<Separation["start"]>().mockResolvedValue(undefined),
    cancel: vi.fn<Separation["cancel"]>().mockResolvedValue(undefined),
    review: vi.fn<Separation["review"]>().mockResolvedValue(undefined),
    reset: vi.fn<Separation["reset"]>(),
  });
  const wrapper = mount(VideoSourceSeparationPanel, { props: { spaceId: "space", source, disabled: false } });
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}
beforeEach(() => {
  vi.resetAllMocks();
  mocks.exportFile.mockResolvedValue(true);
});
it("saves an accepted accompaniment without requiring a dubbing project", async () => {
  const wrapper = render();
  await wrapper.get('[data-testid="export-separated-background"]').trigger("click");
  await flushPromises();
  expect(mocks.exportFile).toHaveBeenCalledWith(candidate.background.assetPath, "accompaniment.wav");
});
it("shows the selected separation model and first-use preparation", () => {
  const wrapper = render();
  expect(wrapper.get('[data-testid="separation-model"]').text()).toContain("Spleeter");
  expect(wrapper.text()).toContain("spaces.speech.modelFirstUse");
});
it("does not offer export for an unreviewed candidate", () => {
  expect(render("pending").find('[data-testid="export-separated-background"]').exists()).toBe(false);
});
it("shows a Rust export failure and allows another attempt", async () => {
  mocks.exportFile.mockRejectedValueOnce(new Error("Cannot write destination"));
  const wrapper = render();
  await wrapper.get('[data-testid="export-separated-background"]').trigger("click");
  await flushPromises();
  expect(wrapper.get('[role="alert"]').text()).toContain("Cannot write destination");
  await wrapper.get('[data-testid="export-separated-background"]').trigger("click");
  await flushPromises();
  expect(wrapper.find('[role="alert"]').exists()).toBe(false);
  expect(mocks.exportFile).toHaveBeenCalledTimes(2);
});

it("preserves an independently playable vocal track and exports it", async () => {
  const wrapper = render();
  expect(wrapper.findAll("audio").map((audio) => audio.attributes("src"))).toContain(
    candidate.vocals.assetPath,
  );
  await wrapper.get('[data-testid="export-separated-vocals"]').trigger("click");
  await flushPromises();
  expect(mocks.exportFile).toHaveBeenCalledWith(candidate.vocals.assetPath, "vocals.wav");
});
it("keeps both unreviewed stems under candidate review", () => {
  const wrapper = render("pending");
  expect(wrapper.findAll("audio")).toHaveLength(2);
  expect(wrapper.find('[data-testid="export-separated-vocals"]').exists()).toBe(false);
});

it("reuses the accepted vocal track as a Space audio node", async () => {
  const wrapper = render();
  await wrapper.get('[data-testid="insert-separated-vocals"]').trigger("click");
  await flushPromises();
  expect(mocks.addContent).toHaveBeenCalledWith(
    expect.objectContaining({
      kind: "audio",
      assetPath: candidate.vocals.assetPath,
      sourceRevision: candidate.vocals.sourceRevision,
    }),
  );
});
it("does not insert a track into a different active Space", async () => {
  const wrapper = render();
  mocks.activeSpaceId = "other";
  await wrapper.get('[data-testid="insert-separated-vocals"]').trigger("click");
  await flushPromises();
  expect(mocks.addContent).not.toHaveBeenCalled();
  mocks.activeSpaceId = "space";
});
