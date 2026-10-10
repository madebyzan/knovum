import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { reactive, ref } from "vue";

import DesktopAgentsSettingsPane from "../components/panes/DesktopAgentsSettingsPane.vue";

const calls = vi.hoisted(() => ({
  loadAi: vi.fn<() => Promise<void>>(),
  loadCreation: vi.fn<() => never>(),
}));
vi.mock("../../../foundation/ai/ai-settings-store", () => ({ useAiSettingsStore: () => aiStore }));
vi.mock("../../../foundation/ai/creation-library-store", () => ({
  useCreationLibraryStore: calls.loadCreation,
}));
vi.mock("../../../foundation/ai/use-execution-catalog-query", () => ({
  useExecutionCatalogQuery: () => catalog,
}));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

const loadError = ref<Error>();
const aiStore = reactive({
  settings: ref({ desktopAgents: [] }),
  ready: ref(true),
  loadError,
  saveError: ref(),
  ensureLoaded: calls.loadAi,
});
const catalog = {
  data: ref({ agents: [], installations: [] }),
  installationFetching: ref(false),
  descriptorPending: ref(false),
  descriptorError: ref(),
  installationError: ref(),
};
const global = {
  stubs: {
    UiSettingsHeaderActions: { template: "<div><slot /></div>" },
    UiSettingsSection: { template: '<section data-testid="agent-settings"><slot /></section>' },
    SettingsLoadFailure: { template: '<div data-testid="settings-failure" />' },
  },
};

describe("desktop Agent settings ownership", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    loadError.value = undefined;
    calls.loadAi.mockResolvedValue();
    calls.loadCreation.mockImplementation(() => {
      throw new Error("Unrelated creation library is unavailable");
    });
  });
  it("remains usable independently of the generation preset store", async () => {
    const wrapper = mount(DesktopAgentsSettingsPane, { global });
    await flushPromises();
    expect(wrapper.find('[data-testid="agent-settings"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="settings-failure"]').exists()).toBe(false);
    expect(calls.loadAi).toHaveBeenCalledOnce();
    expect(calls.loadCreation).not.toHaveBeenCalled();
    wrapper.unmount();
  });
  it("still surfaces the owning AI settings failure", async () => {
    loadError.value = new Error("AI preferences are malformed");
    const wrapper = mount(DesktopAgentsSettingsPane, { global });
    await flushPromises();
    expect(wrapper.find('[data-testid="settings-failure"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="agent-settings"]').exists()).toBe(false);
    wrapper.unmount();
  });
});
