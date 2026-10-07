import { mount } from "@vue/test-utils";
import { expect, it, vi } from "vitest";
import { defineComponent, ref } from "vue";

import { defaultAiModelSettings } from "../../../foundation/ai/schema/model-settings";
import AiRoutingSettings from "../components/AiRoutingSettings.vue";

vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));
vi.mock("../../../foundation/ai/execution-catalog-query", () => ({
  useExecutionCatalogQuery: () => ({ data: ref({ agents: [] }) }),
}));
const Select = defineComponent({ emits: ["update:modelValue"], setup: () => () => null });

it("emits a route mutation without changing the authoritative settings object", () => {
  const settings = structuredClone(defaultAiModelSettings);
  settings.desktopAgents = [{ kind: "codex", enabled: true }];
  settings.defaults.defaultTarget = { type: "agent", runtimeKind: "codex" };
  const previous = structuredClone(settings.defaults);
  const wrapper = mount(AiRoutingSettings, {
    props: { defaults: settings.defaults, connections: [], desktopAgents: settings.desktopAgents },
    global: { stubs: { UiSelect: Select, UiSettingsSection: { template: "<div><slot /></div>" } } },
  });
  wrapper.findAllComponents(Select)[0]!.vm.$emit("update:modelValue", "__unassigned__");
  expect(settings.defaults).toEqual(previous);
  expect(wrapper.emitted("updateRoute")).toEqual([["defaultTarget", null]]);
  wrapper.unmount();
});
