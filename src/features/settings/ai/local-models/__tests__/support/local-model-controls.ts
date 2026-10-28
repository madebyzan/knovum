import { mount } from "@vue/test-utils";
import { beforeEach, onTestFinished, vi } from "vitest";

import type { startResourcePreparation } from "../../../../../../foundation/runtime/resource-preparation";
import type * as ResourcePreparation from "../../../../../../foundation/runtime/resource-preparation";
import type { LocalModelAvailability } from "../../../../../../shared/contracts/generated/LocalModelAvailability";
import type { LocalModelId } from "../../../../../../shared/contracts/generated/LocalModelId";
import type { confirmAction } from "../../../../../../shared/ui/confirm-action";
import LocalModelConfiguration from "../../components/LocalModelConfiguration.vue";

const mocks = vi.hoisted(() => ({
  prepare: vi.fn<typeof startResourcePreparation>(),
  select: vi.fn<(model: LocalModelId) => Promise<void>>(),
  remove: vi.fn<(model: LocalModelId) => Promise<void>>(),
  cancel: vi.fn<() => Promise<void>>(),
  invalidate: vi.fn<() => Promise<void>>(),
  confirm: vi.fn<typeof confirmAction>(),
}));
vi.mock("../../../../../../foundation/runtime/resource-preparation", async (importOriginal) => ({
  ...(await importOriginal<typeof ResourcePreparation>()),
  startResourcePreparation: mocks.prepare,
}));
vi.mock("../../../../../../foundation/runtime/local-resources", () => ({
  removeLocalModel: mocks.remove,
  selectLocalModel: mocks.select,
}));
vi.mock("../../../../../../foundation/runtime/local-resources-query", () => ({
  invalidateLocalResources: mocks.invalidate,
}));
vi.mock("../../../../../../shared/ui/confirm-action", () => ({ confirmAction: mocks.confirm }));
vi.mock("../../../../../../foundation/i18n/instance", () => ({
  i18n: { global: { t: (key: string) => key } },
}));
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
  memoryAdvisoryBytes: null,
};

beforeEach(() => {
  vi.clearAllMocks();
  mocks.confirm.mockResolvedValue(true);
});
export function render(overrides: Partial<LocalModelAvailability> = {}) {
  const wrapper = mount(LocalModelConfiguration, {
    attachTo: document.body,
    props: { model: { ...model, ...overrides } },
  });
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}

export { mocks };
