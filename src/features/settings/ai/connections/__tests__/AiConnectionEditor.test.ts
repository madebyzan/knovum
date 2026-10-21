import { flushPromises, shallowMount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import type { AiConnectionSettings } from "../../../../../shared/contracts/generated/AiConnectionSettings";
import type { ProviderDescriptor } from "../../../../../shared/contracts/generated/ProviderDescriptor";
import UiButton from "../../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../../shared/ui/UiSelect.vue";
import AiConnectionCredentialField from "../components/AiConnectionCredentialField.vue";
import AiConnectionEditor from "../components/AiConnectionEditor.vue";

vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

const providers: ProviderDescriptor[] = [
  {
    id: "openai",
    displayName: "OpenAI",
    defaultBaseUrl: "https://api.openai.com/v1",
    capabilities: ["text-generation"],
    builtinAgentProtocol: "open_ai_responses",
  },
  {
    id: "anthropic",
    displayName: "Anthropic",
    defaultBaseUrl: "https://api.anthropic.com",
    capabilities: ["text-generation"],
    builtinAgentProtocol: "anthropic",
  },
];
const connection: AiConnectionSettings = {
  id: "primary",
  name: "Primary",
  provider: "openai",
  baseUrl: providers[0]!.defaultBaseUrl,
  models: [
    { id: "text", name: "Text", model: "example", capabilities: ["text-generation"], builtinAgent: null },
  ],
};
function open(existing = true) {
  return shallowMount(AiConnectionEditor, {
    props: { open: true, providers, busy: false, ...(existing ? { connection } : {}) },
    global: { stubs: { UiDialog: { template: "<div><slot /><slot name='footer' /></div>" } } },
  });
}
function dirty(wrapper: ReturnType<typeof open>) {
  return wrapper.emitted("dirty")!.at(-1)![0];
}

describe("AI connection editor", () => {
  it("ignores whitespace-only edits and tracks changes to connection fields", async () => {
    const wrapper = open();
    const [name, url] = wrapper.findAllComponents(UiInput);
    name!.vm.$emit("update:modelValue", "  Primary  ");
    url!.vm.$emit("update:modelValue", `  ${connection.baseUrl}  `);
    await flushPromises();
    expect(dirty(wrapper)).toBe(false);
    name!.vm.$emit("update:modelValue", "Updated");
    await flushPromises();
    expect(dirty(wrapper)).toBe(true);
    await wrapper.findAllComponents(UiButton).at(-1)!.trigger("click");
    expect(wrapper.emitted("save")![0]![0]).toEqual({
      connection: { ...connection, name: "Updated" },
      secret: undefined,
    });
    expect(connection.name).toBe("Primary");
    wrapper.unmount();
  });

  it("tracks credential removal, cancellation and a trimmed replacement independently", async () => {
    const wrapper = open();
    const credential = wrapper.findComponent(AiConnectionCredentialField);
    credential.vm.$emit("update:modelValue", null);
    await flushPromises();
    expect(dirty(wrapper)).toBe(true);
    credential.vm.$emit("update:modelValue", undefined);
    await flushPromises();
    expect(dirty(wrapper)).toBe(false);
    credential.vm.$emit("update:modelValue", "  replacement  ");
    await flushPromises();
    await wrapper.findAllComponents(UiButton).at(-1)!.trigger("click");
    expect(wrapper.emitted("save")![0]![0]).toEqual({ connection, secret: "replacement" });
    wrapper.unmount();
  });

  it("tracks a new connection's provider selection and uses its owned defaults", async () => {
    const wrapper = open(false);
    wrapper.findComponent(UiSelect).vm.$emit("update:modelValue", "anthropic");
    await flushPromises();
    expect(dirty(wrapper)).toBe(true);
    await wrapper.findAllComponents(UiButton).at(-1)!.trigger("click");
    expect(wrapper.emitted("save")![0]![0]).toMatchObject({
      connection: { name: "Anthropic", provider: "anthropic", baseUrl: providers[1]!.defaultBaseUrl },
    });
    wrapper.unmount();
  });

  it("rejects URL credentials without emitting them", async () => {
    const wrapper = open();
    const url = wrapper.findAllComponents(UiInput)[1]!;
    url.vm.$emit("update:modelValue", "https://user:secret@models.example.test/v1");
    await flushPromises();
    await wrapper.findAllComponents(UiButton).at(-1)!.trigger("click");

    expect(wrapper.emitted("save")).toBeUndefined();
    expect(wrapper.text()).toContain("settings.ai.connections.invalidBaseUrl");
    expect(wrapper.text()).not.toContain("secret");
    wrapper.unmount();
  });
});
