import { flushPromises, shallowMount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
import { reactive } from "vue";

import type { GenerationPreset } from "../../../foundation/ai/schema/models";
import {
  settingsLeaveCoordinatorKey,
  type SettingsLeaveParticipant,
} from "../../../shared/contracts/settings-leave";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import GenerationPresetEditor from "../components/GenerationPresetEditor.vue";

vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));

function leaveContext(capture?: (participant: SettingsLeaveParticipant) => void) {
  return {
    [settingsLeaveCoordinatorKey as symbol]: {
      register(participant: SettingsLeaveParticipant) {
        capture?.(participant);
        return () => {};
      },
      requestLeave: async () => {},
    },
  };
}

describe("generation preset editor", () => {
  it("opens, edits and saves a reactive preset without mutating the source", async () => {
    const savePreset = vi.fn<(preset: GenerationPreset) => Promise<void>>(async (_preset) => {});
    const source = reactive<GenerationPreset>({
      id: "preset",
      name: "Original",
      actionId: "generate_image",
      target: { type: "agent", runtimeKind: "codex" },
      parameters: { width: 1024 },
    });
    const wrapper = shallowMount(GenerationPresetEditor, {
      props: {
        open: true,
        preset: source,
        savePreset,
        loadGenerationActions: async () => [{ id: "generate_image", routingPurpose: "image-generation" }],
      },
      global: {
        provide: leaveContext(),
        stubs: { UiDialog: { template: "<div><slot /><slot name='footer' /></div>" } },
      },
    });
    await flushPromises();
    wrapper.findComponent(UiInput).vm.$emit("update:modelValue", "  Edited  ");
    await wrapper.findAllComponents(UiButton).at(-1)!.trigger("click");
    const saved = savePreset.mock.calls[0]![0];
    expect(saved).toEqual({ ...source, name: "Edited" });
    expect(() => structuredClone(saved)).not.toThrow();
    expect(source.name).toBe("Original");
    expect(wrapper.emitted("update:open")).toContainEqual([false]);
    await wrapper.setProps({ open: false });
    await wrapper.setProps({ open: true });
    expect(wrapper.findComponent(UiInput).props("modelValue")).toBe("Original");
    wrapper.unmount();
  });

  it("waits for an admitted save and keeps the editor open when it fails", async () => {
    let participant!: SettingsLeaveParticipant;
    let rejectSave!: (error: Error) => void;
    const savePreset = vi.fn<(preset: GenerationPreset) => Promise<void>>(
      () =>
        new Promise<void>((_resolve, reject) => {
          rejectSave = reject;
        }),
    );
    const wrapper = shallowMount(GenerationPresetEditor, {
      props: {
        open: true,
        preset: {
          id: "preset",
          name: "Original",
          actionId: "generate_image",
          target: { type: "agent", runtimeKind: "codex" },
          parameters: {},
        },
        savePreset,
        loadGenerationActions: async () => [{ id: "generate_image", routingPurpose: "image-generation" }],
      },
      global: {
        provide: leaveContext((value) => (participant = value)),
        stubs: { UiDialog: { template: "<div><slot /><slot name='footer' /></div>" } },
      },
    });
    await flushPromises();
    wrapper.findComponent(UiInput).vm.$emit("update:modelValue", "Changed");
    await wrapper.findAllComponents(UiButton).at(-1)!.trigger("click");
    const leaving = participant.prepare();

    rejectSave(new Error("disk full"));
    await expect(leaving).resolves.toBe(false);
    await flushPromises();
    expect(wrapper.props("open")).toBe(true);
    expect(wrapper.get('[role="alert"]').text()).toContain("disk full");
    wrapper.unmount();
  });
});
