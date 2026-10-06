import { shallowMount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
import { nextTick } from "vue";

import type { isFileDropInside, listenForFileDrops } from "../../../foundation/file-drop/file-drop";
import type { TaskTargetRef } from "../../../shared/contracts/generated/TaskTargetRef";
import SpaceConversationComposer from "../presentation/conversation/SpaceConversationComposer.vue";

vi.mock("../../../foundation/file-drop/file-drop", () => ({
  listenForFileDrops: vi.fn<typeof listenForFileDrops>(async () => () => {}),
  isFileDropInside: vi.fn<typeof isFileDropInside>(() => false),
}));

describe("Agent composer", () => {
  it.each<TaskTargetRef>([
    { type: "model", connectionId: "connection", modelId: "model" },
    { type: "agent", runtimeKind: "codex" },
  ])("submits for $type with no mode selection while retaining admission guards", async (target) => {
    const wrapper = shallowMount(SpaceConversationComposer, {
      props: {
        modelValue: "Explain the notes",
        target,
        busy: false,
        attachments: [],
        attachmentCandidates: [],
        pickingFiles: false,
      },
      global: { mocks: { $t: (key: string) => key } },
    });
    try {
      expect(wrapper.find("ui-select-stub").exists()).toBe(false);
      await wrapper.find("textarea").trigger("keydown", { key: "Enter" });
      expect(wrapper.emitted("send")).toBeUndefined();
      wrapper.findComponent({ name: "SpaceConversationTargetField" }).vm.$emit("update:available", true);
      await nextTick();
      await wrapper.find("textarea").trigger("keydown", { key: "Enter" });
      expect(wrapper.emitted("send")).toHaveLength(1);
      await wrapper.setProps({ busy: true });
      await wrapper.find("textarea").trigger("keydown", { key: "Enter" });
      expect(wrapper.emitted("send")).toHaveLength(1);
    } finally {
      wrapper.unmount();
    }
  });
});
