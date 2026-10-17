import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { reactive, ref } from "vue";

import AiRoutingSettingsPane from "../AiRoutingSettingsPane.vue";

const calls = vi.hoisted(() => ({
  loadAi: vi.fn<() => Promise<void>>(),
  routerPush: vi.fn<(to: unknown) => Promise<void>>(),
  setDefaultExecutionTarget: vi.fn<() => Promise<void>>(),
}));

vi.mock("vue-router", () => ({
  useRouter: () => ({ push: calls.routerPush }),
  useRoute: () => ({ path: "/settings", query: { pane: "aiRouting" } }),
}));
vi.mock("vue-sonner", () => ({
  toast: { success: vi.fn<(msg: string) => void>(), error: vi.fn<(msg: string) => void>() },
}));
vi.mock("../../../../foundation/ai/execution-settings", () => ({
  setDefaultExecutionTarget: calls.setDefaultExecutionTarget,
}));
vi.mock("../../../../foundation/ai/ai-settings-store", () => ({ useAiSettingsStore: () => aiStore }));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

const loadError = ref<Error>();
const aiStore = reactive({
  settings: ref({
    defaults: { defaultTarget: null },
    connections: [],
    desktopAgents: [],
  }),
  ready: ref(true),
  loadError,
  ensureLoaded: calls.loadAi,
});

const global = {
  stubs: {
    SettingsLoadFailure: { template: '<div data-testid="settings-failure" />' },
    AiRoutingSettings: {
      template: `
        <div data-testid="routing-settings">
          <button data-testid="add-model-btn" @click="$emit('requestModelCreation', 'writing')">Add Model</button>
        </div>
      `,
      emits: ["requestModelCreation", "updateRoute"],
    },
  },
};

describe("AiRoutingSettingsPane", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    loadError.value = undefined;
    calls.loadAi.mockResolvedValue();
  });

  it("renders routing settings when the AI settings are ready", async () => {
    const wrapper = mount(AiRoutingSettingsPane, { global });
    await flushPromises();
    expect(wrapper.find('[data-testid="routing-settings"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="settings-failure"]').exists()).toBe(false);
    expect(calls.loadAi).toHaveBeenCalledOnce();
    wrapper.unmount();
  });

  it("surfaces load failure when the AI settings fail to load", async () => {
    loadError.value = new Error("Failed to load settings");
    const wrapper = mount(AiRoutingSettingsPane, { global });
    await flushPromises();
    expect(wrapper.find('[data-testid="settings-failure"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="routing-settings"]').exists()).toBe(false);
    wrapper.unmount();
  });

  it("navigates to aiProviders pane when requesting model creation", async () => {
    const wrapper = mount(AiRoutingSettingsPane, { global });
    await flushPromises();
    await wrapper.find('[data-testid="add-model-btn"]').trigger("click");
    expect(calls.routerPush).toHaveBeenCalledWith({
      path: "/settings",
      query: { pane: "aiProviders", setting: "settings-ai-providers" },
    });
    wrapper.unmount();
  });
});
