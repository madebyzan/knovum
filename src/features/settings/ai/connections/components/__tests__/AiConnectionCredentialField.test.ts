import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";

const getSecret = vi.hoisted(() => vi.fn<(key: string) => Promise<string | null>>());
vi.mock("../../../../../../foundation/secrets/secret-store", () => ({ getSecret }));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

import AiConnectionCredentialField from "../AiConnectionCredentialField.vue";

beforeEach(() => {
  getSecret.mockReset().mockResolvedValue(null);
});
function open(existing = true) {
  return mount(AiConnectionCredentialField, {
    props: { modelValue: undefined, connectionId: "primary", existing, disabled: false },
  });
}

describe("AI credential configuration", () => {
  it("loads the stored credential into the field for editing", async () => {
    getSecret.mockResolvedValue("stored-key");
    const wrapper = open();
    await flushPromises();
    expect(getSecret).toHaveBeenCalledWith("integrations.ai.connections.primary.api-key");
    expect((wrapper.get("input").element as HTMLInputElement).value).toBe("stored-key");
    expect(wrapper.get("input").attributes("type")).toBe("password");
    expect(wrapper.find('button[aria-label="common.showSecret"]').exists()).toBe(true);
    wrapper.unmount();
  });

  it("does not query an unsaved connection", async () => {
    const wrapper = open(false);
    await flushPromises();
    expect(getSecret).not.toHaveBeenCalled();
    expect(wrapper.get("input").attributes("placeholder")).toBe("settings.credentials.optionalPlaceholder");
    wrapper.unmount();
  });

  it("reports a failed load without pretending the credential is absent", async () => {
    getSecret.mockRejectedValue(new Error("Secret database unavailable"));
    const wrapper = open();
    await flushPromises();
    expect(wrapper.get('[role="alert"]').text()).toContain("Secret database unavailable");
    expect(wrapper.get("input").attributes("placeholder")).toBe("");
    wrapper.unmount();
  });

  it("marks a cleared stored credential for removal and restores it on undo", async () => {
    getSecret.mockResolvedValue("stored-key");
    const wrapper = open();
    await flushPromises();
    await wrapper.get("input").setValue("");
    expect(wrapper.emitted("update:modelValue")?.at(-1)).toEqual([null]);
    expect(wrapper.text()).toContain("settings.credentials.removalPending");
    const undo = wrapper.findAll("button").find((button) => button.text() === "common.undo");
    await undo!.trigger("click");
    expect(wrapper.emitted("update:modelValue")?.at(-1)).toEqual([undefined]);
    wrapper.unmount();
  });
});
