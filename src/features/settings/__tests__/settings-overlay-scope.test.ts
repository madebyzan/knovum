import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia } from "pinia";
import { expect, it, onTestFinished, vi } from "vitest";
import { computed, defineComponent, shallowRef } from "vue";
import { createMemoryHistory, createRouter } from "vue-router";

import { DEFAULT_AI_MODEL_SETTINGS } from "../../../foundation/ai/schema/model-settings";
import { i18n } from "../../../foundation/i18n/instance";
import { archiveSettingsKey, type ArchiveSettingsContract } from "../../../shared/contracts/archive-settings";
import { overlayScopeVisibleKey } from "../../../shared/contracts/overlay-scope-visibility";
import AgentConnectionHelp from "../desktop-agents/components/AgentConnectionHelp.vue";
import SettingsView from "../SettingsView.vue";

vi.mock("../../../foundation/ai/ai-settings-store", () => ({ useAiSettingsStore: () => aiStore }));
vi.mock("../../../foundation/ai/providers", () => ({ loadProviderDescriptors: async () => [] }));
vi.mock("../../../foundation/ai/use-execution-catalog-query", () => ({
  useExecutionCatalogQuery: () => ({
    data: shallowRef({
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
      installations: [{ kind: "claude_code", detected: false, executablePath: null }],
    }),
    descriptorPending: shallowRef(false),
    descriptorError: shallowRef(),
    installationFetching: shallowRef(false),
    installationError: shallowRef(),
    refetchDescriptors: async () => {},
  }),
}));
vi.mock("../../../foundation/lifecycle/window-close-coordinator", () => ({
  registerWindowCloseCommit: () => () => {},
}));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => ({ label: "main" }) }));

const aiStore = {
  settings: shallowRef(DEFAULT_AI_MODEL_SETTINGS),
  ready: shallowRef(true),
  loadError: shallowRef<Error>(),
  ensureLoaded: async () => {},
};

it("renders connection help while the shell reports the workspace inactive", async () => {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      {
        path: "/settings",
        component: SettingsView,
        props: {
          extensionsComponent: defineComponent({ render: () => null }),
          voiceLibraryComponent: defineComponent({ render: () => null }),
          returnToWorkspace: async () => {},
        },
      },
    ],
  });
  await router.push("/settings?pane=desktopAgents");
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const archive: ArchiveSettingsContract = {
    spaces: shallowRef([]),
    initialize: async () => {},
    restore: async () => {},
    permanentlyDelete: async () => {},
    permanentlyDeleteAll: async () => {},
  };
  const wrapper = mount(defineComponent({ template: "<router-view />" }), {
    attachTo: document.body,
    global: {
      plugins: [createPinia(), i18n, router, [VueQueryPlugin, { queryClient }]],
      provide: {
        [archiveSettingsKey as symbol]: archive,
        [overlayScopeVisibleKey as symbol]: computed(() => false),
      },
      stubs: {
        Transition: false,
        UiScrollArea: { template: "<div><slot /></div>" },
      },
    },
  });
  onTestFinished(() => {
    wrapper.unmount();
    queryClient.clear();
  });
  await flushPromises();

  const help = wrapper.getComponent(AgentConnectionHelp);
  const trigger = help.find("button");
  expect(trigger.exists()).toBe(true);
  expect(trigger.text()).not.toBe("");
});
