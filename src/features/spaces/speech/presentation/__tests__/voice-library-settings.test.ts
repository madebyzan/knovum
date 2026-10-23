import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, expect, it, onTestFinished, vi } from "vitest";
import { defineComponent, nextTick } from "vue";
import type * as VueI18n from "vue-i18n";

import {
  settingsLeaveCoordinatorKey,
  type SettingsLeaveParticipant,
} from "../../../../../shared/contracts/settings-leave";
import type * as ipc from "../../ipc";
import VoiceLibrarySettings from "../VoiceLibrarySettings.vue";

const mocks = vi.hoisted(() => ({
  confirm: vi.fn<() => Promise<boolean>>(),
  createPresetVoice: vi.fn<typeof ipc.createPresetVoice>(),
  listVoiceRevisions: vi.fn<typeof ipc.listVoiceRevisions>(),
}));
vi.mock("../../../../../shared/ui/confirm-action", () => ({ confirmAction: mocks.confirm }));
vi.mock("../../ipc", () => ({
  createPresetVoice: mocks.createPresetVoice,
  listVoiceRevisions: mocks.listVoiceRevisions,
}));
vi.mock("vue-i18n", async (importOriginal) => ({
  ...(await importOriginal<typeof VueI18n>()),
  useI18n: () => ({ t: (key: string) => key }),
}));

const VoiceProfileAdministration = defineComponent({
  props: { creating: Boolean, expandedId: String },
  emits: ["dirty", "toggle"],
  template: '<div data-testid="administration"><slot v-if="creating" name="create" /></div>',
});
const VoiceProfileForm = defineComponent({
  props: { disabled: Boolean },
  emits: ["create", "dirty"],
  template:
    '<div><button data-testid="dirty-preset" @click="$emit(\'dirty\', true)" /><button data-testid="submit-preset" @click="$emit(\'create\', { name: \'Narrator\' })" /></div>',
});
const VoiceCandidateForm = defineComponent({ template: '<div data-testid="candidate-form" />' });

beforeEach(() => {
  vi.resetAllMocks();
  mocks.listVoiceRevisions.mockResolvedValue([]);
});

function render() {
  let participant: SettingsLeaveParticipant | undefined;
  const headerActions = document.createElement("div");
  headerActions.id = "settings-header-actions";
  document.body.append(headerActions);
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const wrapper = mount(VoiceLibrarySettings, {
    attachTo: document.body,
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
      stubs: { VoiceProfileAdministration, VoiceProfileForm, VoiceCandidateForm },
    },
  });
  onTestFinished(() => {
    wrapper.unmount();
    headerActions.remove();
    queryClient.clear();
  });
  return { wrapper, participant: () => participant! };
}

async function openCreation() {
  const add = [...document.querySelectorAll<HTMLButtonElement>("#settings-header-actions button")].find(
    (button) => button.textContent?.includes("spaces.speech.library.add"),
  );
  add!.click();
  await nextTick();
}

it("loads one application voice library without a Space selector", async () => {
  const { wrapper } = render();
  await flushPromises();
  expect(mocks.listVoiceRevisions).toHaveBeenCalledOnce();
  await openCreation();
  // Where a voice comes from is a visible choice, not a value hidden inside a dropdown.
  expect(wrapper.findAll('[role="tab"]').map((tab) => tab.text())).toEqual([
    "spaces.speech.library.presets",
    "spaces.speech.voice.clone.fromReference",
    "spaces.speech.voice.design.action",
  ]);
  expect(wrapper.find("[data-space-id]").exists()).toBe(false);
});

it("uses the settings leave decision before discarding a voice draft", async () => {
  mocks.confirm.mockResolvedValue(false);
  const { wrapper, participant } = render();
  await openCreation();
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
  await openCreation();
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
    modelSpecId: "moss-tts-nano-100m-onnx-bundled-v2",
    language: "zh-CN",
    supportsDeliveryInstruction: false,
    isActive: true,
    updatedAt: 1,
  });
  await flushPromises();
  expect(wrapper.findComponent(VoiceProfileForm).exists()).toBe(false);
});
