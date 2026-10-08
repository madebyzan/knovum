import { flushPromises, mount } from "@vue/test-utils";
import { expect, it, onTestFinished, vi } from "vitest";
import { defineComponent } from "vue";

import { i18n } from "../../../../../foundation/i18n/i18n";
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type * as ipc from "../../../speech/voice-reference-ipc";
import VoiceReferenceEditor from "../VoiceReferenceEditor.vue";

const { prepare, cancel } = vi.hoisted(() => ({
  prepare: vi.fn<typeof ipc.prepareVoiceReference>(),
  cancel: vi.fn<(jobId: string) => Promise<boolean>>(),
}));
vi.mock("@tauri-apps/api/core", () => ({ convertFileSrc: (path: string) => path }));
vi.mock("../../../speech/voice-reference-ipc", () => ({ prepareVoiceReference: prepare }));
vi.mock("../../../speech/ipc", () => ({ cancelSpeechGeneration: cancel }));
const Player = defineComponent({
  name: "SpeechMediaPlayer",
  props: ["kind", "source"],
  emits: ["loaded"],
  template: "<div />",
});

it.each(["current", "new-space"] as const)(
  "scopes a reference cancellation failure to the %s operation",
  async (context) => {
    prepare.mockReturnValue(new Promise(() => {}));
    cancel.mockRejectedValue(new Error("Old cancellation failed"));
    const wrapper = mount(VoiceReferenceEditor, {
      props: {
        spaceId: "first",
        language: "en",
        disabled: false,
        sourceNode: {
          id: "audio",
          type: "audio",
          initialPosition: { x: 0, y: 0 },
          data: {
            kind: "audio",
            title: "Reference",
            assetPath: "/voice.wav",
            fileName: "voice.wav",
            sourceRevision: "sha256:source",
          },
        },
      },
      global: { plugins: [i18n], stubs: { SpeechMediaPlayer: Player } },
    });
    onTestFinished(() => wrapper.unmount());
    wrapper.getComponent(Player).vm.$emit("loaded", 10_000);
    await flushPromises();
    await wrapper.get('[data-testid="reference-prepare"]').trigger("click");
    await flushPromises();
    if (context === "current") await wrapper.get('[data-testid="reference-cancel"]').trigger("click");
    else await wrapper.setProps({ spaceId: "second" });
    await flushPromises();
    expect(cancel).toHaveBeenCalled();
    expect(wrapper.find('[role="alert"]').exists()).toBe(context === "current");
  },
);

it.each(["audio", "video"] as const)(
  "uses the selected %s source and freezes its reference interval",
  async (kind) => {
    const assetPath = kind === "video" ? "/interview.mp4" : "/interview.wav";
    const base = { id: "reference", initialPosition: { x: 0, y: 0 } };
    const data = { title: "Interview", assetPath, fileName: "interview", sourceRevision: "sha256:source" };
    const node: SpaceCanvasNode =
      kind === "video"
        ? { ...base, type: "video", data: { ...data, kind: "video" } }
        : { ...base, type: "audio", data: { ...data, kind: "audio" } };
    prepare.mockResolvedValue({
      sourcePath: assetPath,
      sourceRevision: "sha256:source",
      startMs: 5_000,
      endMs: 15_000,
      transcript: "Interview",
    });
    const wrapper = mount(VoiceReferenceEditor, {
      props: { sourceNode: node, spaceId: "space", language: "en", disabled: false },
      global: { plugins: [i18n], stubs: { SpeechMediaPlayer: Player } },
    });
    onTestFinished(() => wrapper.unmount());
    expect(wrapper.find('[data-testid="reference-choose"]').exists()).toBe(false);
    expect(wrapper.getComponent(Player).props()).toMatchObject({ kind, source: assetPath });
    wrapper.getComponent(Player).vm.$emit("loaded", 20_000);
    await flushPromises();
    await wrapper.get('[data-testid="reference-start"]').setValue("5");
    await wrapper.get('[data-testid="reference-end"]').setValue("15");
    await wrapper.get('[data-testid="reference-prepare"]').trigger("click");
    await flushPromises();
    expect(prepare).toHaveBeenLastCalledWith(
      expect.objectContaining({ sourcePath: assetPath, startMs: 5_000, endMs: 15_000, spaceId: "space" }),
    );
    expect(wrapper.emitted("update:modelValue")?.at(-1)?.[0]).toMatchObject({
      sourceRevision: "sha256:source",
    });
  },
);
