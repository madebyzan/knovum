import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";

const isSecretConfigured = vi.hoisted(() => vi.fn<(key: string) => Promise<boolean>>());
vi.mock("../../../../foundation/secrets/secret-store", () => ({ isSecretConfigured }));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

import AiConnectionCredentialField from "../AiConnectionCredentialField.vue";

beforeEach(() => {
  isSecretConfigured.mockReset().mockResolvedValue(false);
});
function open(existing = true) {
  return mount(AiConnectionCredentialField, {
    props: { modelValue: undefined, connectionId: "primary", existing, disabled: false },
  });
}

describe("AI credential configuration", () => {
  it("displays stored configuration without loading a credential value", async () => {
    isSecretConfigured.mockResolvedValue(true);
    const wrapper = open();
    await flushPromises();
    expect(isSecretConfigured).toHaveBeenCalledWith("integrations.ai.connections.primary.api-key");
    expect(wrapper.text()).toContain("common.saved");
    expect(wrapper.get("input").element.value).toBe("");
    wrapper.unmount();
  });

  it("does not query an unsaved connection", async () => {
    const wrapper = open(false);
    await flushPromises();
    expect(isSecretConfigured).not.toHaveBeenCalled();
    expect(wrapper.get("input").attributes("placeholder")).toBe("settings.credentials.optionalPlaceholder");
    wrapper.unmount();
  });

  it("reports a failed state query without pretending the credential is absent", async () => {
    isSecretConfigured.mockRejectedValue(new Error("Secret database unavailable"));
    const wrapper = open();
    await flushPromises();
    expect(wrapper.get('[role="alert"]').text()).toContain("Secret database unavailable");
    expect(wrapper.get("input").attributes("placeholder")).toBe("");
    wrapper.unmount();
  });
});
