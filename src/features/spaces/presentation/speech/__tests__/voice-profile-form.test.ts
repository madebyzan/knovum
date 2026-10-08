import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { expect, it, onTestFinished, vi } from "vitest";

import type { listPresetVoices } from "../../../speech/ipc";
import VoiceProfileForm from "../VoiceProfileForm.vue";

const mocks = vi.hoisted(() => ({ presets: vi.fn<typeof listPresetVoices>() }));
vi.mock("../../../speech/ipc", () => ({ listPresetVoices: mocks.presets }));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));
it("starts with a named preset when no custom library name is entered", async () => {
  mocks.presets.mockResolvedValue([
    { speakerId: "0", name: "Default speaker", primaryLanguage: "zh-CN", gender: "female" },
  ]);
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const wrapper = mount(VoiceProfileForm, {
    props: { spaceId: "space" },
    global: { plugins: [[VueQueryPlugin, { queryClient }]], stubs: { VoiceAudition: true } },
  });
  onTestFinished(() => {
    wrapper.unmount();
    queryClient.clear();
  });
  await flushPromises();
  await wrapper.get("form").trigger("submit");
  expect(wrapper.emitted("create")?.[0]?.[0]).toEqual(
    expect.objectContaining({ name: "Default speaker", presetVoiceId: "0", engineId: "moss_nano" }),
  );
  await wrapper.get("input").setValue("Narrator");
  await wrapper.get("form").trigger("submit");
  expect(wrapper.emitted("create")?.[1]?.[0]).toEqual(expect.objectContaining({ name: "Narrator" }));
});
