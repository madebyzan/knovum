import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";

const isSecretConfigured = vi.hoisted(() => vi.fn<(key: string) => Promise<boolean>>());
const setSecret = vi.hoisted(() => vi.fn<(key: string, value: string) => Promise<void>>());
const deleteSecret = vi.hoisted(() => vi.fn<(key: string) => Promise<void>>());

vi.mock("../../../../foundation/secrets/secret-store", () => ({
  isSecretConfigured,
  setSecret,
  deleteSecret,
}));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));
vi.mock("vue-sonner", () => ({
  toast: { error: vi.fn<(message: string) => void>(), success: vi.fn<(message: string) => void>() },
}));

import SecretPreferenceField from "../SecretPreferenceField.vue";

describe("SecretPreferenceField", () => {
  beforeEach(() => {
    isSecretConfigured.mockReset().mockResolvedValue(false);
    setSecret.mockReset().mockResolvedValue(undefined);
    deleteSecret.mockReset().mockResolvedValue(undefined);
  });

  it("inspects stored credential presence without pulling plaintext into the input", async () => {
    isSecretConfigured.mockResolvedValue(true);
    const wrapper = mount(SecretPreferenceField, { props: { label: "API key", secretKey: "ai.key" } });
    await flushPromises();
    expect(isSecretConfigured).toHaveBeenCalledWith("ai.key");
    expect((wrapper.get("input").element as HTMLInputElement).value).toBe("");
    expect(wrapper.get("input").attributes("placeholder")).toBe("settings.credentials.configuredPlaceholder");
    expect(setSecret).not.toHaveBeenCalled();
    expect(deleteSecret).not.toHaveBeenCalled();
  });

  it("writes a trimmed replacement when input is blurred and clears the field", async () => {
    const wrapper = mount(SecretPreferenceField, { props: { label: "API key", secretKey: "ai.key" } });
    await flushPromises();
    const input = wrapper.get("input");
    await input.setValue("  replacement  ");
    await input.trigger("blur");
    await flushPromises();
    expect(setSecret).toHaveBeenCalledWith("ai.key", "replacement");
    expect((wrapper.get("input").element as HTMLInputElement).value).toBe("");
  });

  it("removes the stored credential when remove is clicked", async () => {
    isSecretConfigured.mockResolvedValue(true);
    const wrapper = mount(SecretPreferenceField, { props: { label: "API key", secretKey: "ai.key" } });
    await flushPromises();
    const remove = wrapper.findAll("button").find((button) => button.text() === "common.remove");
    await remove!.trigger("click");
    await flushPromises();
    expect(deleteSecret).toHaveBeenCalledWith("ai.key");
  });

  it("does not delete a configured credential when blurred while empty", async () => {
    isSecretConfigured.mockResolvedValue(true);
    const wrapper = mount(SecretPreferenceField, { props: { label: "API key", secretKey: "ai.key" } });
    await flushPromises();
    const input = wrapper.get("input");
    await input.setValue("");
    await input.trigger("blur");
    expect(deleteSecret).not.toHaveBeenCalled();
  });
});
