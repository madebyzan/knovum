import { mount } from "@vue/test-utils";
import { expect, it, vi } from "vitest";
import { defineComponent } from "vue";

import { DEFAULT_AI_MODEL_SETTINGS } from "../../../foundation/ai/schema/model-settings";
import AiRoutingSettings from "../components/AiRoutingSettings.vue";

vi.mock("../../../foundation/ai/use-execution-catalog-query", () => ({
  useExecutionCatalogQuery: () => ({ data: { value: { agents: [] } } }),
}));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

it("selects the exact configured target with separator-containing identities and clears the route", () => {
  const settings = structuredClone(DEFAULT_AI_MODEL_SETTINGS);
  settings.connections = [
    {
      id: "provider:v1",
      name: "Provider",
      provider: "openai",
      baseUrl: "https://example.test/v1",
      models: [{ id: "model:chat", name: "Chat", model: "chat", capabilities: ["text-generation"] }],
    },
  ];
  const select = defineComponent({
    name: "UiSelect",
    props: ["modelValue", "options", "label"],
    emits: ["update:modelValue"],
    template: "<button />",
  });
  const wrapper = mount(AiRoutingSettings, {
    props: { defaults: settings.defaults, connections: settings.connections, desktopAgents: [] },
    global: { stubs: { UiSelect: select, UiSettingsSection: { template: "<section><slot /></section>" } } },
  });
  try {
    const selector = wrapper.findComponent({ name: "UiSelect" });
    const target = { type: "model" as const, connectionId: "provider:v1", modelId: "model:chat" };
    selector.vm.$emit("update:modelValue", JSON.stringify(target));
    selector.vm.$emit("update:modelValue", "__unassigned__");
    expect(wrapper.emitted("updateRoute")).toEqual([
      ["defaultTarget", target],
      ["defaultTarget", null],
    ]);
    expect(settings.defaults.defaultTarget).toBeNull();
  } finally {
    wrapper.unmount();
  }
});
