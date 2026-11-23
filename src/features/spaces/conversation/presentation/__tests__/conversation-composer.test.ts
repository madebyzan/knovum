import { shallowMount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick, shallowRef } from "vue";

import type { AiReadiness } from "../../../../../foundation/ai/ai-readiness";
import type { isFileDropInside, listenForFileDrops } from "../../../../../foundation/file-drop/events";
import type { TaskTarget } from "../../../../../shared/contracts/generated/TaskTarget";
import SpaceConversationComposer from "../SpaceConversationComposer.vue";

const readinessState = vi.hoisted(() => ({ value: { state: "ready" } as AiReadiness }));
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));
vi.mock("../../../../../foundation/i18n/instance", () => ({
  i18n: { global: { t: (key: string) => key } },
}));
vi.mock("../../../../../foundation/ai/use-ai-readiness", () => ({
  useAiReadiness: () => shallowRef(readinessState.value),
}));
vi.mock("../../../../../foundation/file-drop/events", () => ({
  listenForFileDrops: vi.fn<typeof listenForFileDrops>(async () => () => {}),
  isFileDropInside: vi.fn<typeof isFileDropInside>(() => false),
}));

beforeEach(() => {
  readinessState.value = { state: "ready" };
});

function mountComposer(props: Record<string, unknown>) {
  return shallowMount(SpaceConversationComposer, {
    props: {
      modelValue: "Explain the notes",
      running: false,
      cancellable: false,
      disabled: false,
      canPause: false,
      attachments: [],
      attachmentCandidates: [],
      pickingFiles: false,
      unresolvedFileNames: [],
      ...props,
    },
    global: { mocks: { $t: (key: string) => key } },
  });
}

describe("Agent composer", () => {
  it.each<TaskTarget>([
    { type: "model", connectionId: "connection", modelId: "model" },
    { type: "agent", runtimeKind: "codex" },
  ])("submits for $type with no mode selection while retaining admission guards", async (target) => {
    const wrapper = mountComposer({ target });
    try {
      expect(wrapper.find("ui-select-stub").exists()).toBe(false);
      await wrapper.find("textarea").trigger("keydown", { key: "Enter" });
      expect(wrapper.emitted("send")).toBeUndefined();
      wrapper.findComponent({ name: "SpaceConversationTargetField" }).vm.$emit("update:available", true);
      await nextTick();
      await wrapper.find("textarea").trigger("keydown", { key: "Enter" });
      expect(wrapper.emitted("send")).toHaveLength(1);
      await wrapper.setProps({ running: true });
      await wrapper.find("textarea").trigger("keydown", { key: "Enter" });
      expect(wrapper.emitted("send")).toHaveLength(1);
    } finally {
      wrapper.unmount();
    }
  });

  it("blocks unresolved files and emits an explicit reference discard", async () => {
    const wrapper = mountComposer({
      target: { type: "agent", runtimeKind: "codex" },
      unresolvedFileNames: ["lost.pdf", "keep.pdf"],
    });
    try {
      wrapper.findComponent({ name: "SpaceConversationTargetField" }).vm.$emit("update:available", true);
      await nextTick();
      await wrapper.find("textarea").trigger("keydown", { key: "Enter" });
      expect(wrapper.emitted("send")).toBeUndefined();

      wrapper
        .findComponent({ name: "SpaceConversationAttachmentTray" })
        .vm.$emit("discardReference", "lost.pdf");
      expect(wrapper.emitted("discardFileReference")).toEqual([["lost.pdf"]]);

      await wrapper.setProps({ unresolvedFileNames: [] });
      await wrapper.find("textarea").trigger("keydown", { key: "Enter" });
      expect(wrapper.emitted("send")).toHaveLength(1);
    } finally {
      wrapper.unmount();
    }
  });

  it("offers stopping only while an answer is being produced, and only once it can be stopped", async () => {
    const wrapper = mountComposer({ target: { type: "agent", runtimeKind: "codex" } });
    const actions = () => wrapper.findAllComponents({ name: "UiIconButton" });
    const labelled = (label: string) => actions().find((button) => button.props("label") === label);
    try {
      // A conversation that is not ready yet disables sending; it must not offer to stop an answer
      // that was never asked for.
      await wrapper.setProps({ disabled: true });
      expect(labelled("spaces.conversation.stop")).toBeUndefined();
      expect(labelled("spaces.conversation.input.blocked.notReady")?.props("disabled")).toBe(true);

      // A request that has not been accepted yet has no run to pull: the control is there, so the
      // button does not move under the pointer, but it cannot lie about being able to stop.
      await wrapper.setProps({ disabled: false, running: true });
      expect(labelled("spaces.conversation.input.send")).toBeUndefined();
      expect(labelled("spaces.conversation.stop")?.props("disabled")).toBe(true);
      expect(labelled("common.pause")).toBeUndefined();

      await wrapper.setProps({ cancellable: true, canPause: true });
      expect(labelled("spaces.conversation.stop")?.props("disabled")).toBe(false);
      await labelled("spaces.conversation.stop")!.trigger("click");
      expect(wrapper.emitted("cancel")).toHaveLength(1);
      await labelled("common.pause")!.trigger("click");
      expect(wrapper.emitted("pause")).toHaveLength(1);
    } finally {
      wrapper.unmount();
    }
  });
});
