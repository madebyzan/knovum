import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia } from "pinia";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { defineComponent, h, reactive, shallowRef } from "vue";
import { createMemoryHistory, createRouter } from "vue-router";

import { defaultCreationLibrary } from "../../../foundation/ai/schema/creation-library";
import { defaultAiModelSettings } from "../../../foundation/ai/schema/model-settings";
import { i18n } from "../../../foundation/i18n/i18n";
import SettingsView from "../SettingsView.vue";

vi.mock("../../../foundation/ai/ai-settings-store", () => ({ useAiSettingsStore: () => aiStore }));
vi.mock("../../../foundation/ai/creation-library-store", () => ({
  useCreationLibraryStore: () => creationStore,
}));
vi.mock("../../../foundation/ai/providers", () => ({ loadProviderDescriptors: async () => [] }));
vi.mock("../../../foundation/lifecycle/window-close-coordinator", () => ({
  registerWindowCloseCommit: () => () => {},
}));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => ({ label: "main" }) }));

const ready = shallowRef(false);
const loadError = shallowRef<Error>();
const aiStore = reactive({
  settings: shallowRef(defaultAiModelSettings),
  ready,
  loadError,
  ensureLoaded: async () => {},
});
const creationStore = reactive({
  library: shallowRef(defaultCreationLibrary),
  ready,
  loadError: shallowRef<Error>(),
  ensureLoaded: async () => {},
  flush: async () => {},
});
const extensions = defineComponent({ render: () => h("section", { "data-testid": "extensions-pane" }) });

beforeEach(() => {
  ready.value = false;
  loadError.value = undefined;
});

it.each(["loading", "ready", "failed"])(
  "keeps other settings reachable after leaving models in the %s state",
  async (state) => {
    ready.value = state === "ready";
    if (state === "failed") loadError.value = new Error("Unable to load model settings");
    const router = createRouter({
      history: createMemoryHistory(),
      routes: [
        {
          path: "/settings",
          component: SettingsView,
          props: {
            loadGenerationActions: async () => [],
            extensionsComponent: extensions,
            voiceLibraryComponent: extensions,
          },
        },
      ],
    });
    await router.push("/settings?pane=appearance");
    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const wrapper = mount(defineComponent({ template: "<router-view />" }), {
      global: {
        plugins: [createPinia(), i18n, router, [VueQueryPlugin, { queryClient }]],
        stubs: {
          Transition: { template: "<div><slot /></div>" },
          UiScrollArea: { template: "<div><slot /></div>" },
          AiConnectionsSettings: { template: '<section data-testid="model-connections" />' },
          AiRoutingSettings: true,
          PythonRuntimeSettings: { template: '<section data-testid="python-runtimes" />' },
          LocalModelsSettings: true,
          CreationSettingsPane: { template: '<section data-testid="creation-pane" />' },
          UsageSettingsPane: { template: '<section data-testid="usage-pane" />' },
        },
      },
    });
    onTestFinished(() => {
      wrapper.unmount();
      queryClient.clear();
    });
    await flushPromises();
    expect(wrapper.find("#settings-appearance-theme").exists()).toBe(true);
    /* oxlint-disable no-await-in-loop -- Each navigation must finish before the next settings switch. */
    for (let cycle = 0; cycle < 2; cycle++) {
      await router.replace({ path: "/settings", query: { pane: "aiModels" } });
      expect(router.currentRoute.value.query.pane).toBe("aiModels");
      await flushPromises();
      await vi.waitFor(() =>
        expect(wrapper.find('[data-testid="ai-models-settings-pane"]').exists()).toBe(true),
      );
      expect(wrapper.find('[data-testid="python-runtimes"]').exists()).toBe(false);
      expect(wrapper.find('[data-testid="model-connections"]').exists()).toBe(state === "ready");
      expect(wrapper.find('[role="status"]').exists()).toBe(state === "loading");
      expect(wrapper.text().includes("Unable to load model settings")).toBe(state === "failed");
      for (const [pane, selector] of [
        ["appearance", "#settings-appearance-theme"],
        ["extensions", '[data-testid="extensions-pane"]'],
        ["creation", '[data-testid="creation-pane"]'],
        ["usage", '[data-testid="usage-pane"]'],
      ] as const) {
        await router.replace({ path: "/settings", query: { pane } });
        await flushPromises();
        await vi.waitFor(() => expect(wrapper.find(selector).exists()).toBe(true));
      }
    }
    /* oxlint-enable no-await-in-loop */
  },
);
