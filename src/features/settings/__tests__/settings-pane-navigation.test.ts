import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia } from "pinia";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { defineComponent, h, reactive, shallowRef } from "vue";
import { createMemoryHistory, createRouter } from "vue-router";

import { DEFAULT_CREATION_LIBRARY } from "../../../foundation/ai/schema/creation-library";
import { DEFAULT_AI_MODEL_SETTINGS } from "../../../foundation/ai/schema/model-settings";
import { i18n } from "../../../foundation/i18n/instance";
import { archiveSettingsKey, type ArchiveSettingsContract } from "../../../shared/contracts/archive-settings";
import UiSettingsHeaderActions from "../../../shared/ui/UiSettingsHeaderActions.vue";
import SettingsSidebar from "../components/SettingsSidebar.vue";
import SettingsView from "../SettingsView.vue";

vi.mock("../../../foundation/ai/ai-settings-store", () => ({ useAiSettingsStore: () => aiStore }));
vi.mock("../../../foundation/ai/creation-library-store", () => ({
  useCreationLibraryStore: () => creationStore,
}));
vi.mock("../../../foundation/ai/providers", () => ({ loadProviderDescriptors: async () => [] }));
vi.mock("../../../foundation/ai/use-execution-catalog-query", () => ({
  useExecutionCatalogQuery: () => ({
    data: shallowRef({ agents: [], installations: [] }),
    descriptorPending: shallowRef(false),
    descriptorError: shallowRef(),
    installationFetching: shallowRef(false),
    installationError: shallowRef(),
  }),
}));
vi.mock("../../../foundation/lifecycle/window-close-coordinator", () => ({
  registerWindowCloseCommit: () => () => {},
}));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => ({ label: "main" }) }));

const ready = shallowRef(false);
const loadError = shallowRef<Error>();
const aiStore = reactive({
  settings: shallowRef(DEFAULT_AI_MODEL_SETTINGS),
  ready,
  loadError,
  ensureLoaded: async () => {},
});
const creationStore = reactive({
  library: shallowRef(DEFAULT_CREATION_LIBRARY),
  ready,
  loadError: shallowRef<Error>(),
  ensureLoaded: async () => {},
  flush: async () => {},
});
const extensions = defineComponent({
  render: () =>
    h("section", { "data-testid": "extensions-pane" }, [
      h(UiSettingsHeaderActions, null, {
        default: () => h("button", { "data-testid": "extensions-header-action" }, "Action"),
      }),
    ]),
});

beforeEach(() => {
  ready.value = false;
  loadError.value = undefined;
});

it.each(["loading", "ready", "failed"])(
  "keeps other settings reachable after leaving models in the %s state",
  async (state) => {
    const returnToWorkspace = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);
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
            returnToWorkspace,
          },
        },
      ],
    });
    await router.push("/settings?pane=appearance");
    const warnings = vi.fn<(message: string) => void>();
    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const archive: ArchiveSettingsContract = {
      spaces: shallowRef([{ id: "archived-space", title: "Archived space" }]),
      initialize: async () => {},
      restore: async () => {},
      permanentlyDelete: async () => {},
      permanentlyDeleteAll: async () => {},
    };
    const wrapper = mount(defineComponent({ template: "<router-view />" }), {
      attachTo: document.body,
      global: {
        config: { warnHandler: warnings },
        plugins: [createPinia(), i18n, router, [VueQueryPlugin, { queryClient }]],
        provide: { [archiveSettingsKey as symbol]: archive },
        stubs: {
          Transition: false,
          UiScrollArea: { template: "<div><slot /></div>" },
          AiConnectionsSettings: { template: '<section data-testid="model-connections" />' },
          AiRoutingSettings: true,
          PythonRuntimeSettings: { template: '<section data-testid="python-runtimes" />' },
          LocalModelsSettings: { template: '<section data-testid="local-models-pane" />' },
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
    expect(wrapper.find("#settings-background-task-notifications").exists()).toBe(false);
    /* oxlint-disable no-await-in-loop -- Each navigation must finish before the next settings switch. */
    for (let cycle = 0; cycle < 2; cycle++) {
      await router.replace({ path: "/settings", query: { pane: "aiProviders" } });
      expect(router.currentRoute.value.query.pane).toBe("aiProviders");
      await flushPromises();
      await vi.waitFor(() =>
        expect(wrapper.find('[data-testid="ai-providers-settings-pane"]').exists()).toBe(true),
      );
      expect(wrapper.find('[data-testid="python-runtimes"]').exists()).toBe(false);
      expect(wrapper.find('[data-testid="model-connections"]').exists()).toBe(state === "ready");
      expect(wrapper.find('[role="status"]').exists()).toBe(state === "loading");
      expect(wrapper.text().includes("Unable to load model settings")).toBe(state === "failed");
      for (const [pane, selector] of [
        ["aiRouting", '[data-testid="ai-routing-settings-pane"]'],
        ["localModels", '[data-testid="local-models-pane"]'],
        ["appearance", "#settings-appearance-theme"],
        ["notifications", "#settings-background-task-notifications"],
        ["extensions", '[data-testid="extensions-pane"]'],
        ["desktopAgents", "#settings-header-actions button"],
        ["archivedSpaces", '[data-testid="archive-restore-space"]'],
        ["creation", '[data-testid="creation-pane"]'],
        ["usage", '[data-testid="usage-pane"]'],
      ] as const) {
        await router.replace({ path: "/settings", query: { pane } });
        await flushPromises();
        await vi.waitFor(() => {
          expect(wrapper.getComponent(SettingsSidebar).props("activePane")).toBe(pane);
          expect(wrapper.find(selector).exists()).toBe(true);
          expect(
            wrapper.get("#settings-header-actions").find('[data-testid="extensions-header-action"]').exists(),
          ).toBe(pane === "extensions");
          expect(wrapper.get("#settings-header-actions").findAll("button")).toHaveLength(
            ["extensions", "desktopAgents", "archivedSpaces"].includes(pane) ? 1 : 0,
          );
        });
      }
    }
    /* oxlint-enable no-await-in-loop */
    wrapper.getComponent(SettingsSidebar).vm.$emit("back");
    expect(returnToWorkspace).toHaveBeenCalledExactlyOnceWith();
    expect(warnings.mock.calls.filter(([message]) => message.includes("non-element root node"))).toEqual([]);
  },
);
