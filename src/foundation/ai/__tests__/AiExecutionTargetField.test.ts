import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { reactive, ref } from "vue";

import type { TaskTarget } from "../../../shared/contracts/generated/TaskTarget";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import { i18n } from "../../i18n/instance";
import type { AiSettingsPane } from "../ai-settings-navigation";
import AiExecutionTargetField from "../AiExecutionTargetField.vue";

const state = vi.hoisted(() => ({
  settings: null as null | Record<string, unknown>,
  ready: null as null | { value: boolean },
}));

vi.mock("../ai-settings-store", () => ({
  useAiSettingsStore: () => ({
    settings: ref(state.settings),
    ready: ref(state.ready?.value ?? false),
    loadError: ref(undefined),
    ensureLoaded: async () => {},
  }),
}));
vi.mock("../use-execution-catalog-query", () => ({
  useExecutionCatalogQuery: () => ({ data: ref({ agents: [] }) }),
}));
vi.mock("../ai-settings-navigation", () => ({
  openAiSettings: vi.fn<(pane: AiSettingsPane) => Promise<void>>(),
}));

const TARGET: TaskTarget = { type: "model", connectionId: "c1", modelId: "m1" };

function settingsWith(models: { id: string; capabilities: string[] }[]) {
  return reactive({
    connections: [{ id: "c1", name: "Conn", models: models.map((m) => ({ ...m, name: m.id })) }],
    desktopAgents: [],
    defaults: { writing: TARGET, defaultTarget: TARGET },
  });
}

beforeEach(() => {
  setActivePinia(createPinia());
  state.ready = ref(true);
});

describe("AiExecutionTargetField hideWhenSingle", () => {
  it("hides the select when one target is configured but still syncs the default", async () => {
    state.settings = settingsWith([{ id: "m1", capabilities: ["text-generation"] }]);
    const target = ref<TaskTarget | undefined>();
    const wrapper = mount(AiExecutionTargetField, {
      props: { purpose: "writing", compact: true, hideWhenSingle: true, modelValue: target.value },
      global: { plugins: [i18n] },
    });
    await flushPromises();
    expect(wrapper.findComponent(UiSelect).exists()).toBe(false);
    expect(wrapper.emitted("update:modelValue")?.at(-1)?.[0]).toEqual(TARGET);
    wrapper.unmount();
  });

  it("shows the select when several targets are configured", async () => {
    state.settings = settingsWith([
      { id: "m1", capabilities: ["text-generation"] },
      { id: "m2", capabilities: ["text-generation"] },
    ]);
    const wrapper = mount(AiExecutionTargetField, {
      props: { purpose: "writing", compact: true, hideWhenSingle: true },
      global: { plugins: [i18n] },
    });
    await flushPromises();
    expect(wrapper.findComponent(UiSelect).exists()).toBe(true);
    wrapper.unmount();
  });
});
