import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { reactive, ref } from "vue";

import DesktopAgentsSettingsPane from "../DesktopAgentsSettingsPane.vue";

const calls = vi.hoisted(() => ({
  loadAi: vi.fn<() => Promise<void>>(),
  setDesktopAgentEnabled: vi.fn<() => Promise<void>>(),
}));
vi.mock("vue-sonner", () => ({
  toast: { success: vi.fn<(msg: string) => void>(), error: vi.fn<(msg: string) => void>() },
}));
vi.mock("../../../../foundation/ai/execution-settings", () => ({
  setDesktopAgentEnabled: calls.setDesktopAgentEnabled,
}));
vi.mock("../../../../foundation/ai/ai-settings-store", () => ({ useAiSettingsStore: () => aiStore }));
vi.mock("../../../../foundation/ai/use-execution-catalog-query", () => ({
  useExecutionCatalogQuery: () => catalog,
}));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

const loadError = ref<Error>();
const aiStore = reactive({
  settings: ref({
    desktopAgents: [{ kind: "claude_code", enabled: true }],
  }),
  ready: ref(true),
  loadError,
  saveError: ref(),
  ensureLoaded: calls.loadAi,
});
const catalog = {
  data: ref({
    agents: [
      {
        kind: "claude_code",
        displayName: "Claude Code",
        executableNames: ["claude"],
        help: {
          connection: "executable_cli",
          documentationUrl: "https://example.com",
          setupUrl: "https://example.com/setup",
        },
      },
    ],
    installations: [
      {
        kind: "claude_code",
        detected: true,
        executablePath: "/usr/local/bin/claude",
      },
    ],
  }),
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
    calls.setDesktopAgentEnabled.mockResolvedValue();
  });
  it("renders once the owning AI settings are ready", async () => {
    const wrapper = mount(DesktopAgentsSettingsPane, { global });
    await flushPromises();
    expect(wrapper.find('[data-testid="agent-settings"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="settings-failure"]').exists()).toBe(false);
    expect(calls.loadAi).toHaveBeenCalledOnce();
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
  it("toggles agent enabled state via setDesktopAgentEnabled", async () => {
    const wrapper = mount(DesktopAgentsSettingsPane, { global });
    await flushPromises();
    const toggleButton = wrapper
      .findAll("button")
      .find((btn) => btn.text().includes("settings.agents.disable"));
    expect(toggleButton?.exists()).toBe(true);
    await toggleButton!.trigger("click");
    await flushPromises();
    expect(calls.setDesktopAgentEnabled).toHaveBeenCalledWith("claude_code", false);
    wrapper.unmount();
  });
});
