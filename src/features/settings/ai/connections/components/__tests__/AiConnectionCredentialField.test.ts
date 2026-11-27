import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";

const isSecretConfigured = vi.hoisted(() => vi.fn<(key: string) => Promise<boolean>>());
vi.mock("../../../../../../foundation/secrets/secret-store", () => ({ isSecretConfigured }));
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
  it("inspects stored credential presence without pulling plaintext into the WebView", async () => {
    isSecretConfigured.mockResolvedValue(true);
    const wrapper = open();
    await flushPromises();
    expect(isSecretConfigured).toHaveBeenCalledWith("integrations.ai.connections.primary.api-key");
    expect((wrapper.get("input").element as HTMLInputElement).value).toBe("");
    expect(wrapper.get("input").attributes("type")).toBe("password");
    expect(wrapper.get("input").attributes("placeholder")).toBe("settings.credentials.configuredPlaceholder");
    wrapper.unmount();
  });

  it("does not query an unsaved connection", async () => {
    const wrapper = open(false);
    await flushPromises();
    expect(isSecretConfigured).not.toHaveBeenCalled();
    expect(wrapper.get("input").attributes("placeholder")).toBe("settings.credentials.optionalPlaceholder");
    wrapper.unmount();
  });

  it("reports a failed inspection without pretending the credential is absent", async () => {
    isSecretConfigured.mockRejectedValue(new Error("Secret database unavailable"));
    const wrapper = open();
    await flushPromises();
    expect(wrapper.get('[role="alert"]').text()).toContain("Secret database unavailable");
    expect(wrapper.get("input").attributes("placeholder")).toBe("");
    wrapper.unmount();
  });

  it("marks a stored credential for removal on remove click and restores it on undo", async () => {
    isSecretConfigured.mockResolvedValue(true);
    const wrapper = open();
    await flushPromises();
    const remove = wrapper.findAll("button").find((button) => button.text() === "common.remove");
    await remove!.trigger("click");
    expect(wrapper.emitted("update:modelValue")?.at(-1)).toEqual([null]);
    await wrapper.setProps({ modelValue: null });
    expect(wrapper.text()).toContain("settings.credentials.removalPending");
    const undo = wrapper.findAll("button").find((button) => button.text() === "common.undo");
    await undo!.trigger("click");
    expect(wrapper.emitted("update:modelValue")?.at(-1)).toEqual([undefined]);
    wrapper.unmount();
  });
});
