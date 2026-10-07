import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";

import type { startResourcePreparation } from "../../../foundation/runtime/resource-preparation";
import LocalSpeechResourcesSettings from "../components/LocalSpeechResourcesSettings.vue";

const mocks = vi.hoisted(() => ({
  data: {
    value: {
      models: {
        diarization: { verified: false, installed: false, tokenConfigured: false, totalBytes: 32_820_977 },
      },
    },
  },
  error: { value: null },
  setToken: vi.fn<() => Promise<void>>(),
  remove: vi.fn<() => Promise<void>>(),
  prepare: vi.fn<typeof startResourcePreparation>(),
  invalidate: vi.fn<() => Promise<void>>(),
  cancel: vi.fn<() => Promise<void>>(),
}));
vi.mock("../../../foundation/runtime/resource-query", () => ({
  useLocalResourcesQuery: () => mocks,
  invalidateLocalResources: mocks.invalidate,
}));
vi.mock("../../../foundation/runtime/local-resources", () => ({
  setDiarizationAccessToken: mocks.setToken,
  removeDiarizationModel: mocks.remove,
}));
vi.mock("../../../foundation/runtime/resource-preparation", () => ({
  startResourcePreparation: mocks.prepare,
}));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn<(url: string) => Promise<void>>() }));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

beforeEach(() => {
  vi.clearAllMocks();
  mocks.data.value.models.diarization = {
    verified: false,
    installed: false,
    tokenConfigured: false,
    totalBytes: 32_820_977,
  };
  mocks.setToken.mockResolvedValue(undefined);
  mocks.invalidate.mockResolvedValue(undefined);
  mocks.cancel.mockResolvedValue(undefined);
});

function render() {
  const wrapper = mount(LocalSpeechResourcesSettings);
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}

it("uses official access credentials and a download action without folder imports or declarations", () => {
  const wrapper = render();
  expect(wrapper.find('input[type="password"]').exists()).toBe(true);
  expect(wrapper.find('input[type="checkbox"]').exists()).toBe(false);
  expect(wrapper.get('[data-testid="community-1-download"]').attributes("disabled")).toBeDefined();
  expect(wrapper.text()).not.toContain("settings.ai.localSpeech.import");
});

it("saves access credentials without exposing the stored token in the status or retaining the input", async () => {
  const wrapper = render();
  await wrapper.get('input[type="password"]').setValue("test-read-access");
  await wrapper.get('[data-testid="community-1-save-access"]').trigger("click");
  await flushPromises();
  expect(mocks.setToken).toHaveBeenCalledWith("test-read-access");
  expect((wrapper.get("input").element as HTMLInputElement).value).toBe("");
  expect(mocks.invalidate).toHaveBeenCalledOnce();
});

it("prepares the whole speaker recognition capability and lets Task Center own its lifetime", async () => {
  mocks.data.value.models.diarization.tokenConfigured = true;
  mocks.prepare.mockReturnValue({ completion: Promise.resolve(), cancel: mocks.cancel });
  const wrapper = render();
  await wrapper.get('[data-testid="community-1-download"]').trigger("click");
  await flushPromises();
  expect(mocks.prepare).toHaveBeenCalledWith(
    { kind: "speechDiarization" },
    "settings.ai.localSpeech.downloading",
  );
  expect(mocks.setToken).not.toHaveBeenCalled();
});

it("keeps a verified offline model usable after access credentials are removed", () => {
  mocks.data.value.models.diarization.verified = true;
  mocks.data.value.models.diarization.installed = true;
  const wrapper = render();
  expect(wrapper.find('[data-testid="community-1-download"]').exists()).toBe(false);
  expect(wrapper.text()).toContain("settings.ai.localSpeech.ready");
});

it("provides removal and redownload guidance for a corrupt installation", () => {
  mocks.data.value.models.diarization.installed = true;
  const wrapper = render();
  expect(wrapper.find('[data-testid="community-1-download"]').exists()).toBe(false);
  expect(wrapper.text()).toContain("settings.ai.localSpeech.invalid");
  expect(wrapper.text()).toContain("settings.ai.localSpeech.remove");
});
