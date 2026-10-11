import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { defineComponent, nextTick } from "vue";
import type * as VueI18n from "vue-i18n";

import {
  settingsLeaveCoordinatorKey,
  type SettingsLeaveParticipant,
} from "../../../../../shared/contracts/settings-leave";
import type * as ipc from "../../../speech/ipc";
import VoiceLibrarySettings from "../VoiceLibrarySettings.vue";

const mocks = vi.hoisted(() => ({
  confirm: vi.fn<() => Promise<boolean>>(),
  createPresetVoice: vi.fn<typeof ipc.createPresetVoice>(),
  listVoiceRevisions: vi.fn<typeof ipc.listVoiceRevisions>(),
}));
vi.mock("../../../../../shared/ui/confirm-action", () => ({ confirmAction: mocks.confirm }));
vi.mock("../../../speech/ipc", () => ({
  createPresetVoice: mocks.createPresetVoice,
  listVoiceRevisions: mocks.listVoiceRevisions,
}));
vi.mock("vue-i18n", async (importOriginal) => ({
  ...(await importOriginal<typeof VueI18n>()),
  useI18n: () => ({ t: (key: string) => key }),
}));

const VoiceManagement = defineComponent({
  props: { creating: Boolean, expandedId: String },
  emits: ["dirty", "toggle"],
  template: '<div data-testid="management"><slot v-if="creating" name="create" /></div>',
});
const VoiceProfileForm = defineComponent({
  emits: ["create", "dirty"],
  template:
    '<button data-testid="dirty-preset" @click="$emit(\'dirty\', true)" /><button data-testid="submit-preset" @click="$emit(\'create\', { name: \'Narrator\' })" />',
});
const VoiceCandidateForm = defineComponent({ template: '<div data-testid="candidate-form" />' });

beforeEach(() => {
  vi.resetAllMocks();
  mocks.listVoiceRevisions.mockResolvedValue([]);
});

function render() {
  let participant: SettingsLeaveParticipant | undefined;
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const wrapper = mount(VoiceLibrarySettings, {
    global: {
      plugins: [[VueQueryPlugin, { queryClient }]],
      provide: {
        [settingsLeaveCoordinatorKey as symbol]: {
          register(value: SettingsLeaveParticipant) {
            participant = value;
            return vi.fn<() => void>();
          },
          requestLeave: vi.fn<() => Promise<void>>(),
        },
      },
      stubs: { VoiceManagement, VoiceProfileForm, VoiceCandidateForm },
    },
  });
  onTestFinished(() => {
    wrapper.unmount();
    queryClient.clear();
  });
  return { wrapper, participant: () => participant! };
}

async function openCreation(wrapper: ReturnType<typeof render>["wrapper"]) {
  const add = wrapper.findAll("button").find((button) => button.text().includes("spaces.speech.voice.add"));
  await add!.trigger("click");
}

it("loads one application voice library without a Space selector", async () => {
  const { wrapper } = render();
  await flushPromises();
  expect(mocks.listVoiceRevisions).toHaveBeenCalledOnce();
  await openCreation(wrapper);
  const source = wrapper.getComponent({ name: "UiSelect" });
  expect(source.props("options")).toHaveLength(3);
  expect(wrapper.find("[data-space-id]").exists()).toBe(false);
});

it("uses the settings leave decision before discarding a voice draft", async () => {
  mocks.confirm.mockResolvedValue(false);
  const { wrapper, participant } = render();
  await openCreation(wrapper);
  await wrapper.get('[data-testid="dirty-preset"]').trigger("click");
  await expect(participant().prepare()).resolves.toBe(false);
  expect(mocks.confirm).toHaveBeenCalledOnce();
  expect(wrapper.findComponent(VoiceProfileForm).exists()).toBe(true);
  mocks.confirm.mockResolvedValue(true);
  await expect(participant().prepare()).resolves.toBe(true);
  participant().finalize?.();
  await nextTick();
  expect(wrapper.findComponent(VoiceProfileForm).exists()).toBe(false);
});

it("waits for a submitted preset before closing the creation row", async () => {
  let finish!: (voice: Awaited<ReturnType<typeof ipc.createPresetVoice>>) => void;
  mocks.createPresetVoice.mockReturnValue(new Promise((resolve) => (finish = resolve)));
  const { wrapper } = render();
  await openCreation(wrapper);
  await wrapper.get('[data-testid="submit-preset"]').trigger("click");
  await wrapper.get('button[aria-label="common.close"]').trigger("click");
  await nextTick();
  expect(wrapper.findComponent(VoiceProfileForm).exists()).toBe(true);
  finish({
    profileId: "voice",
    revisionId: "revision",
    revision: 1,
    name: "Narrator",
    sourceKind: "preset",
    engineId: "moss_nano",
    modelSpecId: "moss-tts-nano-100m-onnx-native-v2",
    language: "zh-CN",
    supportsDeliveryInstruction: false,
    isActive: true,
    updatedAt: 1,
  });
  await flushPromises();
  expect(wrapper.findComponent(VoiceProfileForm).exists()).toBe(false);
});
