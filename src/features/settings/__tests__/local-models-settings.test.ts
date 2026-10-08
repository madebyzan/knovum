import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";

import type { startResourcePreparation } from "../../../foundation/runtime/resource-preparation";
import type { LocalModelAvailability } from "../../../shared/contracts/generated/LocalModelAvailability";
import type { LocalModelId } from "../../../shared/contracts/generated/LocalModelId";
import type { confirmAction } from "../../../shared/ui/confirm-action";
import LocalModelConfiguration from "../components/LocalModelConfiguration.vue";

const mocks = vi.hoisted(() => ({
  prepare: vi.fn<typeof startResourcePreparation>(),
  select: vi.fn<(model: LocalModelId) => Promise<void>>(),
  remove: vi.fn<(model: LocalModelId) => Promise<void>>(),
  cancel: vi.fn<() => Promise<void>>(),
  invalidate: vi.fn<() => Promise<void>>(),
  confirm: vi.fn<typeof confirmAction>(),
}));
vi.mock("../../../foundation/runtime/resource-preparation", () => ({
  startResourcePreparation: mocks.prepare,
}));
vi.mock("../../../foundation/runtime/local-resources", () => ({
  removeLocalModel: mocks.remove,
  selectLocalModel: mocks.select,
}));
vi.mock("../../../foundation/runtime/resource-query", () => ({ invalidateLocalResources: mocks.invalidate }));
vi.mock("../../../shared/ui/confirm-action", () => ({ confirmAction: mocks.confirm }));
vi.mock("../../../foundation/i18n/i18n", () => ({ i18n: { global: { t: (key: string) => key } } }));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

const model: LocalModelAvailability = {
  id: "sherpaDiarization",
  name: "Sherpa",
  capability: "speechDiarization",
  operations: [],
  referenceTranscript: "unused",
  group: "audio",
  essential: false,
  selected: false,
  selectable: false,
  fixed: false,
  totalBytes: 41_146_685,
  installedBytes: 0,
  installed: false,
  verified: false,
  downloadAvailable: true,
  unavailableReason: null,
};

beforeEach(() => {
  vi.clearAllMocks();
  mocks.confirm.mockResolvedValue(true);
});
function render(overrides: Partial<LocalModelAvailability> = {}) {
  const wrapper = mount(LocalModelConfiguration, { props: { model: { ...model, ...overrides } } });
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}

it("downloads the selected model through capability preparation without account forms", async () => {
  mocks.prepare.mockReturnValue({ completion: Promise.resolve(), cancel: mocks.cancel });
  const wrapper = render();
  expect(wrapper.find("input").exists()).toBe(false);
  await wrapper.get('[data-testid="model-download"]').trigger("click");
  await flushPromises();
  expect(mocks.prepare).toHaveBeenCalledWith(
    { kind: "model", model: "sherpaDiarization" },
    "settings.ai.localModels.downloading",
    expect.any(Function),
  );
});

it("keeps preparation cancellable while active", async () => {
  let finish: (() => void) | undefined;
  mocks.prepare.mockReturnValue({
    completion: new Promise<void>((resolve) => {
      finish = resolve;
    }),
    cancel: mocks.cancel,
  });
  const wrapper = render();
  await wrapper.get("button").trigger("click");
  await wrapper.get("button").trigger("click");
  expect(mocks.cancel).toHaveBeenCalledOnce();
  finish?.();
  await flushPromises();
});

it("shows a corrupt optional installation with a safe removal action", async () => {
  const wrapper = render({ installed: true, installedBytes: 30 });
  expect(wrapper.text()).toContain("settings.ai.localModels.invalid");
  await wrapper.get('[data-testid="model-remove"]').trigger("click");
  await flushPromises();
  expect(mocks.remove).toHaveBeenCalledWith("sherpaDiarization");
  expect(mocks.invalidate).toHaveBeenCalledOnce();
});

it("repairs a corrupt optional snapshot directly without requiring removal first", async () => {
  mocks.prepare.mockReturnValue({ completion: Promise.resolve(), cancel: mocks.cancel });
  const wrapper = render({ installed: true, installedBytes: 30 });
  await wrapper.get('[data-testid="model-download"]').trigger("click");
  await flushPromises();
  expect(mocks.prepare).toHaveBeenCalledWith(
    { kind: "model", model: "sherpaDiarization" },
    "settings.ai.localModels.downloading",
    expect.any(Function),
  );
  expect(mocks.remove).not.toHaveBeenCalled();
});

it("keeps the embedding model fixed and essential models under application ownership", () => {
  const wrapper = render({
    id: "qwen3Embedding",
    essential: true,
    fixed: true,
    installed: true,
    verified: true,
  });
  expect(wrapper.text()).toContain("settings.ai.localModels.fixedEmbedding");
  expect(wrapper.find('[data-testid="model-remove"]').exists()).toBe(false);
});

it("allows a corrupt essential model to be downloaded again without exposing removal", async () => {
  mocks.prepare.mockReturnValue({ completion: Promise.resolve(), cancel: mocks.cancel });
  const wrapper = render({ id: "qwen3Embedding", essential: true, installed: true });
  expect(wrapper.find('[data-testid="model-remove"]').exists()).toBe(false);
  await wrapper.get('[data-testid="model-download"]').trigger("click");
  await flushPromises();
  expect(mocks.prepare).toHaveBeenCalledWith(
    { kind: "model", model: "qwen3Embedding" },
    "settings.ai.localModels.downloading",
    expect.any(Function),
  );
});

it.each([
  ["whisperTiny", "transcription"],
  ["voxCpm2", "speechSynthesis"],
  ["u2net", "imageSegmentation"],
  ["spleeter2StemsFull", "speechSeparation"],
  ["sherpaDiarizationFull", "speechDiarization"],
] as const)("selects a verified %s default without altering existing voices", async (id, capability) => {
  const wrapper = render({ id, capability, selectable: true, installed: true, verified: true });
  await wrapper.get('[data-testid="model-select"]').trigger("click");
  await flushPromises();
  expect(mocks.select).toHaveBeenCalledWith(id);
  await wrapper.setProps({
    model: {
      ...model,
      id,
      capability,
      selectable: true,
      installed: true,
      verified: true,
      selected: true,
    },
  });
  expect(wrapper.find('[data-testid="model-remove"]').exists()).toBe(false);
  expect(wrapper.find('[data-testid="model-select"]').exists()).toBe(false);
});

it("requires downloading an alternative before it can be selected", () => {
  const wrapper = render({ id: "whisperLargeV3", capability: "transcription" });
  expect(wrapper.find('[data-testid="model-select"]').exists()).toBe(false);
  expect(wrapper.find('[data-testid="model-download"]').exists()).toBe(true);
});
