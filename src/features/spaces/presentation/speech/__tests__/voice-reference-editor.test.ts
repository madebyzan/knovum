import { flushPromises, mount } from "@vue/test-utils";
import { expect, it, onTestFinished, vi } from "vitest";
import { defineComponent } from "vue";

import { createAppError } from "../../../../../foundation/errors/app-error";
import { i18n } from "../../../../../foundation/i18n/i18n";
import type { PreparedVoiceReference } from "../../../contracts/generated/PreparedVoiceReference";
import type { prepareVoiceReference } from "../../../speech/voice-reference-ipc";
import VoiceReferenceEditor from "../VoiceReferenceEditor.vue";

const mocks = vi.hoisted(() => ({
  open: vi.fn<() => Promise<string>>(),
  authorize: vi.fn<(path: string) => Promise<string>>(),
  prepare: vi.fn<typeof prepareVoiceReference>(),
  cancel: vi.fn<(jobId: string) => Promise<void>>(),
}));
vi.mock("@tauri-apps/api/core", () => ({ convertFileSrc: (path: string) => path }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: mocks.open }));
vi.mock("../../../speech/ipc", () => ({ cancelSpeechGeneration: mocks.cancel }));
vi.mock("../../../speech/voice-reference-ipc", () => ({
  authorizeVoiceReferenceSource: mocks.authorize,
  prepareVoiceReference: mocks.prepare,
}));
const Player = defineComponent({ name: "SpeechMediaPlayer", emits: ["loaded"], template: "<div />" });

function render() {
  vi.clearAllMocks();
  mocks.authorize.mockImplementation(async (path) => path);
  mocks.cancel.mockResolvedValue();
  const wrapper = mount(VoiceReferenceEditor, {
    props: { spaceId: "space", language: "en", disabled: false },
    global: { plugins: [i18n], stubs: { SpeechMediaPlayer: Player } },
  });
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}

it("resets a replacement source and rejects a selection until its actual duration is known", async () => {
  const wrapper = render();
  mocks.open.mockResolvedValueOnce("/first.wav").mockResolvedValueOnce("/second.wav");
  await wrapper.get('[data-testid="reference-choose"]').trigger("click");
  await flushPromises();
  expect(wrapper.get('[data-testid="reference-prepare"]').attributes("disabled")).toBeDefined();
  wrapper.findComponent(Player).vm.$emit("loaded", 120_000);
  await flushPromises();
  await wrapper.get('[data-testid="reference-start"]').setValue("60");
  await wrapper.get('[data-testid="reference-end"]').setValue("75");
  expect(wrapper.get('[data-testid="reference-prepare"]').attributes("disabled")).toBeUndefined();
  await wrapper.get('[data-testid="reference-end"]').setValue("135");
  expect(wrapper.get('[data-testid="reference-prepare"]').attributes("disabled")).toBeDefined();
  await wrapper.get('[data-testid="reference-choose"]').trigger("click");
  await flushPromises();
  expect(wrapper.get<HTMLInputElement>('[data-testid="reference-start"]').element.value).toBe("0");
  expect(wrapper.get('[data-testid="reference-prepare"]').attributes("disabled")).toBeDefined();
  wrapper.findComponent(Player).vm.$emit("loaded", 4000);
  await flushPromises();
  expect(wrapper.get('[data-testid="reference-prepare"]').attributes("disabled")).toBeDefined();
  expect(wrapper.find('[role="status"]').exists()).toBe(true);
});

it("releases parent busy state when a preparing editor is closed", async () => {
  const wrapper = render();
  mocks.open.mockResolvedValue("/reference.wav");
  let finish!: (value: PreparedVoiceReference) => void;
  mocks.prepare.mockReturnValue(
    new Promise((resolve) => {
      finish = resolve;
    }),
  );
  await wrapper.get('[data-testid="reference-choose"]').trigger("click");
  await flushPromises();
  wrapper.findComponent(Player).vm.$emit("loaded", 10_000);
  await flushPromises();
  await wrapper.get('[data-testid="reference-prepare"]').trigger("click");
  await flushPromises();
  expect(wrapper.emitted("busy")?.at(-1)).toEqual([true]);
  wrapper.unmount();
  expect(wrapper.emitted("busy")?.at(-1)).toEqual([false]);
  expect(mocks.cancel).toHaveBeenCalledOnce();
  finish({
    sourcePath: "/prepared.wav",
    sourceRevision: "sha256:source",
    startMs: 0,
    endMs: 10_000,
    transcript: "Reference",
  });
  await flushPromises();
  expect(wrapper.emitted("update:modelValue")).toBeUndefined();
});

it("reloads metadata when the same file is selected again", async () => {
  const wrapper = render();
  mocks.open.mockResolvedValue("/reference.wav");
  await wrapper.get('[data-testid="reference-choose"]').trigger("click");
  await flushPromises();
  wrapper.findComponent(Player).vm.$emit("loaded", 10_000);
  await flushPromises();
  expect(wrapper.get('[data-testid="reference-prepare"]').attributes("disabled")).toBeUndefined();
  await wrapper.get('[data-testid="reference-choose"]').trigger("click");
  await flushPromises();
  expect(wrapper.get('[data-testid="reference-prepare"]').attributes("disabled")).toBeDefined();
  wrapper.findComponent(Player).vm.$emit("loaded", 10_000);
  await flushPromises();
  expect(wrapper.get('[data-testid="reference-prepare"]').attributes("disabled")).toBeUndefined();
});

it.each([
  ["cancelled", false],
  ["operation_failed", true],
] as const)("settles %s with the appropriate error presentation", async (code, alert) => {
  const wrapper = render();
  mocks.open.mockResolvedValue("/reference.wav");
  mocks.prepare.mockRejectedValue(createAppError(code, "Reference preparation stopped"));
  await wrapper.get('[data-testid="reference-choose"]').trigger("click");
  await flushPromises();
  wrapper.findComponent(Player).vm.$emit("loaded", 10_000);
  await flushPromises();
  await wrapper.get('[data-testid="reference-prepare"]').trigger("click");
  await flushPromises();
  expect(wrapper.find('[role="alert"]').exists()).toBe(alert);
  expect(wrapper.emitted("busy")?.at(-1)).toEqual([false]);
  expect(wrapper.get('[data-testid="reference-prepare"]').attributes("disabled")).toBeUndefined();
});

it("does not publish an old reference authorization error into a different Space", async () => {
  const wrapper = render();
  mocks.open.mockResolvedValue("/reference.wav");
  let failAuthorization!: (error: Error) => void;
  mocks.authorize.mockReturnValue(
    new Promise((_resolve, reject) => {
      failAuthorization = reject;
    }),
  );
  await wrapper.get('[data-testid="reference-choose"]').trigger("click");
  await flushPromises();
  await wrapper.setProps({ spaceId: "different-space" });
  failAuthorization(new Error("Old authorization failed"));
  await flushPromises();
  expect(wrapper.find('[role="alert"]').exists()).toBe(false);
  expect(wrapper.findComponent(Player).exists()).toBe(false);
});
