import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";

const getSecret = vi.hoisted(() => vi.fn<(key: string) => Promise<string | null>>());
const setSecret = vi.hoisted(() => vi.fn<(key: string, value: string) => Promise<void>>());
const deleteSecret = vi.hoisted(() => vi.fn<(key: string) => Promise<void>>());

vi.mock("../../../../foundation/secrets/secret-store", () => ({
  getSecret,
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
    getSecret.mockReset().mockResolvedValue(null);
    setSecret.mockReset().mockResolvedValue(undefined);
    deleteSecret.mockReset().mockResolvedValue(undefined);
  });

  it("loads the stored credential without changing it when mounted", async () => {
    getSecret.mockResolvedValue("stored-key");
    const wrapper = mount(SecretPreferenceField, { props: { label: "API key", secretKey: "ai.key" } });
    await flushPromises();
    expect(getSecret).toHaveBeenCalledWith("ai.key");
    expect((wrapper.get("input").element as HTMLInputElement).value).toBe("stored-key");
    expect(setSecret).not.toHaveBeenCalled();
    expect(deleteSecret).not.toHaveBeenCalled();
  });

  it("writes a trimmed replacement when input is blurred", async () => {
    const wrapper = mount(SecretPreferenceField, { props: { label: "API key", secretKey: "ai.key" } });
    await flushPromises();
    const input = wrapper.get("input");
    await input.setValue("  replacement  ");
    await input.trigger("blur");
    expect(setSecret).toHaveBeenCalledWith("ai.key", "replacement");
  });

  it("removes the stored credential when the field is cleared and blurred", async () => {
    getSecret.mockResolvedValue("stored-key");
    const wrapper = mount(SecretPreferenceField, { props: { label: "API key", secretKey: "ai.key" } });
    await flushPromises();
    const input = wrapper.get("input");
    await input.setValue("");
    await input.trigger("blur");
    expect(deleteSecret).toHaveBeenCalledWith("ai.key");
  });

  it("reveals the stored credential without saving it", async () => {
    getSecret.mockResolvedValue("stored-key");
    const wrapper = mount(SecretPreferenceField, { props: { label: "API key", secretKey: "ai.key" } });
    await flushPromises();
    await wrapper.get('button[aria-label="common.showSecret"]').trigger("click");
    expect(wrapper.get("input").attributes("type")).toBe("text");
    expect((wrapper.get("input").element as HTMLInputElement).value).toBe("stored-key");
    expect(setSecret).not.toHaveBeenCalled();
  });
});
